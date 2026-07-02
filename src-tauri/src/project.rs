use crate::types::{FileConfig, ProjectFile, ProjectFileConfig};
use std::path::Path;

use flate2::read::GzDecoder;
use std::io::Read;

// Luu du an: .bg luu bang bincode+gzip (format cu), .bgx hoac format khac luu bang postcard+zstd (format moi)
pub fn save_project_to_file(project: &ProjectFile, path: &Path) -> anyhow::Result<()> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    if extension.as_deref() == Some("bg") {
        // Luu dang cu: bincode + gzip (flate2)
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let serialized = bincode::serialize(project)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&serialized)?;
        let compressed = encoder.finish()?;
        std::fs::write(path, compressed)?;
    } else {
        // Luu dang moi: postcard + zstd
        let serialized = postcard::to_stdvec(project)?;
        // Level 3: can bang giua toc do nen va ty le nen tot
        let compressed = zstd::encode_all(serialized.as_slice(), 3)?;
        std::fs::write(path, compressed)?;
    }
    Ok(())
}

// Tai du an tu file .bg (hoac .bgx) co tu dong nhan dien va fallback sang format cu
pub fn load_project_from_file(path: &Path) -> anyhow::Result<ProjectFile> {
    let raw_data = std::fs::read(path)?;

    // 1. Thu dung Zstd + Postcard (Format moi)
    if let Ok(decompressed) = zstd::decode_all(raw_data.as_slice()) {
        if let Ok(project) = postcard::from_bytes(&decompressed) {
            return Ok(project);
        }
    }

    // 2. Fallback sang Gzip + Bincode (Format cu cua phien ban 0.0.7 tro ve truoc)
    let mut decoder = GzDecoder::new(raw_data.as_slice());
    let mut decompressed = Vec::new();
    if decoder.read_to_end(&mut decompressed).is_ok() {
        if let Ok(project) = bincode::deserialize(&decompressed) {
            return Ok(project);
        }
    }

    Err(anyhow::anyhow!("Khong the doc file du an. Dinh dang khong hop le hoac bi hong."))
}

// Dong goi cac file va cau hinh thanh ProjectFile
pub fn pack_project_files(
    files: Vec<FileConfig>,
    export_format: Option<String>,
    app_mode: Option<String>,
) -> anyhow::Result<ProjectFile> {
    let mut project_configs = Vec::new();

    for config in files {
        if config.path.exists() {
            let raw_data = std::fs::read(&config.path)?;
            let file_name = config.path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let extension = config.path.extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            project_configs.push(ProjectFileConfig {
                config,
                file_name,
                extension,
                raw_data,
            });
        }
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

// Giai nen cac file raw vao thu muc tam va cap nhat duong dan moi
pub fn unpack_project_files(project: &ProjectFile) -> anyhow::Result<Vec<FileConfig>> {
    let temp_dir = std::env::temp_dir().join("Takk_Projects");
    std::fs::create_dir_all(&temp_dir)?;

    let mut new_files = Vec::new();

    for (i, p_config) in project.files.iter().enumerate() {
        let clean_name = strip_index_prefix(&p_config.file_name);
        let unique_name = format!("{}_{}", i, clean_name);
        let temp_file_path = temp_dir.join(unique_name);
        std::fs::write(&temp_file_path, &p_config.raw_data)?;

        let mut config = p_config.config.clone();
        config.path = temp_file_path;
        new_files.push(config);
    }

    Ok(new_files)
}
