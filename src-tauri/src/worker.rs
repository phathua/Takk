use crate::types::{FileConfig, LogLevel, MappingRules, ProcessingUpdate, SuffixPosition};
use calamine::{Reader, open_workbook_auto};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

// Tim dong tieu de tot nhat dua tren diem so tu khoa
pub fn find_header_row(rows: &[Vec<String>]) -> usize {
    let mut best_index = 0;
    let mut best_score = -1.0;

    for (idx, row) in rows.iter().enumerate().take(20) {
        let mut score = 0.0;
        let non_empty_cells: Vec<_> = row.iter()
            .map(|s| s.trim().to_lowercase().replace('\n', " ").replace('\r', " "))
            .filter(|s| !s.is_empty())
            .collect();
        
        let non_empty_count = non_empty_cells.len();
        if non_empty_count < 2 { continue; } 

        let cleaned_cells: Vec<String> = row.iter()
            .map(|s| clean_vietnamese(s))
            .filter(|s| !s.is_empty())
            .collect();

        for s in &cleaned_cells {
            if s == "stt" || s == "no" || s == "no." || s == "sott" || s == "thutu" { 
                score += 3.0; 
            }
            if s.contains("ma") || s.contains("code") || s.contains("partnumber") || s.contains("sku") { 
                score += 2.5; 
            }
            if s.contains("ten") || s.contains("name") || s.contains("diengiai") || s.contains("description") || s.contains("hanghoa") { 
                score += 2.5; 
            }
            if s.contains("gia") || s.contains("price") || s.contains("dongia") || s.contains("thanhtien") || s.contains("vnd") { 
                score += 2.0; 
            }
            if s.contains("xe") || s.contains("model") || s.contains("loai") || s.contains("hieu") || s.contains("doi") { 
                score += 1.2; 
            }
            if s.contains("ghichu") || s.contains("note") || s.contains("mau") || s.contains("color") { 
                score += 1.0; 
            }
        }

        let mut keyword_groups = 0;
        if cleaned_cells.iter().any(|s| s.contains("ma") || s.contains("code")) { keyword_groups += 1; }
        if cleaned_cells.iter().any(|s| s.contains("ten") || s.contains("name")) { keyword_groups += 1; }
        if cleaned_cells.iter().any(|s| s.contains("gia") || s.contains("price")) { keyword_groups += 1; }
        
        if keyword_groups >= 2 {
            score += 5.0;
        }

        score += (non_empty_count as f32) * 0.1;

        if score > best_score {
            best_score = score;
            best_index = idx;
        }
    }

    best_index
}

// Doc file text voi ma hoa tuong thich nhieu bang ma
pub fn read_file_to_string_robust(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    
    // Check BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (decoded, _, _) = encoding_rs::UTF_8.decode(&bytes[3..]);
        return Ok(decoded.into_owned());
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return Ok(decoded.into_owned());
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        let (decoded, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return Ok(decoded.into_owned());
    }
    
    // Thu UTF-8
    let (decoded, _encoding, has_errors) = encoding_rs::UTF_8.decode(&bytes);
    if !has_errors {
        return Ok(decoded.into_owned());
    }
    
    // Fallback sang Windows-1258 (tieng Viet)
    let (decoded, _, _) = encoding_rs::WINDOWS_1258.decode(&bytes);
    Ok(decoded.into_owned())
}

// Lay danh sach sheets va headers tuong ung
pub fn get_sheets_and_headers(path: &Path) -> anyhow::Result<Vec<(Option<String>, Vec<String>)>> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();
    let mut results = Vec::new();

    if ext == "csv" {
        let content = read_file_to_string_robust(path)?;
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
        let mut rows = Vec::new();
        for result in rdr.records().take(50) {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }
        if !rows.is_empty() {
            let header_idx = find_header_row(&rows);
            let headers = rows[header_idx].iter().enumerate().map(|(col_idx, s)| {
                let trimmed = s.trim().to_string();
                if trimmed.is_empty() {
                    format!("Cột {} (Trống)", col_idx + 1)
                } else {
                    trimmed
                }
            }).collect();
            results.push((None, headers));
        }
    } else {
        let mut workbook = open_workbook_auto(path)?;
        let sheet_names = workbook.sheet_names().to_vec();
        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let mut rows = Vec::new();
                for row in range.rows() {
                    let r: Vec<String> = row.iter().map(|cell| match cell {
                        calamine::Data::String(s) => s.clone(),
                        calamine::Data::Float(f) => f.to_string(),
                        calamine::Data::Int(i) => i.to_string(),
                        calamine::Data::Bool(b) => b.to_string(),
                        _ => String::new(),
                    }).collect();
                    rows.push(r);
                }
                if !rows.is_empty() {
                    let header_idx = find_header_row(&rows);
                    let headers = rows[header_idx].iter().enumerate().map(|(col_idx, s)| {
                        let trimmed = s.trim().to_string();
                        if trimmed.is_empty() {
                            format!("Cột {} (Trống)", col_idx + 1)
                        } else {
                            trimmed
                        }
                    }).collect();
                    results.push((Some(sheet_name), headers));
                }
            }
        }
    }
    Ok(results)
}

