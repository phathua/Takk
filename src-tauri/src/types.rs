use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc, NaiveDate};

// Cấu hình file đơn lẻ
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileConfig {
    pub path: PathBuf,
    pub sheet_name: Option<String>,
    pub brand: String,
    pub provider: String,
    pub headers: Vec<String>,
    pub mapping: HashMap<String, String>, // Tên trường -> Cột nguồn
    pub normalize_basic: bool,            // Loai bo "-", " ", viet HOA
    pub normalize_special: bool,          // Them ky tu dau/cuoi
    pub normalize_position: SuffixPosition, // Them vao dau hay cuoi
    pub normalize_suffix: String,         // Ky tu them vao (vd: "M")
    pub generate_cost: bool,              // Tao gia von tu % chiet khau
    pub cost_discount_percent: f64,       // % chiet khau (vd: 30 -> cost = retail * 0.7)
    pub created_at: String,               // Ngay tao bang gia
    #[serde(default)]
    pub not_found: bool,                  // Danh dau neu khong tim thay file (chi dung cho bgx)
    #[serde(default)]
    pub file_hash: Option<String>,
    #[serde(default)]
    pub original_path: Option<PathBuf>,
}

// Cấu hình file dự án khi lưu trữ raw binary
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectFileConfig {
    pub config: FileConfig,
    pub file_name: String,
    pub extension: String,
    pub raw_data: Vec<u8>, // Du lieu nhi phan cua file goc (.xlsx, .csv, .xls)
    #[serde(default)]
    pub original_path: Option<PathBuf>,
}

// Dinh dang luu tru chinh cua du an .bg
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectFile {
    pub version: u32,
    pub app_version: String,
    pub created_at: String,
    pub files: Vec<ProjectFileConfig>,
    #[serde(default)]
    pub export_format: Option<String>,
    #[serde(default)]
    pub app_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadProjectResponse {
    pub files: Vec<FileConfig>,
    pub export_format: Option<String>,
    pub app_mode: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SuffixPosition {
    Prefix,
    Suffix,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PriceRow {
    pub product_code: String,
    pub alt_code: Option<String>,
    pub name: String,
    pub brand: String,
    pub provider: String,
    pub cost_price: f64,
    pub retail_price: Option<f64>,
    pub note: Option<String>,
    pub model: Option<String>,
    pub color_code: Option<String>,
    pub created_at: Option<NaiveDate>,
    pub updated_at: Option<DateTime<Utc>>,
    pub fingerprint: String, // Chuoi dinh danh duy nhat duoc tinh toan
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BrandProviderMapping {
    pub brand: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingRules {
    pub product_code: String,
    pub alt_code: String,
    pub name: String,
    pub cost_price: String,
    pub retail_price: String,
    pub model: String,
    pub color_code: String,
    pub note: String,
    pub ignore: String,
}

impl Default for MappingRules {
    fn default() -> Self {
        Self {
            product_code: "mã hàng, mã phụ tùng, mã vt, part number, mã số, mã, code, product_code, ma hang, ma phu tung, ma vt, ma so, ma".into(),
            alt_code: "mã cũ, old code, mã thay thế, alt_code, ma cu, ma thay the".into(),
            name: "tên hàng hóa, tên hàng, tên phụ tùng, description, tên, name, ten hang hoa, ten hang, ten phu tung, ten".into(),
            cost_price: "giá bán buôn, giá sỉ, giá nhập, giá vốn, buôn, sỉ, vốn, cost_price, gia ban buon, gia si, gia nhap, gia von, buon, si, von".into(),
            retail_price: "giá bán lẻ sau thuế, giá lẻ, giá bán lẻ, giá bán, vat, lẻ, retail, giá niêm yết, retail_price, gia ban le sau thue, gia le, gia ban le, gia ban, le, gia niem yet".into(),
            model: "model, đời xe, xe, doi xe".into(),
            color_code: "màu, color, color_code, mau".into(),
            note: "ghi chú, note, ghi chu".into(),
            ignore: "stt, số thứ tự, so thu tu".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Error,
    Success,
    Warning,
}

// Cap nhat trang thai xu li gui ve frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProcessingUpdate {
    Progress { percent: f32, task: String },
    Log { level: LogLevel, message: String },
    FileAdded(FileConfig),
    AddFilesFinished,
}

// Struct chua thong tin file goi y quet duoc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: u64, // Epoch milliseconds
}
