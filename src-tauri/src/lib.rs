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
use std::io::BufReader;

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
    let path_buf = PathBuf::from(&path);
    let project = project::pack_project_files(files, export_format, app_mode, &path_buf).map_err(|e| e.to_string())?;
    project::save_project_to_file(&project, &path_buf).map_err(|e| e.to_string())?;
    applog!("SUCCESS", "Luu du an thanh cong!");
    Ok(())
}

// Command tai du an .bg
#[tauri::command]
fn load_project(path: String) -> Result<LoadProjectResponse, String> {
    applog!("INFO", "Dang doc du an tu: {}", path);
    let path_buf = PathBuf::from(&path);
    let project = project::load_project_from_file(&path_buf).map_err(|e| e.to_string())?;
    let mut files = project::unpack_project_files(&project, &path_buf).map_err(|e| e.to_string())?;
    for file in &mut files {
        for (col_idx, h) in file.headers.iter_mut().enumerate() {
            if h.trim().is_empty() {
                *h = format!("Cột {} (Trống)", col_idx + 1);
            }
        }
    }
    applog!("SUCCESS", "Tai du an thanh cong! So file: {}", files.len());
    Ok(LoadProjectResponse {
        files,
        export_format: project.export_format,
        app_mode: project.app_mode,
    })
}

// Quet cac file .bg va .bgx tu cac thu muc nguoi dung hay luu (Downloads, Documents, Desktop, cac o dia D, E,...)
// Thuat toan duoc toi uu hoa bang cach duyet ngan xep (stack-based), gioi han do sau (4),
// gioi han tong so file (25k), gioi han thoi gian (800ms) va dung blacklist de bo qua cac thu muc nang.
#[tauri::command]
fn scan_suggested_projects() -> Result<Vec<types::SuggestedFile>, String> {
    let mut suggestions = Vec::new();
    let mut roots = Vec::new();
    // 1. Quet cac o dia khac (D:\, E:\, F:\, G:\, H:\) neu ton tai
    for drive_letter in b'D'..=b'H' {
        let drive_path = PathBuf::from(format!("{}:\\", drive_letter as char));
        if drive_path.exists() && drive_path.is_dir() {
            roots.push(drive_path);
        }
    }

    // 2. Thu muc dac biet cua User Profile (Downloads, Documents, Desktop)
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let user_profile_path = PathBuf::from(user_profile);
        let downloads = user_profile_path.join("Downloads");
        if downloads.exists() && downloads.is_dir() {
            roots.push(downloads);
        }
        let documents = user_profile_path.join("Documents");
        if documents.exists() && documents.is_dir() {
            roots.push(documents);
        }
        let desktop = user_profile_path.join("Desktop");
        if desktop.exists() && desktop.is_dir() {
            roots.push(desktop);
        }
    }

    let start_time = std::time::Instant::now();
    let max_duration = std::time::Duration::from_millis(800); // Gioi han thoi gian xu ly toi da

    // Danh sach cac thu muc chan de giam tai CPU/RAM
    let blacklist = [
        "appdata",
        "local settings",
        "application data",
        "program files",
        "program files (x86)",
        "windows",
        "system32",
        "node_modules",
        "target",
        ".git",
        ".cargo",
        ".rustup",
        ".gemini",
        "temp",
        "tmp",
        "$recycle.bin",
        "system volume information",
    ];

    // Ngan xep luu (duong_dan, do_sau)
    let mut stack = Vec::new();
    for r in roots {
        stack.push((r, 0));
    }

    let mut files_checked = 0;

    while let Some((dir, depth)) = stack.pop() {
        // Cu moi chu ky kiem tra thoi gian va so luong tệp de ngan chan doc qua tai
        if start_time.elapsed() > max_duration || files_checked > 25000 {
            break;
        }

        // Gioi han do sau quet de tranh di qua sau vao cac thu muc dev/he thong
        if depth > 4 {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                files_checked += 1;
                let path = entry.path();

                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let name_lower = name.to_lowercase();
                        // Bo qua cac thu muc thuoc blacklist hoac bat dau bang dau cham
                        if blacklist.contains(&name_lower.as_str()) || name.starts_with('.') {
                            continue;
                        }
                        stack.push((path, depth + 1));
                    }
                } else if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ext_lower == "bg" || ext_lower == "bgx" {
                            if let Ok(metadata) = entry.metadata() {
                                let size = metadata.len();
                                if size == 0 {
                                    continue; // Bo qua file trong
                                }
                                let modified = metadata.modified()
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
                                    .map(|d| d.as_millis() as u64)
                                    .unwrap_or(0);

                                let name = path.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default();

                                suggestions.push(types::SuggestedFile {
                                    path: path.to_string_lossy().to_string(),
                                    name,
                                    size,
                                    modified,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sap xep theo thoi gian sua doi giam dan (file moi nhat len dau)
    suggestions.sort_by(|a, b| b.modified.cmp(&a.modified));
    // Gioi han toi da 10 goi y tot nhat
    suggestions.truncate(10);

    Ok(suggestions)
}


// Lay dung luong (metadata size) cho danh sach duong dan truyen vao
#[tauri::command]
fn get_files_metadata(paths: Vec<String>) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    for p in paths {
        let path_buf = PathBuf::from(&p);
        if let Ok(metadata) = std::fs::metadata(&path_buf) {
            map.insert(p, metadata.len());
        }
    }
    map
}



// Ham chuan hoa ma phu tung
fn normalize_product_code(code: &str) -> String {
    code.chars()
        .filter(|c| *c != '-' && *c != ' ')
        .collect::<String>()
        .to_uppercase()
}

// Precompute map: ten_field -> chi_so_cot_trong_raw_row
// Tra ve None neu field khong co trong headers
fn build_field_col_map(file: &FileConfig) -> HashMap<String, usize> {
    let mut map = HashMap::with_capacity(file.mapping.len());
    for (field, header_col) in &file.mapping {
        if let Some(pos) = file.headers.iter().position(|h| h == header_col) {
            map.insert(field.clone(), pos);
        }
    }
    map
}

// Lay gia tri o theo ten field tu field_col_map, tra ve &str de tranh clone khong can
#[inline]
fn get_field<'a>(row: &'a [String], col_map: &HashMap<String, usize>, field: &str) -> &'a str {
    col_map.get(field).and_then(|&idx| row.get(idx)).map(|s| s.trim()).unwrap_or("")
}

