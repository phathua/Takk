<script>
  import { onMount } from 'svelte';
  import { scale, fade } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { Sliders, RefreshCw, Play, FileUp, Plus, X, Eye } from 'lucide-svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  
  // Import global state
  import { appState } from './lib/state.svelte.js';
  import ConfirmModal from './components/ConfirmModal.svelte';

  let isGlobalDragOver = $state(false);
  let draggedTabIdx = $state(null);

  function handleDragStart(e, idx) {
    draggedTabIdx = idx;
    e.dataTransfer.effectAllowed = 'move';
  }

  function handleDragOver(e, idx) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }

  function handleDrop(e, targetIdx) {
    e.preventDefault();
    if (draggedTabIdx !== null && draggedTabIdx !== targetIdx) {
      const projects = [...appState.openProjects];
      const activeProj = projects[appState.activeProjectIdx];
      
      const [item] = projects.splice(draggedTabIdx, 1);
      projects.splice(targetIdx, 0, item);
      
      appState.openProjects = projects;
      
      const newActiveIdx = projects.indexOf(activeProj);
      if (newActiveIdx !== -1) {
        appState.activeProjectIdx = newActiveIdx;
      }
    }
    draggedTabIdx = null;
  }

  // Import components
  import SidebarLeft from './components/SidebarLeft.svelte';
  import SidebarMiddle from './components/SidebarMiddle.svelte';
  import ConfigGeneralInfo from './components/ConfigGeneralInfo.svelte';
  import ConfigMapping from './components/ConfigMapping.svelte';
  import ConfigNormalization from './components/ConfigNormalization.svelte';
  import ConsoleBar from './components/ConsoleBar.svelte';
  import Toast from './components/ui/Toast.svelte';
  import Select from './components/ui/Select.svelte';
  import CsvIcon from './components/icons/CsvIcon.svelte';
  import ExcelIcon from './components/icons/ExcelIcon.svelte';
  import PreviewModal from './components/PreviewModal.svelte';

  const formatFileSize = (bytes) => {
    if (bytes === undefined || bytes === null) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  // Theo dõi và tự động cập nhật theme (dark, light, auto)
  $effect(() => {
    const currentTheme = appState.theme; // Đọc trực tiếp ở đây để Svelte 5 theo dõi phản hồi
    
    const applyTheme = () => {
      let isDark = false;
      if (currentTheme === 'dark') {
        isDark = true;
      } else if (currentTheme === 'light') {
        isDark = false;
      } else {
        // Chế độ tự động theo hệ thống
        isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      }

      if (isDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    };

    applyTheme();

    // Lắng nghe thay đổi theme hệ thống nếu chọn auto
    if (currentTheme === 'auto') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const listener = (e) => {
        if (appState.theme === 'auto') {
          if (e.matches) {
            document.documentElement.classList.add('dark');
          } else {
            document.documentElement.classList.remove('dark');
          }
        }
      };
      mediaQuery.addEventListener('change', listener);
      return () => {
        mediaQuery.removeEventListener('change', listener);
      };
    }
  });

  let showPreviewModal = $state(false);
  let previewRows = $state([]);
  let isGeneratingPreview = $state(false);
  let previewError = $state(null);

  const formatCurrency = (val) => {
    if (val === undefined || val === null || isNaN(val)) return '-';
    return new Intl.NumberFormat('vi-VN', { style: 'currency', currency: 'VND' }).format(val);
  };

  const handleOpenPreview = async () => {
    if (appState.files.length === 0) {
      appState.addLog("Warning", "Vui lòng thêm ít nhất một file cấu hình.");
      return;
    }

    for (const f of appState.files) {
      if (!f.brand.trim() || !f.provider.trim()) {
        appState.addLog("Warning", `Tệp ${f.path.split(/[\\/]/).pop()} thiếu thông tin Hãng hoặc Nhà cung cấp.`);
        return;
      }
      if (!f.mapping.product_code || !f.mapping.name || !f.mapping.retail_price) {
        appState.addLog("Warning", `Tệp ${f.path.split(/[\\/]/).pop()} chưa được ánh xạ đầy đủ các cột bắt buộc.`);
        return;
      }
    }

    showPreviewModal = true;
    isGeneratingPreview = true;
    previewError = null;
    previewRows = [];

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const rows = await invoke('get_preview_rows', {
        files: $state.snapshot(appState.files),
        limitPerFile: 3
      });
      previewRows = rows;
    } catch (e) {
      previewError = e.toString();
      appState.addLog("Error", `Lỗi tạo bản xem trước: ${e}`);
    } finally {
      isGeneratingPreview = false;
    }
  };

  const handleProceedFromPreview = () => {
    showPreviewModal = false;
    appState.handleProcessAndExport();
  };

  let sidebarLeftComponent;

  // Lang nghe cac event tu Rust backend va drag-drop tu Tauri window
  onMount(() => {
    // Tự động kiểm tra bản cập nhật ngầm khi mở app
    setTimeout(() => {
      if (sidebarLeftComponent && typeof sidebarLeftComponent.checkUpdate === 'function') {
        sidebarLeftComponent.checkUpdate(true);
      }
    }, 1500);

    // Lắng nghe sự kiện yêu cầu đóng ứng dụng
    const appWindow = getCurrentWindow();
    let forceClose = false;
    const unlistenClose = appWindow.onCloseRequested(async (event) => {
      if (forceClose) {
        return;
      }
      if (appState.hasAnyDirtyProjects) {
        event.preventDefault(); // Ngăn chặn hành động đóng ngay lập tức
        const confirmClose = await appState.confirm({
          title: "Thoát ứng dụng",
          message: "Có các thay đổi chưa được lưu trong dự án của bạn. Bạn có chắc chắn muốn thoát ứng dụng không?\nMọi thay đổi chưa lưu sẽ bị mất.",
          confirmText: "Thoát ứng dụng",
          cancelText: "Hủy",
          kind: "danger"
        });
        if (confirmClose) {
          forceClose = true;
          await appWindow.close(); // Đóng hẳn cửa sổ
        }
      }
    });

    const unlistenProgress = listen('progress-update', (event) => {
      const update = event.payload;
      if (update.type === "Progress") {
        appState.progressPercent = update.data.percent;
        appState.statusText = update.data.task;
      } else if (update.type === "Log") {
        appState.addLog(update.data.level, update.data.message, false);
      } else if (update.type === "FileAdded") {
        appState.saveHistoryState();
        const fileWithId = { ...update.data, id: crypto.randomUUID() };
        appState.files = [...appState.files, fileWithId];
        if (appState.selectedFileIdx === -1) {
          appState.selectedFileIdx = appState.files.length - 1;
        }
      } else if (update.type === "AddFilesFinished") {
        appState.isAddingFiles = false;
        appState.progressPercent = 0;
        appState.statusText = "";
      }
    });

    const unlistenLogs = listen('app-log', (event) => {
      appState.addLog("Info", event.payload, false);
    });

    const unlistenDragEnter = listen('tauri://drag-enter', () => {
      isGlobalDragOver = true;
    });

    const unlistenDragLeave = listen('tauri://drag-leave', () => {
      isGlobalDragOver = false;
    });

    const unlistenDragDrop = listen('tauri://drag-drop', async (event) => {
      isGlobalDragOver = false;
      const paths = event.payload?.paths;
      if (paths && paths.length > 0) {
        const validPaths = paths.filter(p => {
          const ext = p.split('.').pop().toLowerCase();
          return ['xlsx', 'xls', 'csv'].includes(ext);
        });
        if (validPaths.length > 0) {
          await appState.addFilesByPaths(validPaths);
        } else {
          appState.showToast("Warning", "Vui lòng kéo thả file Excel (.xlsx, .xls) hoặc CSV (.csv) hợp lệ!");
        }
      }
    });

    appState.outputPath = "D:\\bang_gia_gop_takk." + appState.exportFormat;

    const handleKeyDown = (e) => {
      // Nhận diện MacOS
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const isCmdOrCtrl = isMac ? e.metaKey : e.ctrlKey;

      if (isCmdOrCtrl) {
        if (e.key.toLowerCase() === 'z') {
          e.preventDefault();
          if (e.shiftKey) {
            appState.redo();
          } else {
            appState.undo();
          }
        } else if (e.key.toLowerCase() === 'y') {
          e.preventDefault();
          appState.redo();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      unlistenClose.then(f => f());
      unlistenProgress.then(f => f());
      unlistenLogs.then(f => f());
      unlistenDragEnter.then(f => f());
      unlistenDragLeave.then(f => f());
      unlistenDragDrop.then(f => f());
    };
  });
</script>

<div class="flex h-screen w-screen bg-[var(--background)] text-[var(--text)] font-sans overflow-hidden relative">
  {#if isGlobalDragOver}
    <div class="absolute inset-0 bg-slate-900/60 backdrop-blur-sm z-[9999] flex flex-col items-center justify-center text-white pointer-events-none transition-all duration-300">
      <div class="p-8 rounded-2xl bg-[var(--sidebar-bg)] border border-[var(--border)] flex flex-col items-center justify-center gap-4 text-center max-w-sm shadow-2xl scale-100 transition-transform">
        <div class="w-16 h-16 rounded-full bg-[var(--accent)]/10 text-[var(--accent)] flex items-center justify-center animate-bounce">
          <FileUp size={36} />
        </div>
        <div>
          <h3 class="text-sm font-bold text-[var(--text)] mb-1">Thả tệp cấu hình tại đây</h3>
          <p class="text-xs text-[var(--text-muted)] leading-relaxed">Hỗ trợ các tệp định dạng Excel (.xlsx, .xls) và CSV (.csv)</p>
        </div>
      </div>
    </div>
  {/if}

  <!-- COT 1: NAVIGATION SIDEBAR -->
  <SidebarLeft this={sidebarLeftComponent} />

  <!-- COT 2: FILE LIST & PROJECT CONTROLS -->
  <SidebarMiddle />

  <!-- COT 3: CONFIGURATION DETAIL PANEL -->
  <main class="flex-1 flex flex-col min-w-0 h-full relative z-10">
    
    <!-- TOP BAR -->
    <header class="h-16 flex items-center justify-between px-4 md:px-6 gap-2 md:gap-4 bg-[var(--card-bg)] border-b border-[var(--border)] shrink-0 z-20">
      <div class="flex items-center gap-2 min-w-0">
        <Sliders size={15} class="text-[var(--accent)] shrink-0" />
        <h2 class="text-xs font-bold text-[var(--text)] uppercase tracking-wider truncate max-w-[120px] sm:max-w-[200px] lg:max-w-none">Cấu hình ánh xạ & chuẩn hóa</h2>
      </div>

      <div class="flex items-center gap-2 md:gap-3 shrink-0">
        <div class="flex items-center gap-2">
          <span class="text-xs text-[var(--text-muted)] font-semibold uppercase tracking-wider hidden xl:inline">Định dạng:</span>
          <Select 
            bind:value={appState.exportFormat} 
            options={[
              { value: 'csv', label: 'CSV (Chuẩn)', icon: CsvIcon, iconColor: 'text-sky-500' },
              { value: 'xlsx', label: 'Excel (.xlsx)', icon: ExcelIcon, iconColor: 'text-emerald-500' }
            ]} 
            class="w-32"
          />
        </div>

        <button 
          onclick={handleOpenPreview}
          disabled={appState.isProcessing || appState.files.length === 0}
          class="btn-ghost flex items-center gap-2 disabled:opacity-40 disabled:pointer-events-none text-[var(--text)] font-bold text-xs px-4 py-2 rounded-md cursor-pointer transition active:scale-95 shrink-0"
        >
          <Eye size={14} /> XEM TRƯỚC
        </button>

        <button 
          onclick={() => appState.handleProcessAndExport()}
          disabled={appState.isProcessing || appState.files.length === 0}
          class="btn-primary flex items-center gap-2 disabled:opacity-40 disabled:pointer-events-none text-white font-bold text-xs px-4 py-2 rounded-md cursor-pointer transition active:scale-95 shrink-0"
        >
          {#if appState.isProcessing}
            <RefreshCw size={14} class="animate-spin" /> ĐANG XỬ LÝ...
          {:else}
            <Play size={14} fill="currentColor" /> BẮT ĐẦU XỬ LÝ
          {/if}
        </button>
      </div>
    </header>

    <!-- TAB BAR -->
    <div class="h-10 flex items-end bg-[var(--sidebar-bg)] border-b border-[var(--border)] px-4 gap-1 shrink-0 overflow-x-auto select-none">
      {#each appState.openProjects as proj, idx (proj.id)}
        <div 
          animate:flip={{ duration: 180 }}
          role="button"
          tabindex="0"
          draggable="true"
          ondragstart={(e) => handleDragStart(e, idx)}
          ondragover={(e) => handleDragOver(e, idx)}
          ondrop={(e) => handleDrop(e, idx)}
          class="group h-8 flex items-center gap-2 px-3 rounded-t-md text-xs font-semibold border-t border-x cursor-pointer transition-all duration-200 select-none max-w-[200px] outline-none {appState.activeProjectIdx === idx ? 'bg-[var(--background)] border-[var(--border)] text-[var(--text)]' : 'bg-transparent border-transparent text-[var(--text-muted)] hover:bg-[var(--card-bg)]/50'}"
          onclick={() => appState.switchProjectTab(idx)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              appState.switchProjectTab(idx);
            }
          }}
        >
          {#if proj.files.length > 0 && (proj.path === null || proj.lastSavedState !== JSON.stringify(proj.files.map(f => ({
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
          }))))}
            <span class="w-1.5 h-1.5 rounded-full bg-[var(--accent)] shrink-0" title="Chưa lưu"></span>
          {/if}
          <span class="truncate max-w-[120px]" title={proj.path || proj.name}>{proj.name}</span>
          <button 
            type="button"
            class="p-0.5 rounded-full hover:bg-[var(--border)] text-[var(--text-muted)] hover:text-[var(--text)] transition opacity-0 group-hover:opacity-100 focus:opacity-100"
            onclick={(e) => {
              e.stopPropagation();
              appState.closeProjectTab(idx);
            }}
          >
            <X size={10} />
          </button>
        </div>
      {/each}

      <button 
        class="p-1 rounded-md hover:bg-[var(--card-bg)] text-[var(--text-muted)] hover:text-[var(--text)] transition mb-1 ml-1"
        onclick={() => appState.addNewProjectTab()}
        title="Thêm dự án mới"
      >
        <Plus size={12} />
      </button>
    </div>

    <!-- CONTENT SCROLL AREA -->
    <div class="flex-grow overflow-y-auto pt-6 pb-32 px-10 space-y-6">
      {#if appState.selectedFileIdx === -1 || appState.files.length === 0}
        <div 
          in:scale={{ start: 0.96, duration: 300 }}
          class="h-full flex flex-col justify-between space-y-8"
        >
          <!-- Center Banner -->
          <div class="flex flex-col items-center justify-center text-slate-500 space-y-4 pt-12">
            <div class="relative w-20 h-20 rounded-full bg-zinc-500/5 border border-[var(--border)] flex items-center justify-center text-slate-400 shadow-sm">
              <Sliders size={32} class="text-[var(--accent)] animate-pulse" />
            </div>
            <div class="text-center max-w-sm space-y-1.5">
              <h3 class="text-sm font-bold text-[var(--text)]">Cấu hình Ánh xạ & Chuẩn hóa</h3>
              <p class="text-xs text-[var(--text-muted)] leading-relaxed">Chọn một tệp bảng giá ở danh sách bên trái hoặc nhấn thêm file cấu hình mới để bắt đầu thiết lập.</p>
            </div>
          </div>

          <!-- Recent Files / Recommended Section -->
          {#if appState.displayRecentAndSuggestions.length > 0}
            <div class="border-t border-[var(--border)] pt-8 mt-auto">
              <div class="flex items-center justify-between mb-4">
                <h4 class="text-xs font-bold text-[var(--text)] uppercase tracking-wider">Đề xuất & Lịch sử gần đây</h4>
                <button 
                  onclick={() => appState.clearRecentFiles()}
                  class="text-[10px] text-rose-500 hover:underline transition bg-transparent border-none cursor-pointer"
                >
                  Xóa lịch sử
                </button>
              </div>
              
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each appState.displayRecentAndSuggestions.slice(0, 6) as item}
                  <button
                    onclick={async () => {
                      if (item.type === 'project') {
                        await appState.loadProjectByPath(item.path);
                      } else {
                        await appState.addFilesByPaths([item.path]);
                      }
                    }}
                    class="flex items-center gap-3 p-3 rounded-xl border border-[var(--border)] bg-[var(--card-bg)] hover:bg-[var(--active-file-bg)]/40 hover:border-[var(--accent)]/45 text-left transition group cursor-pointer w-full"
                  >
                    <!-- Icon representation -->
                    <div class="p-2 rounded-lg bg-zinc-500/5 shrink-0 transition group-hover:scale-105">
                      {#if item.type === 'project'}
                        <img src="/file-association.webp" alt="Takk Project" class="w-6 h-6 object-contain rounded" />
                      {:else if item.type === 'csv'}
                        <CsvIcon size={24} class="text-sky-500" />
                      {:else}
                        <ExcelIcon size={24} class="text-emerald-500" />
                      {/if}
                    </div>

                    <!-- Details -->
                    <div class="min-w-0 flex-1">
                      <div class="text-xs font-bold text-[var(--text)] truncate group-hover:text-[var(--accent)] transition-colors">
                        {item.name}
                      </div>
                      <div class="text-[10px] text-[var(--text-muted)] mt-0.5 truncate flex items-center gap-1.5 flex-wrap">
                        <span>
                          {#if item.isSuggestion}
                            <span class="px-1 py-0.5 text-[8px] bg-amber-500/10 text-amber-600 dark:text-amber-400 font-extrabold rounded shrink-0">ĐỀ XUẤT</span>
                          {:else if item.type === 'project'}
                            Dự án Takk
                          {:else if item.type === 'csv'}
                            Tệp CSV
                          {:else}
                            Tệp Excel
                          {/if}
                        </span>
                        {#if item.size}
                          <span class="w-1 h-1 rounded-full bg-slate-500"></span>
                          <span class="font-semibold text-zinc-500 dark:text-zinc-400">{formatFileSize(item.size)}</span>
                        {/if}
                        <span class="w-1 h-1 rounded-full bg-slate-500"></span>
                        <span>
                          {(() => {
                            const diff = Date.now() - item.timestamp;
                            const mins = Math.floor(diff / 60000);
                            const hours = Math.floor(mins / 60);
                            const days = Math.floor(hours / 24);
                            if (mins < 1) return "Vừa xong";
                            if (mins < 60) return `${mins} phút trước`;
                            if (hours < 24) return `${hours} giờ trước`;
                            return `${days} ngày trước`;
                          })()}
                        </span>
                      </div>
                    </div>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {:else}
        <ConfigGeneralInfo />
        <ConfigMapping />
        <ConfigNormalization />
      {/if}
    </div>

    <!-- CONSOLE BAR & OUTPUT FOOTER -->
    <ConsoleBar />

  </main>

  <!-- TOAST NOTIFICATION SYSTEM -->
  <Toast />

  <!-- PREVIEW MODAL -->
  <PreviewModal 
    bind:show={showPreviewModal}
    {previewRows}
    {isGeneratingPreview}
    {previewError}
    {formatCurrency}
    onProceed={handleProceedFromPreview}
  />

  <!-- CONFIRM MODAL -->
  <ConfirmModal />

</div>
