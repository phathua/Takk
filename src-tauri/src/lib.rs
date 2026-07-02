#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod types;
pub mod project;
pub mod worker;
pub mod updater;

use types::{FileConfig, BrandProviderMapping, MappingRules, SuffixPosition, PriceRow, LoadProjectResponse};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::AppHandle;
use calamine::{Reader, open_workbook_auto};

pub static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

#[macro_export]
macro_rules! applog {
    ($level:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        if let Some(app) = $crate::APP_HANDLE.get() {
            use tauri::Emitter;
            let log_msg = format!("[{}] {}", $level, msg);
            let _ = app.emit("app-log", log_msg);
        }
    }};
}

// Command de trigger worker doc file bat dong bo
#[tauri::command]
fn add_files_async(
    app: AppHandle,
    paths: Vec<String>,
    brand_mappings: Vec<BrandProviderMapping>,
    mapping_rules: MappingRules,
) {
    let path_bufs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    std::thread::spawn(move || {
        worker::run_add_files_worker(app, path_bufs, brand_mappings, mapping_rules);
    });
}

// Command luu du an .bg
#[tauri::command]
fn save_project(
    files: Vec<FileConfig>,
    path: String,
    export_format: Option<String>,
    app_mode: Option<String>,
) -> Result<(), String> {
    applog!("INFO", "Dang luu du an vao: {}", path);
    let project = project::pack_project_files(files, export_format, app_mode).map_err(|e| e.to_string())?;
    project::save_project_to_file(&project, &PathBuf::from(&path)).map_err(|e| e.to_string())?;
    applog!("SUCCESS", "Luu du an thanh cong!");
    Ok(())
}

// Command tai du an .bg
#[tauri::command]
fn load_project(path: String) -> Result<LoadProjectResponse, String> {
    applog!("INFO", "Dang doc du an tu: {}", path);
    let project = project::load_project_from_file(&PathBuf::from(&path)).map_err(|e| e.to_string())?;
    let files = project::unpack_project_files(&project).map_err(|e| e.to_string())?;
    applog!("SUCCESS", "Tai du an thanh cong! So file: {}", files.len());
    Ok(LoadProjectResponse {
        files,
        export_format: project.export_format,
        app_mode: project.app_mode,
    })
}

// Ham chuan hoa ma phu tung
fn normalize_product_code(code: &str) -> String {
    code.trim()
        .replace('-', "")
        .replace(' ', "")
        .to_uppercase()
}

