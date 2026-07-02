use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub url: String,
    pub download_url: Option<String>,
    pub body: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

// Ham phan tich version dang X.Y.Z thanh Vec<u32>
fn parse_version(v: &str) -> Vec<u32> {
    v.trim_start_matches('v')
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect()
}

// So sanh xem phien ban remote co moi hon local hay khong
fn is_newer(current: &str, remote: &str) -> bool {
    let curr_parts = parse_version(current);
    let rem_parts = parse_version(remote);
    for i in 0..std::cmp::max(curr_parts.len(), rem_parts.len()) {
        let curr = *curr_parts.get(i).unwrap_or(&0);
        let rem = *rem_parts.get(i).unwrap_or(&0);
        if rem > curr {
            return true;
        } else if curr > rem {
            return false;
        }
    }
    false
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    crate::applog!("INFO", "[Updater] Dang gui yeu cau kiem tra cap nhat len GitHub...");

    let client = reqwest::Client::new();
    
    let res = client
        .get("https://api.github.com/repos/phathua/Takk/releases/latest")
        .header("User-Agent", "Takk-App")
        .send()
        .await
        .map_err(|e| {
            let err_msg = format!("Loi gui yeu cau den GitHub API: {}", e);
            crate::applog!("ERROR", "[Updater] {}", err_msg);
            err_msg
        })?;

    let current_version = env!("CARGO_PKG_VERSION");

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        crate::applog!("INFO", "[Updater] Chua co ban phat hanh (release) nao tren GitHub.");
        return Ok(UpdateCheckResult {
            has_update: false,
            current_version: current_version.to_string(),
            latest_version: "Chua co".to_string(),
            url: "https://github.com/phathua/Takk".to_string(),
            download_url: None,
            body: Some("Chua co ban phat hanh nao tren GitHub.".to_string()),
        });
    }

    if !res.status().is_success() {
        let status_code = res.status();
        let err_msg = format!("GitHub API tra ve loi status: {}", status_code);
        crate::applog!("ERROR", "[Updater] {}", err_msg);
        return Err(err_msg);
    }

    let release: GithubRelease = res.json().await.map_err(|e| {
        let err_msg = format!("Loi doc du lieu JSON tu GitHub: {}", e);
        crate::applog!("ERROR", "[Updater] {}", err_msg);
        err_msg
    })?;

    let has_update = is_newer(current_version, &release.tag_name);

    let download_url = release.assets
        .iter()
        .find(|asset| asset.name.ends_with(".exe"))
        .map(|asset| asset.browser_download_url.clone());

    if has_update {
        crate::applog!("SUCCESS", "[Updater] Phat hien phien ban moi: {}", release.tag_name);
    } else {
        crate::applog!("INFO", "[Updater] Ung dung dang o phien ban moi nhat: {}", current_version);
    }

    Ok(UpdateCheckResult {
        has_update,
        current_version: current_version.to_string(),
        latest_version: release.tag_name,
        url: release.html_url,
        download_url,
        body: release.body,
    })
}

#[tauri::command]
pub async fn download_and_install(app: tauri::AppHandle, url: String) -> Result<(), String> {
    crate::applog!("INFO", "[Updater] Bat dau tai ban cap nhat tu: {}", url);

    let token = "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";
    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .header("User-Agent", "Takk-App")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Loi khi ket noi de tai file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Loi phan hoi tu server khi tai file: {}", response.status()));
    }

    // Luu vao thu muc tam
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("Takk_installer_update.exe");
    
    let mut file = std::fs::File::create(&file_path)
        .map_err(|e| format!("Loi khi tao file tam: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut response = response;

    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Loi khi tai khoi du lieu: {}", e))? {
        file.write_all(&chunk)
            .map_err(|e| format!("Loi khi ghi vao file tam: {}", e))?;
        downloaded += chunk.len() as u64;
        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
            use tauri::Emitter;
            let _ = app.emit("update-progress", percent);
        }
    }

    // Dong file de giai phong file handle tren Windows, tranh loi os error 32
    drop(file);

    crate::applog!("SUCCESS", "[Updater] Da tai xong ban cap nhat. Dang chay trinh cai dat va thoat ung dung...");

    // Chay file cai dat va thoat app
    Command::new(&file_path)
        .spawn()
        .map_err(|e| format!("Loi khi chay trinh cai dat: {}", e))?;

    std::process::exit(0);
}

