import { invoke } from '@tauri-apps/api/core';
import { open, save, ask } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';

class AppState {
  // Trang thai goc
  files = $state([]);
  selectedFileIdx = $state(-1);
  isProcessing = $state(false);
  isAddingFiles = $state(false);
  statusText = $state("");
  progressPercent = $state(0);
  logs = $state([]);
  toasts = $state([]);
  currentProjectPath = $state(null);
  theme = $state('auto');
  lastSavedState = $state("");

  // Quan ly da tab / da du an
  openProjects = $state([]);
  activeProjectIdx = $state(-1);

  // Lịch sử file gần đây
  recentFiles = $state([]);
  suggestedFiles = $state([]);

  // Confirm Modal state toàn cục
  confirmDialog = $state({
    show: false,
    title: "Xác nhận",
    message: "",
    confirmText: "Đồng ý",
    cancelText: "Hủy",
    kind: "info",
    buttons: null,
    resolve: null
  });

  confirm(options = {}) {
    return new Promise((resolve) => {
      this.confirmDialog = {
        show: true,
        title: options.title || "Xác nhận",
        message: options.message || "",
        confirmText: options.confirmText || "Đồng ý",
        cancelText: options.cancelText || "Hủy",
        kind: options.kind || "info",
        buttons: options.buttons || null,
        resolve: (result) => {
          this.confirmDialog.show = false;
          resolve(result);
        }
      };
    });
  }

  // Lịch sử undo/redo cho files & mapping
  historyUndoStack = $state([]);
  historyRedoStack = $state([]);

  getHistorySnapshot() {
    return {
      files: JSON.parse(JSON.stringify(this.files)),
      selectedFileIdx: this.selectedFileIdx
    };
  }

  restoreHistorySnapshot(snapshot) {
    this.files = snapshot.files;
    this.selectedFileIdx = snapshot.selectedFileIdx;
  }

  saveHistoryState() {
    if (this.historyUndoStack.length >= 50) {
      this.historyUndoStack.shift();
    }
    this.historyUndoStack.push(this.getHistorySnapshot());
    this.historyRedoStack = [];
  }

  undo() {
    if (this.historyUndoStack.length === 0) return;
    this.historyRedoStack.push(this.getHistorySnapshot());
    const prev = this.historyUndoStack.pop();
    this.restoreHistorySnapshot(prev);
    this.addLog("Info", "Đã hoàn tác (Undo).", false);
  }

  redo() {
    if (this.historyRedoStack.length === 0) return;
    this.historyUndoStack.push(this.getHistorySnapshot());
    const next = this.historyRedoStack.pop();
    this.restoreHistorySnapshot(next);
    this.addLog("Info", "Đã làm lại (Redo).", false);
  }

  updateMappingField(file, field, value) {
    this.saveHistoryState();
    file.mapping[field] = value;
  }

  constructor() {
    try {
      const savedTheme = localStorage.getItem('takk-theme');
      if (savedTheme) {
        this.theme = savedTheme;
      }
    } catch (e) {
      console.error("Lỗi khi đọc theme từ localStorage:", e);
    }
    
    // Tải lịch sử file gần đây từ localStorage
    this.loadRecentFiles();
    this.scanSuggestions();

    // Khoi tao tab mac dinh
    this.lastSavedState = this.serializeCurrentState();
    this.openProjects = [this.createNewProjectObject(null, "Dự án mới")];
    this.activeProjectIdx = 0;

    // Lang nghe su kien mo file tu backend
    try {
      listen('open-project-tab', (event) => {
        const filePath = event.payload;
        if (filePath) {
          this.loadProjectByPath(filePath);
        }
      });
    } catch (e) {
      console.error("Lỗi khi đăng ký sự kiện open-project-tab:", e);
    }
  }

  serializeCurrentState() {
    return JSON.stringify(this.files.map(f => ({
      brand: f.brand,
      provider: f.provider,
      created_at: f.created_at,
      normalize_basic: f.normalize_basic,
      normalize_special: f.normalize_special,
      normalize_position: f.normalize_position,
      normalize_suffix: f.normalize_suffix,
      generate_cost: f.generate_cost,
      cost_discount_percent: f.cost_discount_percent,
      mapping: f.mapping
    })));
  }

  get isProjectDirty() {
    return this.serializeCurrentState() !== this.lastSavedState;
  }

  isProjectTabDirty(idx) {
    if (idx === this.activeProjectIdx) {
      return this.serializeCurrentState() !== this.lastSavedState;
    }
    const proj = this.openProjects[idx];
    if (!proj) return false;
    const currentState = JSON.stringify(proj.files.map(f => ({
      brand: f.brand,
      provider: f.provider,
      created_at: f.created_at,
      normalize_basic: f.normalize_basic,
      normalize_special: f.normalize_special,
      normalize_position: f.normalize_position,
      normalize_suffix: f.normalize_suffix,
      generate_cost: f.generate_cost,
      cost_discount_percent: f.cost_discount_percent,
      mapping: f.mapping
    })));
    return currentState !== proj.lastSavedState;
  }

