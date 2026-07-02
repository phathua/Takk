import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 1. Lấy số lượng commit từ Git
let commitCount = 0;
try {
  commitCount = parseInt(execSync('git rev-list --count HEAD', { stdio: ['pipe', 'pipe', 'ignore'] }).toString().trim(), 10);
} catch (error) {
  console.error('Không thể đọc số lượng commit từ Git:', error.message);
  process.exit(1);
}

// 2. Tính toán phiên bản theo công thức X.Y.Z
const major = Math.floor(commitCount / 100);
const minor = Math.floor((commitCount % 100) / 10);
const patch = commitCount % 10;
const newVersion = `${major}.${minor}.${patch}`;

console.log(`[Version Bumper] Số commit: ${commitCount} -> Phiên bản tính toán: ${newVersion}`);

const rootDir = path.resolve(__dirname, '..');

// 3. Cập nhật package.json
const packageJsonPath = path.join(rootDir, 'package.json');
if (fs.existsSync(packageJsonPath)) {
  try {
    const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    if (pkg.version !== newVersion) {
      pkg.version = newVersion;
      fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2) + '\n');
      console.log(`[Version Bumper] Đã cập nhật package.json sang phiên bản: ${newVersion}`);
    } else {
      console.log(`[Version Bumper] Phiên bản trong package.json đã khớp: ${newVersion}`);
    }
  } catch (err) {
    console.error('Lỗi khi cập nhật package.json:', err.message);
  }
}

// 4. Cập nhật tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  try {
    const conf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
    if (conf.version !== newVersion) {
      conf.version = newVersion;
      fs.writeFileSync(tauriConfPath, JSON.stringify(conf, null, 2) + '\n');
      console.log(`[Version Bumper] Đã cập nhật tauri.conf.json sang phiên bản: ${newVersion}`);
    } else {
      console.log(`[Version Bumper] Phiên bản trong tauri.conf.json đã khớp: ${newVersion}`);
    }
  } catch (err) {
    console.error('Lỗi khi cập nhật tauri.conf.json:', err.message);
  }
}

// 5. Cập nhật Cargo.toml
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
if (fs.existsSync(cargoTomlPath)) {
  try {
    let cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');
    const versionRegex = /^version\s*=\s*".*"/m;
    if (versionRegex.test(cargoContent)) {
      cargoContent = cargoContent.replace(versionRegex, `version = "${newVersion}"`);
      fs.writeFileSync(cargoTomlPath, cargoContent);
      console.log(`[Version Bumper] Đã cập nhật Cargo.toml sang phiên bản: ${newVersion}`);
    } else {
      console.warn('[Version Bumper] Không tìm thấy trường version trong Cargo.toml để thay thế.');
    }
  } catch (err) {
    console.error('Lỗi khi cập nhật Cargo.toml:', err.message);
  }
}
