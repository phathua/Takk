use crate::types::{FileConfig, ProjectFile, ProjectFileConfig};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use std::io::Read;

// Tinh toan duong dan tuong doi tu target toi base
fn get_relative_path(target: &Path, base: &Path) -> Option<PathBuf> {
    let target = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());
    let base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());

    let mut target_comps = target.components().peekable();
    let mut base_comps = base.components().peekable();

    if target_comps.peek() != base_comps.peek() {
        return None;
    }

    while let (Some(t), Some(b)) = (target_comps.peek(), base_comps.peek()) {
        if t == b {
            target_comps.next();
            base_comps.next();
        } else {
            break;
        }
    }

    let mut result = PathBuf::new();
    for _ in base_comps {
        result.push("..");
    }
    for comp in target_comps {
        result.push(comp.as_os_str());
    }
    Some(result)
}

// Luu du an: tat ca luu bang postcard+zstd (format moi), tuong thich khi mo ca file .bg cu
pub fn save_project_to_file(project: &ProjectFile, path: &Path) -> anyhow::Result<()> {
    // Luu dang moi: postcard + zstd
    let serialized = postcard::to_stdvec(project)?;
    // Level 3: can bang giua toc do nen va ty le nen tot
    let compressed = zstd::encode_all(serialized.as_slice(), 3)?;
    std::fs::write(path, compressed)?;
    Ok(())
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileConfigV0 {
    pub path: PathBuf,
    pub sheet_name: Option<String>,
    pub brand: String,
    pub provider: String,
    pub headers: Vec<String>,
    pub mapping: HashMap<String, String>,
    pub normalize_basic: bool,
    pub normalize_special: bool,
    pub normalize_position: crate::types::SuffixPosition,
    pub normalize_suffix: String,
    pub generate_cost: bool,
    pub cost_discount_percent: f64,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectFileConfigV0 {
    pub config: FileConfigV0,
    pub file_name: String,
    pub extension: String,
    pub raw_data: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectFileV0 {
    pub version: u32,
    pub app_version: String,
    pub created_at: String,
    pub files: Vec<ProjectFileConfigV0>,
    #[serde(default)]
    pub export_format: Option<String>,
    #[serde(default)]
    pub app_mode: Option<String>,
}

impl From<FileConfigV0> for FileConfig {
    fn from(v0: FileConfigV0) -> Self {
        FileConfig {
            path: v0.path,
            sheet_name: v0.sheet_name,
            brand: v0.brand,
            provider: v0.provider,
            headers: v0.headers,
            mapping: v0.mapping,
            normalize_basic: v0.normalize_basic,
            normalize_special: v0.normalize_special,
            normalize_position: v0.normalize_position,
            normalize_suffix: v0.normalize_suffix,
            generate_cost: v0.generate_cost,
            cost_discount_percent: v0.cost_discount_percent,
            created_at: v0.created_at,
            not_found: false,
            file_hash: None,
        }
    }
}

impl From<ProjectFileConfigV0> for ProjectFileConfig {
    fn from(v0: ProjectFileConfigV0) -> Self {
        ProjectFileConfig {
            config: FileConfig::from(v0.config),
            file_name: v0.file_name,
            extension: v0.extension,
            raw_data: v0.raw_data,
        }
    }
}

impl From<ProjectFileV0> for ProjectFile {
    fn from(v0: ProjectFileV0) -> Self {
        ProjectFile {
            version: v0.version,
            app_version: v0.app_version,
            created_at: v0.created_at,
            files: v0.files.into_iter().map(ProjectFileConfig::from).collect(),
            export_format: v0.export_format,
            app_mode: v0.app_mode,
        }
    }
}

// Tai du an tu file .bg (hoac .bgx) co tu dong nhan dien va fallback sang format cu
pub fn load_project_from_file(path: &Path) -> anyhow::Result<ProjectFile> {
    let raw_data = std::fs::read(path)?;

    // 1. Thu dung Zstd + Postcard (Format moi)
    if let Ok(decompressed) = zstd::decode_all(raw_data.as_slice()) {
        if let Ok(project) = postcard::from_bytes::<ProjectFile>(&decompressed) {
            return Ok(project);
        }
        if let Ok(project_v0) = postcard::from_bytes::<ProjectFileV0>(&decompressed) {
            return Ok(ProjectFile::from(project_v0));
        }
    }

    // 2. Fallback sang Gzip + Bincode (Format cu cua phien ban 0.0.7 tro ve truoc)
    let mut decoder = GzDecoder::new(raw_data.as_slice());
    let mut decompressed = Vec::new();
    if decoder.read_to_end(&mut decompressed).is_ok() {
        if let Ok(project) = bincode::deserialize::<ProjectFile>(&decompressed) {
            return Ok(project);
        }
        if let Ok(project_v0) = bincode::deserialize::<ProjectFileV0>(&decompressed) {
            return Ok(ProjectFile::from(project_v0));
        }
    }

    Err(anyhow::anyhow!("Khong the doc file du an. Dinh dang khong hop le hoac bi hong."))
}

// Dong goi cac file va cau hinh thanh ProjectFile
pub fn pack_project_files(
    files: Vec<FileConfig>,
    export_format: Option<String>,
    app_mode: Option<String>,
    project_path: &Path,
) -> anyhow::Result<ProjectFile> {
    let mut project_configs = Vec::new();
    let is_bgx = project_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase() == "bgx")
        .unwrap_or(false);

    let project_dir = project_path.parent();

    for mut config in files {
        let file_name = config.path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let extension = config.path.extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_bytes = if config.path.exists() {
            std::fs::read(&config.path).ok()
        } else {
            None
        };

        // Tính toán mã băm BLAKE3 của tệp tin
        let file_hash = file_bytes.as_ref().map(|bytes| {
            blake3::hash(bytes).to_hex().to_string()
        });
        config.file_hash = file_hash;

        let raw_data = if is_bgx {
            // Neu la bgx, doi sang duong dan tuong doi neu cung o dia
            if let Some(proj_dir) = project_dir {
                if let Some(rel_path) = get_relative_path(&config.path, proj_dir) {
                    config.path = rel_path;
                }
            }
            Vec::new()
        } else {
            file_bytes.unwrap_or_default()
        };

        project_configs.push(ProjectFileConfig {
            config,
            file_name,
            extension,
            raw_data,
        });
    }

    Ok(ProjectFile {
        version: 1,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        files: project_configs,
        export_format,
        app_mode,
    })
}

// Loại bỏ tiền tố số và dấu gạch dưới ở đầu tên file (ví dụ "0_0_HONDA.xlsx" -> "HONDA.xlsx")
fn strip_index_prefix(file_name: &str) -> &str {
    let mut current = file_name;
    while let Some(idx) = current.find('_') {
        let prefix = &current[..idx];
        if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
            current = &current[idx + 1..];
        } else {
            break;
        }
    }
    current
}

// Giai nen cac file hoac giai quyet duong dan tham chieu
pub fn unpack_project_files(project: &ProjectFile, project_path: &Path) -> anyhow::Result<Vec<FileConfig>> {
    let temp_dir = std::env::temp_dir().join("Takk_Projects");
    std::fs::create_dir_all(&temp_dir)?;

    let project_dir = project_path.parent();
    let mut new_files = Vec::new();

    for (i, p_config) in project.files.iter().enumerate() {
        let mut config = p_config.config.clone();
        let mut not_found = false;

        if p_config.raw_data.is_empty() {
            // Day la file tham chieu cua .bgx
            let stored_path = &p_config.config.path;
            let mut resolved_path = stored_path.clone();

            if stored_path.is_relative() {
                if let Some(proj_dir) = project_dir {
                    let abs_path = proj_dir.join(stored_path);
                    if abs_path.exists() {
                        resolved_path = abs_path;
                    } else {
                        not_found = true;
                        resolved_path = abs_path;
                    }
                } else {
                    not_found = true;
                }
            } else {
                // Duong dan tuyet doi
                if !stored_path.exists() {
                    // Thu tim trong cung thu muc chua project
                    if let Some(proj_dir) = project_dir {
                        if let Some(file_name) = stored_path.file_name() {
                            let fallback_path = proj_dir.join(file_name);
                            if fallback_path.exists() {
                                resolved_path = fallback_path;
                            } else {
                                not_found = true;
                            }
                        } else {
                            not_found = true;
                        }
                    } else {
                        not_found = true;
                    }
                }
            }

            config.path = resolved_path;
            config.not_found = not_found;
        } else {
            // Day la file dong goi cua .bg
            let clean_name = strip_index_prefix(&p_config.file_name);
            let unique_name = format!("{}_{}", i, clean_name);
            let temp_file_path = temp_dir.join(unique_name);
            std::fs::write(&temp_file_path, &p_config.raw_data)?;

            config.path = temp_file_path;
            config.not_found = false;
        }

        new_files.push(config);
    }

    Ok(new_files)
}