fn clean_vietnamese(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if ('\u{0300}'..='\u{036f}').contains(&c) {
            continue;
        }
        let mapped = match c {
            'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ' | 'ắ' | 'ặ' | 'ẳ' | 'ẵ' |
            'À' | 'Á' | 'Ạ' | 'Ả' | 'Ã' | 'Â' | 'Ầ' | 'Ấ' | 'Ậ' | 'Ẩ' | 'Ẫ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ặ' | 'Ẳ' | 'Ẵ' => 'a',
            'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' |
            'È' | 'É' | 'Ẹ' | 'Ẻ' | 'Ẽ' | 'Ê' | 'Ề' | 'Ế' | 'Ệ' | 'Ể' | 'Ễ' => 'e',
            'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' |
            'Ì' | 'Í' | 'Ị' | 'Ỉ' | 'Ĩ' => 'i',
            'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ' | 'ớ' | 'ợ' | 'ở' | 'ỡ' |
            'Ò' | 'Ó' | 'Ọ' | 'Ỏ' | 'Õ' | 'Ô' | 'Ồ' | 'Ố' | 'Ộ' | 'Ổ' | 'Ỗ' | 'Ơ' | 'Ờ' | 'Ớ' | 'Ợ' | 'Ở' | 'Ỡ' => 'o',
            'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' |
            'Ù' | 'Ú' | 'Ụ' | 'Ủ' | 'Ũ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ự' | 'Ử' | 'Ữ' => 'u',
            'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' |
            'Ỳ' | 'Ý' | 'Y' | 'Ỷ' | 'Ỹ' => 'y',
            'đ' | 'Đ' => 'd',
            _ => c,
        };
        result.push(mapped);
    }
    result.to_lowercase().replace(' ', "").replace('_', "").replace('-', "")
}

