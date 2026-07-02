use crate::types::{FileConfig, ProjectFile, ProjectFileConfig};
use std::io::{Read, Write};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::path::Path;

// Luu du an .bg (nen Gzip + Bincode)
pub fn save_project_to_file(project: &ProjectFile, path: &Path) -> anyhow::Result<()> {
    let serialized = bincode::serialize(project)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&serialized)?;
    let compressed = encoder.finish()?;
    std::fs::write(path, compressed)?;
    Ok(())
}

// Tai du an tu file .bg
pub fn load_project_from_file(path: &Path) -> anyhow::Result<ProjectFile> {
    let data = std::fs::read(path)?;
    let mut decoder = GzDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    let project: ProjectFile = bincode::deserialize(&decompressed)?;
    Ok(project)
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

// Giai nen cac file raw vao thu muc tam va cap nhat duong dan moi
pub fn unpack_project_files(project: &ProjectFile) -> anyhow::Result<Vec<FileConfig>> {
    let temp_dir = std::env::temp_dir().join("Takk_Projects");
    std::fs::create_dir_all(&temp_dir)?;

    let mut new_files = Vec::new();

    for (i, p_config) in project.files.iter().enumerate() {
        let unique_name = format!("{}_{}", i, p_config.file_name);
        let temp_file_path = temp_dir.join(unique_name);
        std::fs::write(&temp_file_path, &p_config.raw_data)?;

        let mut config = p_config.config.clone();
        config.path = temp_file_path;
        new_files.push(config);
    }

    Ok(new_files)
}