  get hasAnyDirtyProjects() {
    // Đồng bộ state hiện tại vào active slot trước khi kiểm tra
    this.saveCurrentStateToActiveSlot();
    return this.openProjects.some((_, i) => this.isProjectTabDirty(i));
  }

  setTheme(newTheme) {
    this.theme = newTheme;
    try {
      localStorage.setItem('takk-theme', newTheme);
    } catch (e) {
      console.error("Lỗi khi lưu theme vào localStorage:", e);
    }
  }

  createNewProjectObject(path = null, name = "Dự án mới", files = [], selectedFileIdx = -1, exportFormat = "xlsx") {
    return {
      id: crypto.randomUUID(),
      path,
      name,
      files,
      selectedFileIdx,
      exportFormat,
      lastSavedState: JSON.stringify(files.map(f => ({
        brand: f.brand,
        provider: f.provider,
        created_at: f.created_at,
        normalize_basic: f.normalize_basic,
        normalize_special: f.normalize_special,
        normalize_position: f.normalize_position,
        normalize_suffix: f.normalize_suffix,
        generate_cost: f.generate_cost,
        cost_discount_percent: f.cost_discount_percent,
        mapping: f.mapping
      })))
    };
  }

  saveCurrentStateToActiveSlot() {
    if (this.activeProjectIdx >= 0 && this.activeProjectIdx < this.openProjects.length) {
      this.openProjects[this.activeProjectIdx].path = this.currentProjectPath;
      this.openProjects[this.activeProjectIdx].name = this.currentProjectPath 
        ? this.currentProjectPath.split(/[\\/]/).pop() 
        : "Dự án mới";
      this.openProjects[this.activeProjectIdx].files = this.files;
      this.openProjects[this.activeProjectIdx].selectedFileIdx = this.selectedFileIdx;
      this.openProjects[this.activeProjectIdx].exportFormat = this.exportFormat;
      this.openProjects[this.activeProjectIdx].lastSavedState = this.lastSavedState;
    }
  }

  loadActiveSlotState() {
    if (this.activeProjectIdx >= 0 && this.activeProjectIdx < this.openProjects.length) {
      const proj = this.openProjects[this.activeProjectIdx];
      this.currentProjectPath = proj.path;
      this.files = proj.files;
      this.selectedFileIdx = proj.selectedFileIdx;
      this.exportFormat = proj.exportFormat;
      this.lastSavedState = proj.lastSavedState;
    }
  }

  switchProjectTab(idx) {
    if (idx === this.activeProjectIdx) return;
    this.saveCurrentStateToActiveSlot();
    this.activeProjectIdx = idx;
    this.loadActiveSlotState();
  }

  addNewProjectTab() {
    this.saveCurrentStateToActiveSlot();
    const newProj = this.createNewProjectObject(null, "Dự án mới");
    this.openProjects = [...this.openProjects, newProj];
    this.activeProjectIdx = this.openProjects.length - 1;
    this.loadActiveSlotState();
    this.scanSuggestions();
  }