// Goi y anh xa cot tu dong
pub fn suggest_mapping(headers: &[String], rules: &MappingRules) -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    for h in headers {
        let h_lower = h.to_lowercase().trim().to_string();
        if h_lower == "product_code" || h_lower == "name" || h_lower == "retail_price" 
            || h_lower == "cost_price" || h_lower == "alt_code" || h_lower == "brand" 
            || h_lower == "provider" || h_lower == "model" || h_lower == "color_code" 
            || h_lower == "note" 
        {
            mapping.insert(h_lower.clone(), h.clone());
            continue;
        }

        let h_clean = clean_vietnamese(h);

        let match_rule = |rule_str: &str| -> bool {
            rule_str.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .any(|k| {
                    let k_clean = clean_vietnamese(k);
                    !k_clean.is_empty() && h_clean.contains(&k_clean)
                })
        };

        if match_rule(&rules.ignore) {
            continue;
        }

        if match_rule(&rules.product_code) {
            mapping.entry("product_code".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.alt_code) {
            mapping.entry("alt_code".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.name) {
            mapping.entry("name".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.cost_price) {
            mapping.entry("cost_price".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.retail_price) {
            mapping.entry("retail_price".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.model) {
            mapping.entry("model".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.color_code) {
            mapping.entry("color_code".to_string()).or_insert(h.clone());
        } else if match_rule(&rules.note) {
            mapping.entry("note".to_string()).or_insert(h.clone());
        }
    }
    mapping
}

// Worker thread xu ly add files bat dong bo
pub fn run_add_files_worker(
    app: AppHandle,
    paths: Vec<PathBuf>,
    brand_mappings: Vec<crate::types::BrandProviderMapping>,
    mapping_rules: MappingRules,
) {
    let total = paths.len();
    for (idx, path) in paths.iter().enumerate() {
        let file_path = path.clone();
        let filename = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let progress = (idx as f32) / (total as f32);
        
        let _ = app.emit("progress-update", ProcessingUpdate::Progress {
            percent: progress,
            task: format!("Dang xu ly cau hinh ({} / {}): {}", idx + 1, total, filename),
        });
        let _ = app.emit("progress-update", ProcessingUpdate::Log {
            level: LogLevel::Info,
            message: format!("Dang nhan dien cau hinh tep: {}", filename),
        });

        let mut brand = String::new();
        let mut provider = String::new();

        let filename_lower = filename.to_lowercase();
        for mapping in &brand_mappings {
            if filename_lower.contains(&mapping.brand.to_lowercase()) || filename_lower.contains(&mapping.provider.to_lowercase()) {
                brand = mapping.brand.clone();
                provider = mapping.provider.clone();
                break;
            }
        }

        let sheets = match get_sheets_and_headers(&file_path) {
            Ok(s) => s,
            Err(e) => {
                let _ = app.emit("progress-update", ProcessingUpdate::Log {
                    level: LogLevel::Error,
                    message: format!("Loi doc file '{}': {}", file_path.display(), e),
                });
                continue;
            }
        };

        let mut any_sheet_valid = false;
        for (sheet, headers) in sheets {
            let mapping = suggest_mapping(&headers, &mapping_rules);
            let mut auto_brand = brand.clone();
            let mut auto_provider = provider.clone();

            // Guess brand/provider if empty
            if auto_brand.is_empty() {
                let suzuki_models = ["SUZUKI", "FU150", "SATRIA", "RAIDER", "GSX", "HAYATE", "VIVA"];
                let honda_models = ["HONDA", "AIRBLADE", "VISION", "SH150", "LEAD", "WINNER", "VARIO", "WAVE", "DREAM"];
                let yamaha_models = ["YAMAHA", "EXCITER", "NVX", "SIRIUS", "JUPITER", "GRANDE", "JANUS"];
                
                if suzuki_models.iter().any(|m| filename_lower.contains(&m.to_lowercase())) {
                    auto_brand = "SUZUKI".to_string();
                } else if honda_models.iter().any(|m| filename_lower.contains(&m.to_lowercase())) {
                    auto_brand = "HONDA".to_string();
                } else if yamaha_models.iter().any(|m| filename_lower.contains(&m.to_lowercase())) {
                    auto_brand = "YAMAHA".to_string();
                }
            }

            if auto_provider.is_empty() && !auto_brand.is_empty() {
                for bm in &brand_mappings {
                    if bm.brand.to_uppercase() == auto_brand.to_uppercase() {
                        auto_provider = bm.provider.clone();
                        break;
                    }
                }
            }

            let is_minh_dong = auto_provider.to_uppercase().contains("MINH DONG") || auto_provider.to_uppercase().contains("MINH ĐÔNG");
            let created_at = if let Ok(metadata) = std::fs::metadata(&file_path) {
                let time = metadata.created().or_else(|_| metadata.modified()).unwrap_or_else(|_| std::time::SystemTime::now());
                let datetime: chrono::DateTime<chrono::Local> = time.into();
                datetime.format("%d/%m/%Y").to_string()
            } else {
                chrono::Local::now().format("%d/%m/%Y").to_string()
            };

            let config = FileConfig {
                path: file_path.clone(),
                sheet_name: sheet,
                brand: auto_brand,
                provider: auto_provider,
                headers,
                mapping,
                normalize_basic: true,
                normalize_special: is_minh_dong,
                normalize_position: SuffixPosition::Suffix,
                normalize_suffix: if is_minh_dong { "@".to_string() } else { String::new() },
                generate_cost: false,
                cost_discount_percent: 0.0,
                created_at,
                not_found: false,
                file_hash: {
                    let bytes = std::fs::read(&file_path).ok();
                    bytes.map(|b| blake3::hash(&b).to_hex().to_string())
                },
                original_path: Some(file_path.clone()),
            };

            let _ = app.emit("progress-update", ProcessingUpdate::Log {
                level: LogLevel::Info,
                message: format!("Auto-detect {}: Hang={}, NCC={}", filename, config.brand, config.provider),
            });
            let _ = app.emit("progress-update", ProcessingUpdate::FileAdded(config));
            any_sheet_valid = true;
        }

        if !any_sheet_valid {
            let _ = app.emit("progress-update", ProcessingUpdate::Log {
                level: LogLevel::Warning,
                message: format!("Tu choi file '{}' vi khong tim thay trang tinh hop le", filename),
            });
        }
    }
    let _ = app.emit("progress-update", ProcessingUpdate::AddFilesFinished);
}