// Command xu ly va xuat file gop Excel/CSV
#[tauri::command]
fn process_and_export(
    files: Vec<FileConfig>,
    _export_format: String,
    output_path: String,
) -> Result<String, String> {
    applog!("INFO", "Bat dau gop va xuat {} file...", files.len());
    let mut all_rows: Vec<PriceRow> = Vec::new();

    for (idx, file) in files.iter().enumerate() {
        if !file.path.exists() {
            applog!("WARN", "File khong ton tai: {:?}", file.path);
            continue;
        }

        applog!("INFO", "Dang doc file ({}/{}): {:?}", idx + 1, files.len(), file.path.file_name().unwrap_or_default());
        
        let mut file_rows = Vec::new();
        let ext = file.path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();

        // 1. Doc du lieu tu Excel hoac CSV
        if ext == "csv" {
            let content = worker::read_file_to_string_robust(&file.path)
                .map_err(|e| format!("Loi doc file CSV: {}", e))?;
            let mut delimiter = b',';
            if content.contains('\t') && !content.contains(',') {
                delimiter = b'\t';
            } else if content.contains(';') && !content.contains(',') {
                delimiter = b';';
            }

            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_reader(content.as_bytes());
            
            let mut raw_rows = Vec::new();
            for result in rdr.records() {
                let record = result.map_err(|e| e.to_string())?;
                let r: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                raw_rows.push(r);
            }

            if raw_rows.is_empty() { continue; }
            let header_idx = worker::find_header_row(&raw_rows);
            
            // Map tung hang
            for row_data in raw_rows.into_iter().skip(header_idx + 1) {
                let mut mapped_row = HashMap::new();
                for (field, header_col) in &file.mapping {
                    if let Some(pos) = file.headers.iter().position(|h| h == header_col) {
                        if let Some(val) = row_data.get(pos) {
                            mapped_row.insert(field.clone(), val.trim().to_string());
                        }
                    }
                }
                file_rows.push(mapped_row);
            }
        } else {
            // Excel (xlsx, xls)
            let mut workbook = open_workbook_auto(&file.path).map_err(|e| e.to_string())?;
            let sheet = file.sheet_name.clone().unwrap_or_else(|| {
                workbook.sheet_names().first().cloned().unwrap_or_default()
            });

            if let Ok(range) = workbook.worksheet_range(&sheet) {
                let mut raw_rows = Vec::new();
                for row in range.rows() {
                    let r: Vec<String> = row.iter().map(|cell| match cell {
                        calamine::Data::String(s) => s.clone(),
                        calamine::Data::Float(f) => f.to_string(),
                        calamine::Data::Int(i) => i.to_string(),
                        calamine::Data::Bool(b) => b.to_string(),
                        _ => String::new(),
                    }).collect();
                    raw_rows.push(r);
                }

                if raw_rows.is_empty() { continue; }
                let header_idx = worker::find_header_row(&raw_rows);

                for row_data in raw_rows.into_iter().skip(header_idx + 1) {
                    let mut mapped_row = HashMap::new();
                    for (field, header_col) in &file.mapping {
                        if let Some(pos) = file.headers.iter().position(|h| h == header_col) {
                            if let Some(val) = row_data.get(pos) {
                                mapped_row.insert(field.clone(), val.trim().to_string());
                            }
                        }
                    }
                    file_rows.push(mapped_row);
                }
            }
        }

        // 2. Chuan hoa va tao doi tuong PriceRow
        for item in file_rows {
            let mut product_code = item.get("product_code").cloned().unwrap_or_default();
            if product_code.is_empty() { continue; }

            // Chuan hoa co ban
            if file.normalize_basic {
                product_code = normalize_product_code(&product_code);
            }

            // Chuan hoa dac biet (Prefix / Suffix)
            if file.normalize_special && !file.normalize_suffix.is_empty() {
                match file.normalize_position {
                    SuffixPosition::Prefix => {
                        product_code = format!("{}{}", file.normalize_suffix, product_code);
                    }
                    SuffixPosition::Suffix => {
                        product_code = format!("{}{}", product_code, file.normalize_suffix);
                    }
                }
            }

            let name = item.get("name").cloned().unwrap_or_default();
            let alt_code = item.get("alt_code").cloned().filter(|s| !s.is_empty());
            let note = item.get("note").cloned().filter(|s| !s.is_empty());
            let model = item.get("model").cloned().filter(|s| !s.is_empty());
            let color_code = item.get("color_code").cloned().filter(|s| !s.is_empty());

            // Tinh toan gia ca
            let retail_val = item.get("retail_price").and_then(|s| s.parse::<f64>().ok());
            let mut cost_val = item.get("cost_price").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

            if file.generate_cost && file.cost_discount_percent > 0.0 {
                if let Some(retail) = retail_val {
                    cost_val = retail * (1.0 - file.cost_discount_percent / 100.0);
                }
            } else if cost_val == 0.0 {
                if let Some(retail) = retail_val {
                    cost_val = retail;
                }
            }

            // Tinh toan fingerprint de loai trung
            let fingerprint = format!("{}-{}-{}-{}", product_code, file.brand, file.provider, name);

            all_rows.push(PriceRow {
                product_code,
                alt_code,
                name,
                brand: file.brand.clone(),
                provider: file.provider.clone(),
                cost_price: cost_val,
                retail_price: retail_val,
                note,
                model,
                color_code,
                created_at: chrono::NaiveDate::parse_from_str(&file.created_at, "%d/%m/%Y").ok(),
                updated_at: Some(chrono::Utc::now()),
                fingerprint,
            });
        }
    }

    if all_rows.is_empty() {
        return Err("Khong co du lieu nao duoc gop.".to_string());
    }

    // 3. Xuat file
    let path = PathBuf::from(&output_path);
    let mut wtr = csv::Writer::from_path(&path).map_err(|e| e.to_string())?;
    wtr.write_record([
        "Mã sản phẩm", "Mã cũ/thay thế", "Tên sản phẩm", "Hãng", "Nhà cung cấp",
        "Giá vốn", "Giá bán lẻ", "Đời xe", "Mã màu", "Ghi chú", "Ngày tạo"
    ]).map_err(|e| e.to_string())?;

    for r in &all_rows {
        wtr.write_record(&[
            r.product_code.clone(),
            r.alt_code.clone().unwrap_or_default(),
            r.name.clone(),
            r.brand.clone(),
            r.provider.clone(),
            r.cost_price.to_string(),
            r.retail_price.map(|v| v.to_string()).unwrap_or_default(),
            r.model.clone().unwrap_or_default(),
            r.color_code.clone().unwrap_or_default(),
            r.note.clone().unwrap_or_default(),
            r.created_at.map(|d| d.format("%d/%m/%Y").to_string()).unwrap_or_default(),
        ]).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;

    let success_msg = format!("Da gop va xuat {} san pham ra file: {:?}", all_rows.len(), path.file_name().unwrap_or_default());
    applog!("SUCCESS", "{}", success_msg);
    Ok(success_msg)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            use tauri::{Emitter, Manager};
            // Nhap dup mo file tu instance thu hai
            if args.len() > 1 {
                let file_path = &args[1];
                if file_path.ends_with(".bg") {
                    crate::applog!("INFO", "[SingleInstance] Nhan file tu instance khac: {}", file_path);
                    let _ = app.emit("open-project-tab", file_path);
                }
            }
            // Focus vao cua so chinh
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            use tauri::Emitter;
            let _ = APP_HANDLE.set(app.handle().clone());
            
            // Tu dong mo file neu co doi so truyen vao luc khoi dong
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let file_path = &args[1];
                if file_path.ends_with(".bg") {
                    let handle = app.handle().clone();
                    let path_clone = file_path.clone();
                    tauri::async_runtime::spawn(async move {
                        // Cho frontend khoi tao xong truoc khi emit event
                        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
                        crate::applog!("INFO", "[Startup] Mo file tu doi so dau vao: {}", path_clone);
                        let _ = handle.emit("open-project-tab", path_clone);
                    });
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_files_async,
            save_project,
            load_project,
            process_and_export,
            updater::check_for_updates,
            updater::download_and_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