  async closeProjectTab(idx) {
    // Lưu tạm state hiện tại trước khi check để đảm bảo so sánh chính xác nhất cho tab active
    if (idx === this.activeProjectIdx) {
      this.saveCurrentStateToActiveSlot();
    }

    const proj = this.openProjects[idx];
    if (proj) {
      // Kiểm tra xem dự án ở tab này có bị thay đổi chưa lưu hay không
      const currentState = JSON.stringify(proj.files.map(f => ({
        brand: f.brand,
        provider: f.provider,
        created_at: f.created_at,
        normalize_basic: f.normalize_basic,
        normalize_special: f.normalize_special,
        normalize_position: f.normalize_position,
        normalize_suffix: f.normalize_suffix,
        generate_cost: f.generate_cost,
        cost_discount_percent: f.cost_discount_percent,
        mapping: f.mapping
      })));
      const isDirty = currentState !== proj.lastSavedState;

      if (isDirty) {
        const confirmClose = await this.confirm({
          title: "Lưu thay đổi?",
          message: `Dự án "${proj.name}" có thay đổi chưa lưu. Bạn có muốn lưu các thay đổi trước khi đóng không?`,
          kind: "warning",
          buttons: [
            { text: "Lưu", value: "save", class: "bg-[var(--accent)] hover:bg-[var(--accent)]/90 text-white" },
            { text: "Lưu như...", value: "save_as", class: "bg-slate-500/10 hover:bg-slate-500/20 text-[var(--text)] border border-[var(--border)]" },
            { text: "Hủy bỏ (Không lưu)", value: "discard", class: "bg-rose-500/10 hover:bg-rose-500/20 text-rose-600 dark:text-rose-400" }
          ]
        });

        if (confirmClose === false || confirmClose === null) {
          // Bấm nút X hoặc bấm ngoài (Hủy bỏ hành động đóng tab)
          return;
        }

        if (confirmClose === "save" || confirmClose === "save_as") {
          // Chuyển sang tab này để tiến hành lưu
          if (idx !== this.activeProjectIdx) {
            this.switchProjectTab(idx);
          }
          await this.handleSaveProject(confirmClose === "save_as");
          
          // Kiểm tra xem đã lưu thành công chưa (đề phòng người dùng hủy hộp thoại lưu file)
          const reCheckProj = this.openProjects[idx];
          if (reCheckProj) {
            const reCurrentState = JSON.stringify(reCheckProj.files.map(f => ({
              brand: f.brand,
              provider: f.provider,
              created_at: f.created_at,
              normalize_basic: f.normalize_basic,
              normalize_special: f.normalize_special,
              normalize_position: f.normalize_position,
              normalize_suffix: f.normalize_suffix,
              generate_cost: f.generate_cost,
              cost_discount_percent: f.cost_discount_percent,
              mapping: f.mapping
            })));
            const stillDirty = reCurrentState !== reCheckProj.lastSavedState;
            if (stillDirty) {
              return; // Người dùng đã hủy hộp thoại lưu
            }
          }
        }
      }
    }

    if (this.openProjects.length <= 1) {
      this.openProjects = [this.createNewProjectObject(null, "Dự án mới")];
      this.activeProjectIdx = 0;
      this.loadActiveSlotState();
      this.scanSuggestions();
      return;
    }

    const isClosingActive = (idx === this.activeProjectIdx);
    
    this.openProjects = this.openProjects.filter((_, i) => i !== idx);
    
    if (isClosingActive) {
      this.activeProjectIdx = Math.max(0, idx - 1);
    } else if (this.activeProjectIdx > idx) {
      this.activeProjectIdx -= 1;
    }
    this.loadActiveSlotState();
  }

  async loadProjectByPath(path) {
    const existingIdx = this.openProjects.findIndex(p => p.path === path);
    if (existingIdx !== -1) {
      this.switchProjectTab(existingIdx);
      this.addLog("Info", `Đã chuyển sang tab dự án: ${path}`);
      return;
    }

    try {
      const res = await invoke('load_project', { path });
      const files = res.files.map(f => ({ ...f, id: f.id || crypto.randomUUID() }));
      
      // Canh bao neu co file bi mat (notFound)
      const missingFiles = files.filter(f => f.not_found);
      if (missingFiles.length > 0) {
        const fullFileNames = missingFiles.map(f => f.path.split(/[\\/]/).pop()).join(", ");
        let toastFileNames = missingFiles.slice(0, 2).map(f => f.path.split(/[\\/]/).pop()).join(", ");
        if (missingFiles.length > 2) {
          toastFileNames += `, ... và ${missingFiles.length - 2} tệp khác`;
        }
        this.addLog(
          "Warning",
          `Không tìm thấy ${missingFiles.length} tệp trong dự án: ${fullFileNames}. Vui lòng kiểm tra lại đường dẫn!`,
          true,
          `Không tìm thấy ${missingFiles.length} tệp trong dự án: ${toastFileNames}. Vui lòng kiểm tra lại đường dẫn!`
        );
      }

      const fileName = path.split(/[\\/]/).pop();

      if (!this.currentProjectPath && this.files.length === 0) {
        this.currentProjectPath = path;
        this.files = files;
        if (res.export_format) {
          this.exportFormat = res.export_format;
        }
        if (this.files.length > 0) this.selectedFileIdx = 0;
        this.lastSavedState = this.serializeCurrentState();
        
        this.openProjects[this.activeProjectIdx].path = path;
        this.openProjects[this.activeProjectIdx].name = fileName;
        this.openProjects[this.activeProjectIdx].files = files;
        this.openProjects[this.activeProjectIdx].selectedFileIdx = this.selectedFileIdx;
        this.openProjects[this.activeProjectIdx].exportFormat = this.exportFormat;
        this.openProjects[this.activeProjectIdx].lastSavedState = this.lastSavedState;
      } else {
        this.saveCurrentStateToActiveSlot();
        
        const newProj = this.createNewProjectObject(path, fileName, files, files.length > 0 ? 0 : -1, res.export_format || "xlsx");
        this.openProjects = [...this.openProjects, newProj];
        this.activeProjectIdx = this.openProjects.length - 1;
        
        this.loadActiveSlotState();
      }
      this.addLog("Success", `Đã mở thành công dự án: ${path}`);
      this.addRecentFile(path, 'project');
    } catch (e) {
      const errStr = String(e);
      let viMsg = "Không thể mở dự án. Vui lòng thử lại!";
      if (errStr.includes("cannot find the file") || errStr.includes("os error 2") || errStr.includes("No such file or directory")) {
        viMsg = "Không tìm thấy tệp tin dự án được chỉ định. Vui lòng kiểm tra lại đường dẫn!";
      } else if (errStr.includes("Permission denied") || errStr.includes("os error 5")) {
        viMsg = "Không có quyền truy cập hoặc ghi vào tệp tin dự án này.";
      }
      this.addLog("Error", `Lỗi mở dự án từ đường dẫn: ${e}`, true, viMsg);
    }
  }

