<script>
  import {
    FolderOpen,
    Save,
    Plus,
    Trash2,
    ArrowUp,
    ArrowDown,
    X,
    FileUp,
    FileSpreadsheet,
    SaveAll,
    GripVertical,
    Loader2,
    AlertTriangle,
    ArrowUpDown,
    FileX,
    Undo2,
    Redo2,
  } from "lucide-svelte";
  import { slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { appState } from "../lib/state.svelte.js";
  import CsvIcon from "./icons/CsvIcon.svelte";
  import ExcelIcon from "./icons/ExcelIcon.svelte";

  const isFileInvalid = (file) => {
    if (!file) return false;
    const hasMissingInfo = !file.brand || !file.brand.trim() || !file.provider || !file.provider.trim();
    const hasMissingMapping = !file.mapping || !file.mapping.product_code || !file.mapping.name || !file.mapping.retail_price;
    return hasMissingInfo || hasMissingMapping;
  };

  const getDisplayName = (path) => {
    if (!path) return "";
    const fileName = path.split(/[\\/]/).pop();
    // Nếu file nằm trong thư mục tạm, loại bỏ tiền tố index (ví dụ "0_HONDA" -> "HONDA")
    if (path.includes("Takk_Projects") || path.toLowerCase().includes("temp")) {
      return fileName.replace(/^\d+_/, "");
    }
    return fileName;
  };

  async function handleRemoveInvalidFiles() {
    const invalidFiles = appState.files.filter(isFileInvalid);
    if (invalidFiles.length === 0) return;

    const confirmDelete = await appState.confirm({
      title: "Loại bỏ tệp cấu hình lỗi",
      message: `Bạn có chắc chắn muốn loại bỏ nhanh ${invalidFiles.length} tệp cấu hình bị lỗi (thiếu Hãng, Nhà cung cấp hoặc chưa ánh xạ các cột bắt buộc) không?\nHành động này có thể hoàn tác bằng Ctrl+Z.`,
      confirmText: "Loại bỏ",
      cancelText: "Hủy",
      kind: "danger"
    });

    if (confirmDelete) {
      appState.removeInvalidFiles(invalidFiles.map(f => f.id));
      appState.showToast("Success", `Đã loại bỏ ${invalidFiles.length} tệp cấu hình lỗi.`);
    }
  }

  // --- Trang thai keo tha bang pointer events ---
  let draggedIdx = $state(null);
  let hoveredIdx = $state(null);
  let isDragging = $state(false);

  // Toa do ghost card
  let ghostX = $state(0);
  let ghostY = $state(0);
  let ghostFile = $state(null);

  // Offset tu diem click den goc trai tren cua card
  let offsetX = 0;
  let offsetY = 0;

  // Danh sach phan tu card de tinh vi tri
  let listEl = $state(null);

  function getIdxFromY(clientY) {
    if (!listEl) return null;
    const cards = [...listEl.querySelectorAll("[data-file-card]")];
    for (let i = 0; i < cards.length; i++) {
      const rect = cards[i].getBoundingClientRect();
      const mid = rect.top + rect.height / 2;
      if (clientY < mid) return i;
    }
    return cards.length - 1;
  }

  function onPointerDown(e, idx) {
    // Chi keo tu grip handle
    if (!e.target.closest("[data-grip]")) return;
    e.preventDefault();

    draggedIdx = idx;
    ghostFile = appState.files[idx];

    const card = e.currentTarget;
    const rect = card.getBoundingClientRect();
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;

    ghostX = e.clientX - offsetX;
    ghostY = e.clientY - offsetY;

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  function onPointerMove(e) {
    if (draggedIdx === null) return;
    isDragging = true;
    ghostX = e.clientX - offsetX;
    ghostY = e.clientY - offsetY;

    const newHovered = getIdxFromY(e.clientY);
    if (newHovered !== null) hoveredIdx = newHovered;
  }

  function onPointerUp(e) {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);

    if (
      isDragging &&
      draggedIdx !== null &&
      hoveredIdx !== null &&
      draggedIdx !== hoveredIdx
    ) {
      const files = [...appState.files];
      const [item] = files.splice(draggedIdx, 1);
      files.splice(hoveredIdx, 0, item);
      appState.saveHistoryState();
      appState.files = files;
      appState.selectedFileIdx = hoveredIdx;
    }

    draggedIdx = null;
    hoveredIdx = null;
    isDragging = false;
    ghostFile = null;
  }

  function parseStatus(text) {
    if (!text) return { main: "Đang xử lý...", sub: "" };
    const parts = text.split(":");
    if (parts.length > 1) {
      let mainText = parts[0].trim();
      const subText = parts.slice(1).join(":").trim();

      const mainUpper = mainText.toUpperCase();
      if (mainUpper.includes("DANG XU LY CAU HINH")) {
        mainText =
          "Đang xử lý cấu hình " +
            mainUpper.match(/\((\d+\s*\/\s*\d+)\)/)?.[0] || "";
      } else if (mainUpper.includes("DANG PHAN TICH")) {
        mainText = "Đang phân tích cấu hình...";
      } else {
        mainText =
          mainText.charAt(0).toUpperCase() + mainText.slice(1).toLowerCase();
      }
      return { main: mainText, sub: subText };
    }

    let mainText = text.trim();
    if (mainText.toUpperCase().includes("DANG PHAN TICH CAU HINH TEP")) {
      mainText = "Đang phân tích cấu hình tệp...";
    }
    return { main: mainText, sub: "" };
  }
</script>

<!-- Ghost card theo con tro -->
{#if isDragging && ghostFile}
  {@const ext = ghostFile.path.split(".").pop().toLowerCase()}
  <div
    class="ghost-card fixed z-[999] p-3 rounded-xl border border-[var(--accent)] bg-[var(--active-file-bg)] shadow-2xl shadow-[var(--accent)]/20 flex items-center gap-2 opacity-90 pointer-events-none"
    style="left: {ghostX}px; top: {ghostY}px; width: 280px;"
  >
    <div
      class="p-2 rounded-lg bg-zinc-500/5 shrink-0
      {ext === 'csv' ? 'text-sky-500' : 'text-emerald-500'}"
    >
      {#if ext === "csv"}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="15"
          height="15"
          viewBox="0 0 512 512"
          class="w-[15px] h-[15px] fill-current"
        >
          <path
            d="M0 64C0 28.7 28.7 0 64 0h160v128c0 17.7 14.3 32 32 32h128v144H176c-35.3 0-64 28.7-64 64v144H64c-35.3 0-64-28.7-64-64zm384 64H256V0zM200 352h16c22.1 0 40 17.9 40 40v8c0 8.8-7.2 16-16 16s-16-7.2-16-16v-8c0-4.4-3.6-8-8-8h-16c-4.4 0-8 3.6-8 8v80c0 4.4 3.6 8 8 8h16c4.4 0 8-3.6 8-8v-8c0-8.8 7.2-16 16-16s16 7.2 16 16v8c0 22.1-17.9 40-40 40h-16c-22.1 0-40-17.9-40-40v-80c0-22.1 17.9-40 40-40m133.1 0H368c8.8 0 16 7.2 16 16s-7.2 16-16 16h-34.9c-7.2 0-13.1 5.9-13.1 13.1c0 5.2 3 9.9 7.8 12l37.4 16.6c16.3 7.2 26.8 23.4 26.8 41.2c0 24.9-20.2 45.1-45.1 45.1H304c-8.8 0-16-7.2-16-16s7.2-16 16-16h42.9c7.2 0 13.1-5.9 13.1-13.1c0-5.2-3-9.9-7.8-12l-37.4-16.6c-16.3-7.2-26.8-23.4-26.8-41.2c0-24.9 20.2-45.1 45.1-45.1m98.9 0c8.8 0 16 7.2 16 16v31.6c0 23 5.5 45.6 16 66c10.5-20.3 16-42.9 16-66V368c0-8.8 7.2-16 16-16s16 7.2 16 16v31.6c0 34.7-10.3 68.7-29.6 97.6l-5.1 7.7c-3 4.5-8 7.1-13.3 7.1s-10.3-2.7-13.3-7.1l-5.1-7.7c-19.3-28.9-29.6-62.9-29.6-97.6V368c0-8.8 7.2-16 16-16"
          />
        </svg>
      {:else}
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="15"
          height="15"
          viewBox="0 0 384 512"
          class="w-[15px] h-[15px] fill-current"
        >
          <path
            d="M64 0C28.7 0 0 28.7 0 64v384c0 35.3 28.7 64 64 64h256c35.3 0 64-28.7 64-64V160H256c-17.7 0-32-14.3-32-32V0zm192 0v128h128zM155.7 250.2l36.3 51.9l36.3-51.9c7.6-10.9 22.6-13.5 33.4-5.9s13.5 22.6 5.9 33.4L221.3 344l46.4 66.2c7.6 10.9 5 25.8-5.9 33.4s-25.8 5-33.4-5.9L192 385.8l-36.3 51.9c-7.6 10.9-22.6 13.5-33.4 5.9s-13.5-22.6-5.9-33.4l46.3-66.2l-46.4-66.2c-7.6-10.9-5-25.8 5.9-33.4s25.8-5 33.4 5.9z"
          />
        </svg>
      {/if}
    </div>
    <div class="min-w-0 flex-1">
      <div class="text-xs font-bold truncate text-[var(--text)]">
        {ghostFile.path.split(/[\\/]/).pop()}
      </div>
      <span
        class="px-1.5 py-0.5 rounded text-[8px] font-bold uppercase tracking-wider bg-zinc-500/10 text-[var(--text-muted)]"
      >
        {ghostFile.brand || "CHƯA NHẬP HÃNG"}
      </span>
    </div>
  </div>
{/if}

<aside
  class="w-80 flex flex-col bg-[var(--sidebar-bg)] border-r border-[var(--border)] shrink-0 select-none z-10 shadow-sm"
>
  <!-- Workspace Title & Project Controls -->
  <div
    class="h-16 px-4 border-b border-[var(--border)] flex justify-between items-center shrink-0"
  >
    <div class="flex flex-col gap-1 min-w-0">
      <span
        class="text-[9px] text-[var(--text-muted)] font-bold uppercase tracking-wider"
        >Không gian làm việc</span
      >
      <span class="text-xs font-bold text-[var(--text)] truncate">
        {appState.currentProjectPath
          ? appState.currentProjectPath.split(/[\\/]/).pop()
          : "Chưa lưu cấu hình"}
      </span>
    </div>
    <div class="flex items-center gap-1 shrink-0 ml-2">
      <!-- Mo du an: Highlight bang mau chu dao -->
      <button
        onclick={() => appState.handleOpenProject()}
        class="p-1.5 bg-[var(--accent)] text-white hover:bg-[var(--accent)]/90 rounded-md cursor-pointer transition shadow-sm flex items-center justify-center gap-1.5"
        title="Mở dự án (.bgx, .bg)"
      >
        <FolderOpen size={13} />
        {#if !(appState.files.length > 0 || appState.currentProjectPath)}
          <span class="text-xs font-medium pr-1">Mở dự án</span>
        {/if}
      </button>

      {#if appState.files.length > 0 || appState.currentProjectPath}
        <!-- Luu du an -->
        <button
          onclick={() => appState.handleSaveProject(false)}
          class="p-1.5 hover:bg-[var(--active-file-bg)] rounded-md text-[var(--text-muted)] hover:text-[var(--text)] cursor-pointer transition flex items-center justify-center"
          title="Lưu dự án hiện tại"
        >
          <Save size={13} />
        </button>
        <!-- Luu thanh (Save As) -->
        <button
          onclick={() => appState.handleSaveProject(true)}
          class="p-1.5 hover:bg-[var(--active-file-bg)] rounded-md text-[var(--text-muted)] hover:text-[var(--text)] cursor-pointer transition flex items-center justify-center"
          title="Lưu thành dự án mới..."
        >
          <SaveAll size={13} />
        </button>
        <!-- Don dep Workspace -->
        <button
          onclick={() => appState.handleReset()}
          class="p-1.5 hover:bg-rose-500/10 rounded-md text-rose-500 cursor-pointer transition flex items-center justify-center"
          title="Dọn dẹp toàn bộ Workspace"
        >
          <Trash2 size={13} />
        </button>
      {/if}
    </div>
  </div>

  <!-- Files List Area -->
  <div class="flex-grow overflow-y-auto p-4 space-y-2">
    <div class="flex justify-between items-center px-1 mb-1.5">
      <span
        class="text-[9px] text-[var(--text-muted)] font-bold uppercase tracking-wider"
        >Danh sách file ({appState.files.length})</span
      >
      {#if appState.files.length > 0}
        <div class="flex items-center gap-1">
          <button
            onclick={() => appState.undo()}
            disabled={appState.historyUndoStack.length === 0}
            class="p-1 hover:bg-[var(--active-file-bg)] rounded text-[var(--text-muted)] hover:text-[var(--accent)] disabled:opacity-30 disabled:pointer-events-none cursor-pointer transition"
            title="Hoàn tác (Ctrl+Z)"
          >
            <Undo2 size={12} />
          </button>
          <button
            onclick={() => appState.redo()}
            disabled={appState.historyRedoStack.length === 0}
            class="p-1 hover:bg-[var(--active-file-bg)] rounded text-[var(--text-muted)] hover:text-[var(--accent)] disabled:opacity-30 disabled:pointer-events-none cursor-pointer transition"
            title="Làm lại (Ctrl+Y / Ctrl+Shift+Z)"
          >
            <Redo2 size={12} />
          </button>
          {#if appState.files.some(isFileInvalid)}
            <button
              onclick={handleRemoveInvalidFiles}
              class="p-1 hover:bg-rose-500/10 rounded text-rose-500 cursor-pointer transition animate-pulse"
              title="Loại bỏ nhanh các tệp lỗi cấu hình"
            >
              <FileX size={12} />
            </button>
          {/if}
          <button
            onclick={() => appState.sortFilesByPriority()}
            class="p-1 hover:bg-[var(--active-file-bg)] rounded text-[var(--accent)] cursor-pointer transition"
            title="Sắp xếp theo thứ tự ưu tiên"
          >
            <ArrowUpDown size={12} />
          </button>
          <button
            onclick={() => appState.handleAddFiles()}
            class="p-1 hover:bg-[var(--active-file-bg)] rounded text-[var(--accent)] cursor-pointer transition"
            title="Thêm file nhanh"
          >
            <Plus size={12} />
          </button>
        </div>
      {/if}
    </div>

    {#if appState.files.length === 0}
      <!-- Chi hien thi hop them file cau hinh dang to khi chua co file nao -->
      <button
        onclick={() => appState.handleAddFiles()}
        class="w-full h-40 flex flex-col items-center justify-center border border-dashed border-[var(--accent)]/40 hover:border-[var(--accent)] bg-[var(--accent-glow)]/40 hover:bg-[var(--accent-glow)] rounded-xl text-[var(--accent)] cursor-pointer transition group p-4 text-center mt-2"
      >
        <FileUp
          size={24}
          class="mb-2 group-hover:scale-110 transition-transform"
        />
        <span class="text-xs font-bold text-[var(--text)]"
          >Thêm file cấu hình</span
        >
        <span class="text-[9px] text-[var(--text-muted)] mt-1"
          >Click chọn hoặc Kéo thả file Excel/CSV vào đây</span
        >
      </button>
    {:else}
      <!-- Container de tinh vi tri hover -->
      <div bind:this={listEl} class="space-y-2">
        {#each appState.files as file, idx (file.id)}
          {@const ext = file.path.split(".").pop().toLowerCase()}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            data-file-card
            animate:flip={{ duration: 250 }}
            onpointerdown={(e) => onPointerDown(e, idx)}
            onclick={() => {
              if (!isDragging) appState.selectedFileIdx = idx;
            }}
            class="p-3 rounded-xl border text-left transition duration-200 relative group flex items-start gap-2
              {draggedIdx === idx && isDragging
                ? 'opacity-30 border-dashed border-[var(--border)] bg-transparent scale-95'
                : 'cursor-pointer'}
              {hoveredIdx === idx && isDragging && draggedIdx !== idx
                ? 'border-[var(--accent)] bg-[var(--accent-glow)]/30 scale-[1.02] shadow-lg shadow-[var(--accent)]/5'
                : ''}
              {draggedIdx !== idx
                ? file.not_found
                  ? appState.selectedFileIdx === idx
                    ? 'bg-amber-500/10 border-amber-500 shadow-md shadow-amber-500/5'
                    : 'bg-amber-500/5 border-amber-500/35 hover:bg-amber-500/10 hover:border-amber-500/55 shadow-sm shadow-amber-500/5'
                  : isFileInvalid(file)
                    ? appState.selectedFileIdx === idx
                      ? 'bg-rose-500/10 border-rose-500 shadow-md shadow-rose-500/5'
                      : 'bg-rose-500/5 border-rose-500/35 hover:bg-rose-500/10 hover:border-rose-500/55 shadow-sm shadow-rose-500/5'
                    : appState.selectedFileIdx === idx
                      ? 'bg-[var(--active-file-bg)] border-[var(--accent)]/55 shadow-md shadow-[var(--accent)]/5'
                      : 'bg-[var(--card-bg)] border-[var(--border)] hover:bg-[var(--active-file-bg)]/30 hover:border-zinc-500/30'
                : ''}"
          >
            <!-- Vach chi thi vi tri tha (Drop indicator line) -->
            {#if hoveredIdx === idx && isDragging && draggedIdx !== idx}
              {#if draggedIdx > idx}
                <div
                  class="absolute -top-[5px] left-0 right-0 h-[3px] bg-[var(--accent)] rounded-full z-20 pointer-events-none"
                ></div>
              {:else}
                <div
                  class="absolute -bottom-[5px] left-0 right-0 h-[3px] bg-[var(--accent)] rounded-full z-20 pointer-events-none"
                ></div>
              {/if}
            {/if}

            <!-- Tay nam keo tha (Grip handle) - chi cai nay moi activate drag -->
            <div
              data-grip
              class="flex items-center self-stretch pr-0.5 text-slate-400/60 hover:text-[var(--accent)] transition-all opacity-0 group-hover:opacity-100 -ml-1 cursor-grab active:cursor-grabbing"
              title="Kéo để sắp xếp"
            >
              <GripVertical size={13} />
            </div>
            <!-- Kích hoạt build lại -->
            <div
              class="p-2 rounded-lg bg-zinc-500/5 shrink-0 transition-colors
              {ext === 'csv'
                ? 'text-sky-500/80 group-hover:text-sky-500'
                : 'text-emerald-500/80 group-hover:text-emerald-500'}"
            >
              {#if ext === "csv"}
                <CsvIcon size={15} />
              {:else}
                <ExcelIcon size={15} />
              {/if}
            </div>

            <div class="min-w-0 flex-1 space-y-1.5">
              <div
                class="text-xs font-bold truncate text-[var(--text)] flex items-center gap-1.5"
                title={file.path}
              >
                {#if file.not_found}
                  <AlertTriangle size={12} class="text-amber-500 shrink-0" />
                {:else if isFileInvalid(file)}
                  <AlertTriangle size={12} class="text-rose-500 shrink-0" />
                {/if}
                <span class="truncate">{getDisplayName(file.path)}</span>
              </div>

              <div class="flex items-center gap-2 flex-wrap">
                <span
                  class="px-1.5 py-0.5 rounded text-[8px] font-bold uppercase tracking-wider bg-zinc-500/10 text-[var(--text-muted)]"
                >
                  {file.brand || "CHƯA NHẬP HÃNG"}
                </span>
                <span
                  class="px-1.5 py-0.5 rounded text-[8px] font-semibold bg-zinc-500/5 text-[var(--text-muted)] max-w-[80px] truncate"
                >
                  {file.provider || "Không có NCC"}
                </span>
              </div>
            </div>

            <!-- File actions overlay on hover -->
            <div
              class="absolute right-2 top-2 hidden group-hover:flex items-center gap-0.5 bg-[var(--sidebar-bg)] border border-[var(--border)] rounded-md shadow-lg p-0.5 z-10"
            >
              <button
                onclick={(e) => {
                  e.stopPropagation();
                  appState.moveFile(idx, "up");
                }}
                disabled={idx === 0}
                class="p-1 hover:bg-[var(--active-file-bg)] rounded text-slate-500 hover:text-[var(--text)] disabled:opacity-30 cursor-pointer animate-none"
              >
                <ArrowUp size={11} />
              </button>
              <button
                onclick={(e) => {
                  e.stopPropagation();
                  appState.moveFile(idx, "down");
                }}
                disabled={idx === appState.files.length - 1}
                class="p-1 hover:bg-[var(--active-file-bg)] rounded text-slate-500 hover:text-[var(--text)] disabled:opacity-30 cursor-pointer animate-none"
              >
                <ArrowDown size={11} />
              </button>
              <button
                onclick={(e) => {
                  e.stopPropagation();
                  appState.removeFile(idx);
                }}
                class="p-1 hover:bg-rose-500/10 hover:text-rose-500 rounded text-slate-500 cursor-pointer animate-none"
              >
                <X size={11} />
              </button>
            </div>
          </div>
        {/each}
      </div>

      <!-- Nut them file phu (nam duoi danh sach file khi da co tep) -->
      <div class="pt-2">
        <button
          onclick={() => appState.handleAddFiles()}
          class="w-full flex flex-col items-center justify-center p-4 border border-dashed border-[var(--accent)]/40 hover:border-[var(--accent)] bg-[var(--accent-glow)]/30 hover:bg-[var(--accent-glow)] rounded-xl text-[var(--accent)] cursor-pointer transition group"
        >
          <FileUp
            size={16}
            class="mb-1 group-hover:scale-110 transition-transform"
          />
          <span class="text-xs font-bold text-[var(--text)]"
            >Thêm file cấu hình</span
          >
          <span class="text-[9px] text-[var(--text-muted)] mt-0.5"
            >Click chọn hoặc Kéo thả file</span
          >
        </button>
      </div>
    {/if}
  </div>

  <!-- Tien trinh -->
  {#if appState.isAddingFiles || appState.isProcessing}
    {@const status = parseStatus(appState.statusText)}
    <div
      transition:slide
      class="p-4 bg-[var(--background)] border-t border-[var(--border)] shrink-0 space-y-2.5"
    >
      <div class="flex items-start gap-2.5">
        <div class="text-[var(--accent)] shrink-0 mt-0.5 animate-spin">
          <Loader2 size={13} />
        </div>
        <div class="flex-grow min-w-0 space-y-1">
          <div
            class="flex justify-between items-center text-[10px] font-bold tracking-wider uppercase text-[var(--accent)]"
          >
            <span class="truncate pr-2">{status.main}</span>
            <span class="shrink-0"
              >{Math.round(appState.progressPercent * 100)}%</span
            >
          </div>
          {#if status.sub}
            <div
              class="text-[9px] text-[var(--text-muted)] truncate"
              title={status.sub}
            >
              {status.sub}
            </div>
          {/if}
        </div>
      </div>
      <div class="w-full h-1 bg-zinc-500/10 rounded-full overflow-hidden">
        <div
          class="h-full bg-gradient-to-r from-cyan-400 to-sky-500 rounded-full transition-all duration-300"
          style="width: {appState.progressPercent * 100}%"
        ></div>
      </div>
    </div>
  {:else}
    <div
      class="h-10 text-center text-[10px] text-[var(--text-muted)] border-t border-[var(--border)] shrink-0 select-none flex items-center justify-center"
    >
      © {new Date().getFullYear()} Hùng Tay Ga. Bảo lưu mọi quyền.
    </div>
  {/if}
</aside>

<style>
  .ghost-card {
    transform: rotate(1.5deg);
    transition: none;
  }
</style>
