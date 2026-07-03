<script>
  import { 
    Sliders, Tag, Coins, Settings, 
    QrCode, History, Type, 
    TrendingDown, TrendingUp, 
    Bike, Palette, StickyNote 
  } from 'lucide-svelte';
  import { fly } from 'svelte/transition';
  import { appState } from '../lib/state.svelte.js';
  import Select from './ui/Select.svelte';

  // Chuyển đổi headers thành định dạng options của Select
  let selectOptions = $derived(
    appState.currentFile 
      ? [{ value: "", label: "(Bỏ qua)" }, ...appState.currentFile.headers.map(h => ({ value: h, label: h }))]
      : []
  );
</script>

{#if appState.currentFile}
  {@const file = appState.currentFile}
  <!-- ÁNH XẠ CỘT -->
  <div 
    in:fly={{ y: 20, duration: 350, delay: 100 }}
    class="glass-panel p-6 shadow-sm space-y-5"
  >
    <h2 class="text-xs font-bold text-slate-800 dark:text-slate-200 flex items-center gap-2 border-b border-slate-100 dark:border-zinc-800 pb-3">
      <Sliders size={14} class="text-blue-600 dark:text-cyan-400" />
      Ánh xạ dữ liệu cột
    </h2>

    <div class="grid grid-cols-3 gap-8">
      <!-- Nhóm 1: Mã & Tên -->
      <div class="space-y-4">
        <h4 class="text-[10px] font-bold text-blue-600 dark:text-cyan-400 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-zinc-800 flex items-center gap-1.5">
          <Tag size={12} />
          Mã & Tên
        </h4>
        
        <div class="space-y-4">
          <!-- Mã phụ tùng -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <QrCode size={11} class="text-slate-400 dark:text-zinc-500" />
              Mã phụ tùng (*)
            </span>
            <Select value={file.mapping.product_code} onchange={(v) => appState.updateMappingField(file, 'product_code', v)} options={selectOptions} />
          </div>
          <!-- Mã cũ -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <History size={11} class="text-slate-400 dark:text-zinc-500" />
              Mã cũ/thay thế
            </span>
            <Select value={file.mapping.alt_code} onchange={(v) => appState.updateMappingField(file, 'alt_code', v)} options={selectOptions} />
          </div>
          <!-- Tên -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <Type size={11} class="text-slate-400 dark:text-zinc-500" />
              Tên phụ tùng (*)
            </span>
            <Select value={file.mapping.name} onchange={(v) => appState.updateMappingField(file, 'name', v)} options={selectOptions} />
          </div>
        </div>
      </div>

      <!-- Nhóm 2: Giá cả -->
      <div class="space-y-4">
        <h4 class="text-[10px] font-bold text-blue-600 dark:text-cyan-400 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-zinc-800 flex items-center gap-1.5">
          <Coins size={12} />
          Giá cả (VND)
        </h4>
        
        <div class="space-y-4">
          <!-- Giá nhập -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <TrendingDown size={11} class="text-slate-400 dark:text-zinc-500" />
              Giá vốn/nhập
            </span>
            <Select value={file.mapping.cost_price} onchange={(v) => appState.updateMappingField(file, 'cost_price', v)} options={selectOptions} />
          </div>
          <!-- Giá bán lẻ -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <TrendingUp size={11} class="text-slate-400 dark:text-zinc-500" />
              Giá bán lẻ (*)
            </span>
            <Select value={file.mapping.retail_price} onchange={(v) => appState.updateMappingField(file, 'retail_price', v)} options={selectOptions} />
          </div>
        </div>
      </div>

      <!-- Nhóm 3: Phụ trợ -->
      <div class="space-y-4">
        <h4 class="text-[10px] font-bold text-blue-600 dark:text-cyan-400 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-zinc-800 flex items-center gap-1.5">
          <Settings size={12} />
          Phụ trợ
        </h4>
        
        <div class="space-y-4">
          <!-- Đời xe -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <Bike size={11} class="text-slate-400 dark:text-zinc-500" />
              Đời xe/Model
            </span>
            <Select value={file.mapping.model} onchange={(v) => appState.updateMappingField(file, 'model', v)} options={selectOptions} />
          </div>
          <!-- Mã màu -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <Palette size={11} class="text-slate-400 dark:text-zinc-500" />
              Mã màu
            </span>
            <Select value={file.mapping.color_code} onchange={(v) => appState.updateMappingField(file, 'color_code', v)} options={selectOptions} />
          </div>
          <!-- Ghi chú -->
          <div>
            <span class="flex items-center gap-1.5 text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">
              <StickyNote size={11} class="text-slate-400 dark:text-zinc-500" />
              Ghi chú
            </span>
            <Select value={file.mapping.note} onchange={(v) => appState.updateMappingField(file, 'note', v)} options={selectOptions} />
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