  // Trang thai ConsoleBar
  logContainer = $state(null);
  isLogsExpanded = $state(false);
  logsHeight = $state(160);
  isResizing = $state(false);
  searchQuery = $state("");
  selectedLevel = $state("ALL");
  expandedLogs = $state(new Set());

  // Input & Mappings
  brandInputFocus = $state(false);
  providerInputFocus = $state(false);
  exportFormat = $state("xlsx");
  outputPath = $state("");

  logLevels = [
    { value: "ALL", label: "TẤT CẢ" },
    { value: "DEBUG", label: "DEBUG" },
    { value: "INFO", label: "INFO" },
    { value: "SUCCESS", label: "SUCCESS" },
    { value: "WARN", label: "WARN" },
    { value: "ERROR", label: "ERROR" }
  ];

  isLevelDropdownOpen = $state(false);
  openUpwardLevel = $state(false);

  mappingRules = {
    product_code: "mã hàng, mã phụ tùng, mã vt, part number, mã số, mã, code, product_code, ma hang, ma phu tung, ma vt, ma so, ma",
    alt_code: "mã cũ, old code, mã thay thế, alt_code, ma cu, ma thay the",
    name: "tên hàng hóa, tên hàng, tên phụ tùng, description, tên, name, ten hang hoa, ten hang, ten phu tung, ten",
    cost_price: "giá bán buôn, giá sỉ, giá nhập, giá vốn, buôn, sỉ, vốn, cost_price, gia ban buon, gia si, gia nhap, gia von, buon, si, von",
    retail_price: "giá bán lẻ sau thuế, giá lẻ, giá bán lẻ, giá bán, vat, lẻ, retail, giá niêm yết, retail_price, gia ban le sau thue, gia le, gia ban le, gia ban, le, gia niem yet",
    model: "model, đời xe, xe, doi xe",
    color_code: "màu, color, color_code, mau",
    note: "ghi chú, note, ghi chu",
    ignore: "stt, số thứ tự, so thu tu"
  };

  brandMappings = $state([
    { brand: "HONDA", provider: "TRUNG HÙNG" },
    { brand: "HONDA", provider: "MINH ĐÔNG" },
    { brand: "YAMAHA", provider: "KHIÊM" },
    { brand: "SYM", provider: "NHƠN" },
    { brand: "KYMCO", provider: "KWANG YANG" },
    { brand: "SUZUKI", provider: "ĐỨC VINH" },
    { brand: "CASTROL", provider: "HƯNG THỊNH" },
    { brand: "CHENGSHIN", provider: "NHÂN MỸ" },
    { brand: "MAXXIS", provider: "NHÂN MỸ" },
    { brand: "SE", provider: "MEI YING" },
    { brand: "INOUE", provider: "PHƯỚC LỢI" },
    { brand: "TECH", provider: "AN THÀNH" },
    { brand: "TMH", provider: "TUNG VIỆT" },
    { brand: "CASUMINA", provider: "LIÊN CƯỜNG" },
    { brand: "VILOBE", provider: "KHÁC" },
    { brand: "MOTUL", provider: "KHÁC" },
    { brand: "VELOIL", provider: "KHÁC" },
    { brand: "MEKONG", provider: "KHÁC" }
  ]);

  // Derived states
  get currentFile() {
    return this.selectedFileIdx !== -1 && this.files[this.selectedFileIdx] 
      ? this.files[this.selectedFileIdx] 
      : null;
  }

  get filteredLogs() {
    return this.logs.filter(log => {
      const matchesLevel = this.selectedLevel === "ALL" || log.level.toUpperCase() === this.selectedLevel;
      const matchesSearch = this.searchQuery.trim() === "" || 
        log.message.toLowerCase().includes(this.searchQuery.toLowerCase());
      return matchesLevel && matchesSearch;
    }).slice().reverse(); // Moi nhat o cuoi cho Console
  }

  get filteredBrands() {
    const file = this.currentFile;
    if (!file) return [];
    const matches = this.brandMappings.filter(m => 
      !file.brand || 
      m.brand.toLowerCase().includes(file.brand.toLowerCase())
    );
    const seen = new Set();
    return matches.filter(m => {
      const brandLower = m.brand.toLowerCase();
      if (seen.has(brandLower)) return false;
      seen.add(brandLower);
      return true;
    });
  }

  get filteredProviders() {
    const file = this.currentFile;
    return file ? this.brandMappings.filter(m => 
      !file.provider || 
      m.provider.toLowerCase().includes(file.provider.toLowerCase())
    ) : [];
  }

