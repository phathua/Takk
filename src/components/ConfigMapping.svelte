<script>
  import { Sliders } from 'lucide-svelte';
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
        <h4 class="text-[10px] font-bold text-blue-600 dark:text-cyan-400 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-zinc-800">Mã & Tên</h4>
        
        <div class="space-y-4">
          <!-- Mã phụ tùng -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Mã phụ tùng (*)</span>
            <Select bind:value={file.mapping.product_code} options={selectOptions} />
          </div>
          <!-- Mã cũ -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Mã cũ/thay thế</span>
            <Select bind:value={file.mapping.alt_code} options={selectOptions} />
          </div>
          <!-- Tên -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Tên phụ tùng (*)</span>
            <Select bind:value={file.mapping.name} options={selectOptions} />
          </div>
        </div>
      </div>

      <!-- Nhóm 2: Giá cả -->
      <div class="space-y-4">
        <h4 class="text-[10px] font-bold text-blue-600 dark:text-cyan-400 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-zinc-800">Giá cả (VND)</h4>
        
        <div class="space-y-4">
          <!-- Giá nhập -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Giá vốn/nhập</span>
            <Select bind:value={file.mapping.cost_price} options={selectOptions} />
          </div>
          <!-- Giá bán lẻ -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Giá bán lẻ (*)</span>
            <Select bind:value={file.mapping.retail_price} options={selectOptions} />
          </div>
        </div>
      </div>

      <!-- Nhóm 3: Phụ trợ -->
      <div class="space-y-4">
        <h4 class="text-[10px] font-bold text-blue-600 dark:text-cyan-400 uppercase tracking-widest pb-1 border-b border-slate-100 dark:border-zinc-800">Phụ trợ</h4>
        
        <div class="space-y-4">
          <!-- Đời xe -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Đời xe/Model</span>
            <Select bind:value={file.mapping.model} options={selectOptions} />
          </div>
          <!-- Mã màu -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Mã màu</span>
            <Select bind:value={file.mapping.color_code} options={selectOptions} />
          </div>
          <!-- Ghi chú -->
          <div>
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5 uppercase">Ghi chú</span>
            <Select bind:value={file.mapping.note} options={selectOptions} />
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