// Ham tao PriceRow tu mot raw row, dung chung cho ca preview va export
fn build_price_row(
    row: &[String],
    col_map: &HashMap<String, usize>,
    file: &FileConfig,
    now: chrono::DateTime<chrono::Utc>,
    created_date: Option<chrono::NaiveDate>,
) -> Option<PriceRow> {
    let raw_code = get_field(row, col_map, "product_code");
    if raw_code.is_empty() { return None; }

    // Chuan hoa ma san pham
    let mut product_code = if file.normalize_basic {
        normalize_product_code(raw_code)
    } else {
        raw_code.to_string()
    };

    if file.normalize_special && !file.normalize_suffix.is_empty() {
        product_code = match file.normalize_position {
            SuffixPosition::Prefix => format!("{}{}", file.normalize_suffix, product_code),
            SuffixPosition::Suffix => format!("{}{}", product_code, file.normalize_suffix),
        };
    }

    let name = get_field(row, col_map, "name").to_string();
    let alt_code_str = get_field(row, col_map, "alt_code");
    let note_str = get_field(row, col_map, "note");
    let model_str = get_field(row, col_map, "model");
    let color_code_str = get_field(row, col_map, "color_code");
    let retail_str = get_field(row, col_map, "retail_price");
    let cost_str = get_field(row, col_map, "cost_price");

    let retail_val = retail_str.parse::<f64>().ok();
    let mut cost_val = cost_str.parse::<f64>().unwrap_or(0.0);

    if file.generate_cost && file.cost_discount_percent > 0.0 {
        if let Some(retail) = retail_val {
            cost_val = retail * (1.0 - file.cost_discount_percent / 100.0);
        }
    } else if cost_val == 0.0 {
        if let Some(retail) = retail_val {
            cost_val = retail;
        }
    }

    let fingerprint = format!("{}-{}-{}-{}", product_code, file.brand, file.provider, name);

    Some(PriceRow {
        product_code,
        alt_code: if alt_code_str.is_empty() { None } else { Some(alt_code_str.to_string()) },
        name,
        brand: file.brand.clone(),
        provider: file.provider.clone(),
        cost_price: cost_val,
        retail_price: retail_val,
        note: if note_str.is_empty() { None } else { Some(note_str.to_string()) },
        model: if model_str.is_empty() { None } else { Some(model_str.to_string()) },
        color_code: if color_code_str.is_empty() { None } else { Some(color_code_str.to_string()) },
        created_at: created_date,
        updated_at: Some(now),
        fingerprint,
    })
}