  // Toast
  showToast(type, text) {
    const id = Date.now() + Math.random();
    
    // Gioi han toi da 3 toast tren man hinh cung luc
    let newToasts = [...this.toasts, { id, type: type.toLowerCase(), text }];
    if (newToasts.length > 3) {
      newToasts.shift(); // Xoa bot cai cu nhat ngay lap tuc
    }
    this.toasts = newToasts;

    // Tinh toan thoi gian bien mat lech nhau (staggered delay)
    const index = this.toasts.findIndex(t => t.id === id);
    const delay = 4000 + (index * 1200); // So le nhau 1.2 giay

    setTimeout(() => {
      this.toasts = this.toasts.filter(t => t.id !== id);
    }, delay);

    return id;
  }

  removeToast(id) {
    if (!id) return;
    this.toasts = this.toasts.filter(t => t.id !== id);
    console.log("Toasts after remove:", this.toasts);
  }

  // Logs
  addLog(level, message, triggerToast = true, toastMessage = null) {
    this.logs = [{ time: new Date().toLocaleTimeString(), level, message }, ...this.logs];
    if (triggerToast) {
      return this.showToast(level, toastMessage || message);
    }
    return null;
  }

  toggleLevelDropdown(e) {
    e.stopPropagation();
    if (this.isLevelDropdownOpen) {
      this.isLevelDropdownOpen = false;
    } else {
      const rect = e.currentTarget.getBoundingClientRect();
      const spaceBelow = window.innerHeight - rect.bottom;
      this.openUpwardLevel = spaceBelow < 185;
      this.isLevelDropdownOpen = true;
    }
  }

  handleCopyAllLogs() {
    if (this.logs.length === 0) return;
    const text = this.logs.map(l => `[${l.time}] [${l.level.toUpperCase()}] ${l.message}`).join("\n");
    navigator.clipboard.writeText(text);
    this.showToast("Success", "Đã sao chép toàn bộ nhật ký!");
  }

  handleCopySingleLog(log) {
    const text = `[${log.time}] [${log.level.toUpperCase()}] ${log.message}`;
    navigator.clipboard.writeText(text);
    this.showToast("Success", "Đã sao chép dòng nhật ký!");
  }

  handleDownloadTxt() {
    if (this.logs.length === 0) return;
    const text = this.logs.map(l => `[${l.time}] [${l.level.toUpperCase()}] ${log.message}`).join("\n");
    const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `takk_logs_${new Date().toISOString().slice(0, 19).replace(/:/g, "-")}.txt`;
    a.click();
    URL.revokeObjectURL(url);
    this.showToast("Success", "Đã tải xuống file TXT!");
  }

  toggleExpand(log) {
    const key = `${log.time}_${log.message}`;
    if (this.expandedLogs.has(key)) {
      this.expandedLogs.delete(key);
    } else {
      this.expandedLogs.add(key);
    }
    this.expandedLogs = new Set(this.expandedLogs);
  }

  // Workflows
  async addFilesByPaths(paths) {
    try {
      if (paths && paths.length > 0) {
        this.isAddingFiles = true;
        this.progressPercent = 0;
        this.statusText = "Đang phân tích cấu hình tệp...";
        this.addLog("Info", `Đã chọn ${paths.length} tệp. Bắt đầu phân tích cấu hình...`);
        
        await invoke('add_files_async', {
          paths: paths,
          brandMappings: this.brandMappings,
          mappingRules: this.mappingRules
        });
      }
    } catch (e) {
      this.addLog("Error", `Lỗi thêm file: ${e}`);
      this.isAddingFiles = false;
    }
  }

