<script>
  import { 
    ChevronDown, ChevronRight, Copy, X, Download, Trash2, Search,
    LayoutDashboard, Bug, Info, CheckCircle, AlertTriangle, AlertCircle
  } from 'lucide-svelte';
  import { appState } from '../lib/state.svelte.js';
  import Select from './ui/Select.svelte';

  const levelOptions = [
    { value: "ALL", label: "TẤT CẢ", icon: LayoutDashboard, iconColor: "text-slate-400" },
    { value: "DEBUG", label: "DEBUG", icon: Bug, iconColor: "text-zinc-500" },
    { value: "INFO", label: "INFO", icon: Info, iconColor: "text-blue-400" },
    { value: "SUCCESS", label: "SUCCESS", icon: CheckCircle, iconColor: "text-emerald-400" },
    { value: "WARN", label: "WARN", icon: AlertTriangle, iconColor: "text-amber-500" },
    { value: "ERROR", label: "ERROR", icon: AlertCircle, iconColor: "text-rose-500" }
  ];
</script>

<!-- Console Logs Bar Drawer -->
<div class="bg-[var(--sidebar-bg)] border-t border-[var(--border)] shrink-0 flex flex-col relative z-20">
  <!-- Draggable Handle Bar -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    aria-valuenow={appState.logsHeight}
    aria-valuemin="80"
    aria-valuemax="800"
    tabindex="-1"
    class="absolute -top-1 left-0 right-0 h-2 cursor-row-resize z-40"
    onmousedown={(e) => appState.handleResizeMouseDown(e)}
  ></div>

  <!-- Output Path Area & Console controls -->
  <div class="px-6 py-2.5 border-b border-[var(--border)] flex items-center justify-between gap-4">
    <!-- Left side: Log Toggle -->
    <div class="flex items-center shrink-0">
      <button 
        onclick={() => appState.isLogsExpanded = !appState.isLogsExpanded}
        class="py-1 px-2.5 btn-ghost text-[10px] font-bold rounded-md cursor-pointer flex items-center gap-1.5 text-[var(--text)]"
      >
        <ChevronDown size={13} class="transition-transform duration-200 {!appState.isLogsExpanded ? '-rotate-90' : ''}" />
        <span class="hidden sm:inline">Nhật ký</span>
        <span>({appState.logs.length})</span>
      </button>
    </div>

    <!-- Right side: Log Tools -->
    <div class="flex items-center gap-2 shrink-0">
      {#if appState.logs.length > 0 && appState.isLogsExpanded}
        <button
          onclick={(e) => { e.stopPropagation(); appState.handleCopyAllLogs(); }}
          class="py-1 px-2.5 btn-ghost text-[10px] font-bold rounded-md cursor-pointer flex items-center gap-1.5 text-[var(--text)]"
          title="Sao chép toàn bộ nhật ký"
        >
          <Copy size={12} />
          <span class="hidden sm:inline">Sao chép hết</span>
        </button>
        <button
          onclick={(e) => { e.stopPropagation(); appState.handleDownloadTxt(); }}
          class="py-1 px-2.5 btn-ghost text-[10px] font-bold rounded-md cursor-pointer flex items-center gap-1.5 text-[var(--text)]"
          title="Tải log về máy dạng .txt"
        >
          <Download size={12} />
          <span class="hidden sm:inline">Tải TXT</span>
        </button>
        <button
          onclick={(e) => { e.stopPropagation(); appState.logs = []; }}
          class="py-1 px-2.5 btn-ghost text-[10px] font-bold rounded-md cursor-pointer text-rose-500 hover:bg-rose-500/5 flex items-center gap-1.5"
          title="Xóa nhật ký"
        >
          <Trash2 size={12} />
          <span class="hidden sm:inline">Xóa log</span>
        </button>
      {/if}
    </div>
  </div>
  
  {#if appState.isLogsExpanded}
    <!-- Log Filters Bar -->
    <div class="flex flex-wrap items-center gap-4 px-6 py-2 bg-zinc-500/5 border-b border-[var(--border)] text-[10px] font-mono">
      <!-- Text Search Filter -->
      <div class="flex items-center gap-1.5 shrink-0">
        <span class="font-bold text-[var(--text-muted)] whitespace-nowrap">Lọc:</span>
        <div class="relative w-44">
          <input 
            type="text" 
            bind:value={appState.searchQuery}
            placeholder="Tìm nội dung log..."
            class="w-full pl-6 pr-2 py-1 input-glass rounded-md focus:outline-none text-[10px]"
          />
          <Search size={11} class="text-zinc-500 absolute left-2 top-1/2 -translate-y-1/2" />
        </div>
      </div>

      <!-- Level Filter Dropdown -->
      <div class="flex items-center gap-1.5 shrink-0">
        <span class="font-bold text-[var(--text-muted)] whitespace-nowrap">Cấp độ:</span>
        <Select 
          bind:value={appState.selectedLevel}
          options={levelOptions}
          class="w-28"
          size="sm"
        />
      </div>
    </div>

    <!-- Interactive Log Terminal Area -->
    <div 
      bind:this={appState.logContainer}
      class="bg-zinc-950 text-emerald-400 p-4 font-mono text-[10px] overflow-y-auto flex flex-col gap-1 select-text scroll-smooth"
      style="height: {appState.logsHeight}px; min-height: 80px; max-height: 70vh;"
    >
      {#if appState.filteredLogs.length === 0}
        <span class="text-zinc-600 italic">Không có dòng nhật ký nào phù hợp.</span>
      {:else}
        {#each appState.filteredLogs as log}
          {@const isLong = log.message.length > 120}
          {@const isExpanded = appState.expandedLogs.has(`${log.time}_${log.message}`)}
          <div 
            class="hover:bg-zinc-900/60 px-2 py-1 rounded transition relative group flex items-start gap-2 text-zinc-300 min-w-0"
          >
            <!-- timestamp -->
            <span class="text-zinc-600 shrink-0 select-none">[{log.time}]</span>
            
            <!-- tag level -->
            <span class="px-1 py-0.2 rounded text-[8px] font-bold shrink-0 select-none uppercase
              {log.level.toLowerCase() === 'debug' ? 'bg-zinc-800 text-zinc-400' : ''}
              {log.level.toLowerCase() === 'info' ? 'bg-blue-950/60 text-blue-400' : ''}
              {log.level.toLowerCase() === 'success' ? 'bg-green-950/60 text-green-400' : ''}
              {log.level.toLowerCase() === 'warn' || log.level.toLowerCase() === 'warning' ? 'bg-yellow-950/60 text-yellow-500' : ''}
              {log.level.toLowerCase() === 'error' ? 'bg-red-950/60 text-red-500 font-bold border border-red-800/40' : ''}
            ">
              {log.level}
            </span>

            <!-- message text -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span 
              onclick={() => { if (isLong) appState.toggleExpand(log); }}
              class="flex-grow select-text cursor-text min-w-0
                {log.level.toLowerCase() === 'debug' ? 'text-zinc-500' : ''}
                {log.level.toLowerCase() === 'success' ? 'text-emerald-400 font-medium' : ''}
                {log.level.toLowerCase() === 'warn' || log.level.toLowerCase() === 'warning' ? 'text-amber-300' : ''}
                {log.level.toLowerCase() === 'error' ? 'text-rose-400 font-semibold' : ''}
                {!isExpanded && isLong ? 'line-clamp-1 truncate' : 'whitespace-pre-wrap break-all'}
                {isLong ? 'hover:underline cursor-pointer' : ''}"
              title={isLong ? (isExpanded ? "Click để thu gọn" : "Click để xem đầy đủ") : ""}
            >
              {log.message}
            </span>
            
            <!-- actions group on hover -->
            <div class="flex items-center gap-1 shrink-0 ml-2 select-none opacity-0 group-hover:opacity-100 transition">
              {#if isLong}
                <button
                  onclick={(e) => { e.stopPropagation(); appState.toggleExpand(log); }}
                  class="p-0.5 hover:bg-zinc-800 rounded text-zinc-400 hover:text-white transition cursor-pointer"
                  title={isExpanded ? "Thu gọn" : "Mở rộng"}
                >
                  {#if isExpanded}
                    <ChevronDown size={11} />
                  {:else}
                    <ChevronRight size={11} />
                  {/if}
                </button>
              {/if}
              <button
                onclick={(e) => { e.stopPropagation(); appState.handleCopySingleLog(log); }}
                class="p-0.5 hover:bg-zinc-800 rounded text-zinc-400 hover:text-white transition cursor-pointer"
                title="Sao chép dòng log"
              >
                <Copy size={11} />
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); appState.logs = appState.logs.filter(l => l !== log); }}
                class="p-0.5 hover:bg-zinc-800 rounded text-zinc-400 hover:text-rose-400 transition cursor-pointer"
                title="Xóa dòng log"
              >
                <X size={11} />
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>