// Phat hien delimiter tu bytes dau file (khong doc toan bo file)
fn detect_csv_delimiter(path: &std::path::Path) -> u8 {
    use std::io::Read;
    let mut buf = [0u8; 4096];
    if let Ok(mut f) = std::fs::File::open(path) {
        let n = f.read(&mut buf).unwrap_or(0);
        let sample = &buf[..n];
        let has_tab = sample.contains(&b'\t');
        let has_comma = sample.contains(&b',');
        let has_semi = sample.contains(&b';');
        if has_tab && !has_comma { return b'\t'; }
        if has_semi && !has_comma { return b';'; }
    }
    b','
}

// Command lay du lieu xem truoc 3-5 dong cho moi file de toi uu toc do
#[tauri::command]
async fn get_preview_rows(
    files: Vec<FileConfig>,
    limit_per_file: usize,
) -> Result<Vec<PriceRow>, String> {
    crate::applog!("INFO", "Dang lay du lieu xem truoc cho {} file...", files.len());
    let mut all_rows: Vec<PriceRow> = Vec::with_capacity(files.len() * limit_per_file);
    // Goi 1 lan ngoai loop de tranh syscall lap lai
    let now = chrono::Utc::now();

    for file in &files {
        if !file.path.exists() {
            crate::applog!("WARN", "File khong ton tai luc xem truoc: {:?}", file.path);
            continue;
        }

        // Precompute field->col index 1 lan cho moi file, O(1) lookup per cell
        let col_map = build_field_col_map(file);
        // Parse ngay tao 1 lan ngoai vong for rows
        let created_date = chrono::NaiveDate::parse_from_str(&file.created_at, "%d/%m/%Y").ok();

        let ext = file.path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();

        if ext == "csv" {
            // Phat hien delimiter tu 4KB dau file, khong load toan bo
            let delimiter = detect_csv_delimiter(&file.path);
            // Stream truc tiep tu file, khong load vao String
            let f = std::fs::File::open(&file.path).map_err(|e| format!("Loi mo file CSV: {}", e))?;
            let reader = BufReader::new(f);
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_reader(reader);

            // Chi doc 20 dong dau cho preview de lay header va vai dong du lieu nhanh hon
            let mut raw_rows: Vec<Vec<String>> = Vec::with_capacity(20);
            for result in rdr.records().take(20) {
                let record = result.map_err(|e| e.to_string())?;
                raw_rows.push(record.iter().map(|s| s.to_string()).collect());
            }

            if raw_rows.is_empty() { continue; }
            let header_idx = worker::find_header_row(&raw_rows);

            for row in raw_rows.into_iter().skip(header_idx + 1).take(limit_per_file) {
                if let Some(price_row) = build_price_row(&row, &col_map, file, now, created_date) {
                    all_rows.push(price_row);
                }
            }
        } else {
            // Excel (xlsx, xls)
            let mut workbook = open_workbook_auto(&file.path).map_err(|e| e.to_string())?;
            let sheet = file.sheet_name.clone().unwrap_or_else(|| {
                workbook.sheet_names().first().cloned().unwrap_or_default()
            });

            if let Ok(range) = workbook.worksheet_range(&sheet) {
                // Chi doc 20 dong de lay preview nham toi uu toc do
                let mut raw_rows: Vec<Vec<String>> = Vec::with_capacity(20);
                for row in range.rows().take(20) {
                    raw_rows.push(row.iter().map(|cell| match cell {
                        calamine::Data::String(s) => s.clone(),
                        calamine::Data::Float(f) => f.to_string(),
                        calamine::Data::Int(i) => i.to_string(),
                        calamine::Data::Bool(b) => b.to_string(),
                        _ => String::new(),
                    }).collect());
                }

                if raw_rows.is_empty() { continue; }
                let header_idx = worker::find_header_row(&raw_rows);

                for row in raw_rows.into_iter().skip(header_idx + 1).take(limit_per_file) {
                    if let Some(price_row) = build_price_row(&row, &col_map, file, now, created_date) {
                        all_rows.push(price_row);
                    }
                }
            }
        }
    }

    crate::applog!("SUCCESS", "Lay du lieu xem truoc thanh cong! Tong so dong: {}", all_rows.len());
    Ok(all_rows)
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
    let now = chrono::Utc::now();

    for (idx, file) in files.iter().enumerate() {
        if !file.path.exists() {
            applog!("WARN", "File khong ton tai: {:?}", file.path);
            continue;
        }
        applog!("INFO", "Dang doc file ({}/{}): {:?}", idx + 1, files.len(), file.path.file_name().unwrap_or_default());

        // Precompute field->col index va parse date 1 lan cho moi file
        let col_map = build_field_col_map(file);
        let created_date = chrono::NaiveDate::parse_from_str(&file.created_at, "%d/%m/%Y").ok();
        let ext = file.path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();

        // 1. Doc du lieu tu Excel hoac CSV
        if ext == "csv" {
            // Phat hien delimiter tu 4KB dau file, khong load toan bo
            let delimiter = detect_csv_delimiter(&file.path);
            // Stream truc tiep tu file, khong can load toan bo vao String
            let f = std::fs::File::open(&file.path).map_err(|e| format!("Loi mo file CSV: {}", e))?;
            let reader = BufReader::new(f);
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_reader(reader);

            let mut raw_rows: Vec<Vec<String>> = Vec::new();
            for result in rdr.records() {
                let record = result.map_err(|e| e.to_string())?;
                raw_rows.push(record.iter().map(|s| s.to_string()).collect());
            }

            if raw_rows.is_empty() { continue; }
            let header_idx = worker::find_header_row(&raw_rows);

            for row in raw_rows.into_iter().skip(header_idx + 1) {
                if let Some(price_row) = build_price_row(&row, &col_map, file, now, created_date) {
                    all_rows.push(price_row);
                }
            }
        } else {
            // Excel (xlsx, xls)
            let mut workbook = open_workbook_auto(&file.path).map_err(|e| e.to_string())?;
            let sheet = file.sheet_name.clone().unwrap_or_else(|| {
                workbook.sheet_names().first().cloned().unwrap_or_default()
            });

            if let Ok(range) = workbook.worksheet_range(&sheet) {
                let mut raw_rows: Vec<Vec<String>> = Vec::new();
                for row in range.rows() {
                    raw_rows.push(row.iter().map(|cell| match cell {
                        calamine::Data::String(s) => s.clone(),
                        calamine::Data::Float(f) => f.to_string(),
                        calamine::Data::Int(i) => i.to_string(),
                        calamine::Data::Bool(b) => b.to_string(),
                        _ => String::new(),
                    }).collect());
                }

                if raw_rows.is_empty() { continue; }
                let header_idx = worker::find_header_row(&raw_rows);

                for row in raw_rows.into_iter().skip(header_idx + 1) {
                    if let Some(price_row) = build_price_row(&row, &col_map, file, now, created_date) {
                        all_rows.push(price_row);
                    }
                }
            }
        }
    }

    if all_rows.is_empty() {
        return Err("Khong co du lieu nao duoc gop.".to_string());
    }

    // 2. Xuat file CSV - ghi truc tiep, khong clone toan bo fields
    let path = PathBuf::from(&output_path);
    let mut wtr = csv::Writer::from_path(&path).map_err(|e| e.to_string())?;
    wtr.write_record([
        "Mã sản phẩm", "Mã cũ/thay thế", "Tên sản phẩm", "Hãng", "Nhà cung cấp",
        "Giá vốn", "Giá bán lẻ", "Đời xe", "Mã màu", "Ghi chú", "Ngày tạo"
    ]).map_err(|e| e.to_string())?;

    // Dung buffer tam de tranh alloc string tung truong
    let mut cost_buf = String::new();
    let mut retail_buf = String::new();
    let mut date_buf = String::new();
    for r in &all_rows {
        use std::fmt::Write as _;
        cost_buf.clear(); let _ = write!(cost_buf, "{}", r.cost_price);
        retail_buf.clear();
        if let Some(v) = r.retail_price { let _ = write!(retail_buf, "{}", v); }
        date_buf.clear();
        if let Some(d) = r.created_at { let _ = write!(date_buf, "{}", d.format("%d/%m/%Y")); }

        wtr.write_record(&[
            r.product_code.as_str(),
            r.alt_code.as_deref().unwrap_or(""),
            r.name.as_str(),
            r.brand.as_str(),
            r.provider.as_str(),
            cost_buf.as_str(),
            retail_buf.as_str(),
            r.model.as_deref().unwrap_or(""),
            r.color_code.as_deref().unwrap_or(""),
            r.note.as_deref().unwrap_or(""),
            date_buf.as_str(),
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
                if file_path.ends_with(".bg") || file_path.ends_with(".bgx") {
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
                if file_path.ends_with(".bg") || file_path.ends_with(".bgx") {
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
            get_preview_rows,
            scan_suggested_projects,
            get_files_metadata,
            updater::check_for_updates,
            updater::download_and_install
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