  async handleAddFiles() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Excel/CSV',
          extensions: ['xlsx', 'xls', 'csv']
        }]
      });

      if (selected && selected.length > 0) {
        await this.addFilesByPaths(selected);
      }
    } catch (e) {
      this.addLog("Error", `Lỗi thêm file: ${e}`);
    }
  }

  handleReset() {
    this.files = [];
    this.selectedFileIdx = -1;
    this.currentProjectPath = null;
    this.logs = [];
    this.addLog("Info", "Đã dọn dẹp không gian làm việc.");
    this.lastSavedState = this.serializeCurrentState();
  }

  moveFile(idx, direction) {
    if (direction === 'up' && idx > 0) {
      this.saveHistoryState();
      const temp = this.files[idx];
      this.files[idx] = this.files[idx - 1];
      this.files[idx - 1] = temp;
      this.selectedFileIdx = idx - 1;
    } else if (direction === 'down' && idx < this.files.length - 1) {
      this.saveHistoryState();
      const temp = this.files[idx];
      this.files[idx] = this.files[idx + 1];
      this.files[idx + 1] = temp;
      this.selectedFileIdx = idx + 1;
    }
  }

  sortFilesByPriority() {
    if (this.files.length <= 1) return;
    
    this.saveHistoryState();

    const selectedFileId = this.selectedFileIdx !== -1 && this.files[this.selectedFileIdx] 
      ? this.files[this.selectedFileIdx].id 
      : null;

    const priorities = this.brandMappings;

    const sorted = [...this.files].sort((a, b) => {
      const posA = priorities.findIndex(mapping => 
        mapping.brand?.toLowerCase() === a.brand?.toLowerCase() && 
        mapping.provider?.toLowerCase() === a.provider?.toLowerCase()
      );
      const posB = priorities.findIndex(mapping => 
        mapping.brand?.toLowerCase() === b.brand?.toLowerCase() && 
        mapping.provider?.toLowerCase() === b.provider?.toLowerCase()
      );

      const idxA = posA === -1 ? Number.MAX_SAFE_INTEGER : posA;
      const idxB = posB === -1 ? Number.MAX_SAFE_INTEGER : posB;

      return idxA - idxB;
    });

    this.files = sorted;

    if (selectedFileId) {
      this.selectedFileIdx = this.files.findIndex(f => f.id === selectedFileId);
    }
    
    this.addLog("Success", "Đã sắp xếp danh sách file theo danh sách ưu tiên.");
  }

  removeFile(idx) {
    this.saveHistoryState();
    this.files = this.files.filter((_, i) => i !== idx);
    if (this.selectedFileIdx >= this.files.length) {
      this.selectedFileIdx = this.files.length - 1;
    }
  }

  async relinkFile(idx) {
    const file = this.files[idx];
    if (!file) return;

    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Excel/CSV',
          extensions: ['xlsx', 'xls', 'csv']
        }]
      });

      if (!selected) return;

      const newHash = await invoke('calculate_file_hash', { path: selected });

      if (file.file_hash && file.file_hash !== newHash) {
        const oldName = file.path.split(/[\\/]/).pop();
        const newName = selected.split(/[\\/]/).pop();
        const confirmReplace = await this.confirm({
          title: "Xác nhận thay thế tệp",
          message: `Tệp tin mới chọn có nội dung (mã băm) khác biệt so với tệp gốc của dự án:\n` +
                   `- Tệp gốc: ${oldName}\n` +
                   `- Tệp mới: ${newName}\n\n` +
                   `Bạn có chắc chắn muốn liên kết lại bằng tệp mới này không? Điều này có thể làm thay đổi kết quả gộp dữ liệu.`,
          confirmText: "Đồng ý thay thế",
          cancelText: "Hủy bỏ",
          kind: "warning"
        });
        if (!confirmReplace) return;
      }

      this.saveHistoryState();
      file.path = selected;
      file.not_found = false;
      file.file_hash = newHash;
      this.addLog("Success", `Đã liên kết lại tệp thành công: ${selected}`);
    } catch (e) {
      this.addLog("Error", `Lỗi liên kết lại tệp: ${e}`);
    }
  }

  removeInvalidFiles(invalidIds) {
    if (!invalidIds || invalidIds.length === 0) return;
    this.saveHistoryState();
    
    const selectedFileId = this.selectedFileIdx !== -1 && this.files[this.selectedFileIdx]
      ? this.files[this.selectedFileIdx].id
      : null;

    this.files = this.files.filter(f => !invalidIds.includes(f.id));

    if (selectedFileId && !invalidIds.includes(selectedFileId)) {
      this.selectedFileIdx = this.files.findIndex(f => f.id === selectedFileId);
    } else {
      this.selectedFileIdx = this.files.length > 0 ? 0 : -1;
    }
    
    this.addLog("Info", `Đã loại bỏ ${invalidIds.length} tệp cấu hình lỗi.`, false);
  }


  async handleSaveProject(saveAs = false) {
    try {
      let path = this.currentProjectPath;
      if (!path || saveAs) {
        const now = new Date();
        const yyyy = now.getFullYear();
        const mm = String(now.getMonth() + 1).padStart(2, '0');
        const dd = String(now.getDate()).padStart(2, '0');
        const hh = String(now.getHours()).padStart(2, '0');
        const min = String(now.getMinutes()).padStart(2, '0');
        const ss = String(now.getSeconds()).padStart(2, '0');
        const defaultName = `Takk_${yyyy}${mm}${dd}_${hh}${min}${ss}.bgx`;

        path = await save({
          defaultPath: defaultName,
          filters: [
            {
              name: 'Dự án Takk tham chiếu (.bgx)',
              extensions: ['bgx']
            },
            {
              name: 'Dự án Takk đóng gói (.bg)',
              extensions: ['bg']
            }
          ]
        });
      }

      if (path) {
        await invoke('save_project', { 
          files: this.files, 
          path,
          exportFormat: this.exportFormat,
          appMode: null
        });
        this.currentProjectPath = path;
        this.addLog("Success", `Đã lưu dự án vào: ${path}`);
        this.lastSavedState = this.serializeCurrentState();
        
        // Cap nhat thong tin tab hien tai sau khi luu
        if (this.activeProjectIdx >= 0 && this.activeProjectIdx < this.openProjects.length) {
          this.openProjects[this.activeProjectIdx].path = path;
          this.openProjects[this.activeProjectIdx].name = path.split(/[\\/]/).pop();
          this.openProjects[this.activeProjectIdx].lastSavedState = this.lastSavedState;
        }
        this.addRecentFile(path, 'project');
      }
    } catch (e) {
      const errStr = String(e);
      let viMsg = "Không thể lưu dự án. Vui lòng thử lại!";
      if (errStr.includes("cannot find the file") || errStr.includes("os error 2") || errStr.includes("No such file or directory")) {
        viMsg = "Không tìm thấy thư mục lưu trữ hoặc tệp tin.";
      } else if (errStr.includes("Permission denied") || errStr.includes("os error 5")) {
        viMsg = "Không có quyền ghi tệp tin vào thư mục đã chọn.";
      }
      this.addLog("Error", `Lỗi lưu dự án: ${e}`, true, viMsg);
    }
  }

  async handleOpenProject() {
    try {
      const path = await open({
        multiple: false,
        filters: [
          {
            name: 'Mọi dự án Takk (.bgx, .bg)',
            extensions: ['bgx', 'bg']
          },
          {
            name: 'Dự án Takk tham chiếu (.bgx)',
            extensions: ['bgx']
          },
          {
            name: 'Dự án Takk đóng gói (.bg)',
            extensions: ['bg']
          }
        ]
      });

      if (path) {
        await this.loadProjectByPath(path);
      }
    } catch (e) {
      this.addLog("Error", `Lỗi mở dự án: ${e}`);
    }
  }

  async handleSelectOutputPath() {
    try {
      const isXlsx = this.exportFormat === 'xlsx';
      const now = new Date();
      const yyyy = now.getFullYear();
      const mm = String(now.getMonth() + 1).padStart(2, '0');
      const dd = String(now.getDate()).padStart(2, '0');
      const hh = String(now.getHours()).padStart(2, '0');
      const min = String(now.getMinutes()).padStart(2, '0');
      const ss = String(now.getSeconds()).padStart(2, '0');
      const defaultName = `bang_gia_gop_takk_${yyyy}${mm}${dd}_${hh}${min}${ss}.${this.exportFormat}`;

      const path = await save({
        defaultPath: defaultName,
        filters: [{
          name: isXlsx ? 'Excel (.xlsx)' : 'Bảng giá gộp (.csv)',
          extensions: [this.exportFormat]
        }]
      });
      if (path) {
        this.outputPath = path;
      }
    } catch (e) {
      this.addLog("Error", `Lỗi chọn đường dẫn: ${e}`);
    }
  }

  async handleProcessAndExport() {
    if (this.files.length === 0) {
      this.addLog("Warning", "Vui lòng thêm ít nhất một file cấu hình.");
      return;
    }

    for (const f of this.files) {
      if (!f.brand.trim() || !f.provider.trim()) {
        this.addLog("Warning", `Tệp ${f.path.split(/[\\/]/).pop()} thiếu thông tin Hãng hoặc Nhà cung cấp.`);
        return;
      }
      if (!f.mapping.product_code || !f.mapping.name || !f.mapping.retail_price) {
        this.addLog("Warning", `Tệp ${f.path.split(/[\\/]/).pop()} chưa được ánh xạ đầy đủ các cột bắt buộc.`);
        return;
      }
    }

    // Hiển thị hộp thoại chọn đường dẫn xuất file excel/csv trước khi tiến hành
    let targetPath = "";
    try {
      const isXlsx = this.exportFormat === 'xlsx';
      const now = new Date();
      const yyyy = now.getFullYear();
      const mm = String(now.getMonth() + 1).padStart(2, '0');
      const dd = String(now.getDate()).padStart(2, '0');
      const hh = String(now.getHours()).padStart(2, '0');
      const min = String(now.getMinutes()).padStart(2, '0');
      const ss = String(now.getSeconds()).padStart(2, '0');
      const defaultName = `bang_gia_gop_takk_${yyyy}${mm}${dd}_${hh}${min}${ss}.${this.exportFormat}`;

      const selectedPath = await save({
        defaultPath: defaultName,
        filters: [{
          name: isXlsx ? 'Excel (.xlsx)' : 'Bảng giá gộp (.csv)',
          extensions: [this.exportFormat]
        }]
      });

      if (!selectedPath) {
        this.addLog("Info", "Đã hủy xuất tệp.");
        return;
      }
      targetPath = selectedPath;
      this.outputPath = selectedPath;
    } catch (e) {
      this.addLog("Error", `Lỗi chọn đường dẫn: ${e}`);
      return;
    }

    this.isProcessing = true;
    try {
      const result = await invoke('process_and_export', {
        files: this.files,
        exportFormat: this.exportFormat,
        outputPath: targetPath
      });
      this.addLog("Success", result);

      // Hỏi người dùng có muốn lưu dự án lại không (nếu có thay đổi cấu hình dự án chưa được lưu)
      if (this.isProjectDirty) {
        setTimeout(async () => {
          try {
            const shouldSave = await this.confirm({
              title: "Lưu cấu hình dự án",
              message: "Xuất file thành công! Bạn có muốn lưu lại cấu hình dự án này không?",
              confirmText: "Lưu dự án",
              cancelText: "Không",
              kind: "info"
            });
            if (shouldSave) {
              await this.handleSaveProject(false);
            }
          } catch (err) {
            console.error("Lỗi khi hỏi lưu dự án:", err);
          }
        }, 300);
      }
    } catch (e) {
      this.addLog("Error", `Lỗi khi xử lý gộp file: ${e}`);
    } finally {
      this.isProcessing = false;
    }
  }

  handleResizeMouseDown(e) {
    e.preventDefault();
    this.isResizing = true;
    this.isLogsExpanded = true;
    const startY = e.clientY;
    const startHeight = this.logsHeight;

    const handleMouseMove = (moveEvent) => {
      if (!this.isResizing) return;
      const deltaY = startY - moveEvent.clientY;
      this.logsHeight = Math.max(80, Math.min(window.innerHeight * 0.8, startHeight + deltaY));
    };

    const handleMouseUp = () => {
      this.isResizing = false;
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  }

  async loadRecentFiles() {
    try {
      const data = localStorage.getItem('takk-recent-files');
      if (data) {
        this.recentFiles = JSON.parse(data);

        // Quét và cập nhật dung lượng file từ backend
        const paths = this.recentFiles.map(item => item.path);
        if (paths.length > 0) {
          const sizesMap = await invoke('get_files_metadata', { paths });
          this.recentFiles = this.recentFiles.map(item => {
            if (sizesMap[item.path] !== undefined) {
              return { ...item, size: sizesMap[item.path] };
            }
            return item;
          });
        }
      }
    } catch (e) {
      console.error("Lỗi khi tải lịch sử file gần đây:", e);
    }
  }

  async scanSuggestions() {
    this.addLog("Info", "Đang quét tìm các tệp dự án đề xuất (.bg, .bgx)...", false);
    try {
      const suggestions = await invoke('scan_suggested_projects');
      this.suggestedFiles = suggestions.map(s => ({
        path: s.path,
        name: s.name,
        type: 'project',
        timestamp: s.modified,
        size: s.size,
        isSuggestion: true
      }));
      this.addLog("Success", `Đã quét xong đề xuất! Tìm thấy ${this.suggestedFiles.length} tệp dự án.`, false);
    } catch (e) {
      console.error("Lỗi khi quét file đề xuất:", e);
      this.addLog("Error", `Lỗi khi quét tệp đề xuất: ${e}`, false);
    }
  }

  get displayRecentAndSuggestions() {
    const combined = [...this.recentFiles];
    for (const sug of this.suggestedFiles) {
      if (!combined.some(item => item.path === sug.path)) {
        combined.push(sug);
      }
    }
    // Sắp xếp theo timestamp mới nhất lên đầu
    return combined.sort((a, b) => b.timestamp - a.timestamp);
  }

  saveRecentFiles() {
    try {
      // Khi lưu vào localStorage, loại bỏ trường size để tránh lưu thừa thãi
      const persistList = this.recentFiles.map(({ size, ...rest }) => rest);
      localStorage.setItem('takk-recent-files', JSON.stringify(persistList));
    } catch (e) {
      console.error("Lỗi khi lưu lịch sử file gần đây:", e);
    }
  }

  async addRecentFile(filePath, type = 'project') {
    if (!filePath) return;
    const name = filePath.split(/[\\/]/).pop();
    const now = Date.now();

    let size = undefined;
    try {
      const sizesMap = await invoke('get_files_metadata', { paths: [filePath] });
      if (sizesMap[filePath] !== undefined) {
        size = sizesMap[filePath];
      }
    } catch (e) {
      console.error("Lỗi lấy dung lượng file:", e);
    }

    // Tìm và loại bỏ nếu đã tồn tại để đưa lên đầu
    let list = this.recentFiles.filter(item => item.path !== filePath);

    list.unshift({
      path: filePath,
      name,
      type, // 'project' (.bgx, .bg), 'excel' (.xlsx, .xls), 'csv' (.csv)
      timestamp: now,
      size
    });

    // Giới hạn tối đa 20 file gần đây
    if (list.length > 20) {
      list = list.slice(0, 20);
    }

    this.recentFiles = list;
    this.saveRecentFiles();
  }

  clearRecentFiles() {
    this.recentFiles = [];
    this.saveRecentFiles();
  }
}

export const appState = new AppState();
