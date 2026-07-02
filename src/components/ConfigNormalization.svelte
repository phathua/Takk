<script>
  import { Sparkles, RefreshCw } from 'lucide-svelte';
  import { fly, slide } from 'svelte/transition';
  import { appState } from '../lib/state.svelte.js';
  import Select from './ui/Select.svelte';

  // Định nghĩa các biến dẫn xuất dựa trên file hiện tại
  let isProductCodeUnmapped = $derived(appState.currentFile ? !appState.currentFile.mapping.product_code : true);
  let isCostPriceMapped = $derived(appState.currentFile ? !!appState.currentFile.mapping.cost_price : false);
  let isRetailPriceUnmapped = $derived(appState.currentFile ? !appState.currentFile.mapping.retail_price : true);

  // Tự động vô hiệu hóa generate_cost = false nếu không hợp lệ
  $effect(() => {
    const file = appState.currentFile;
    if (file) {
      if (isCostPriceMapped || isRetailPriceUnmapped) {
        file.generate_cost = false;
      }
    }
  });
</script>

{#if appState.currentFile}
  {@const file = appState.currentFile}
  <!-- CHUẨN HÓA & TẠO GIÁ VỐN -->
  <div 
    in:fly={{ y: 20, duration: 350, delay: 200 }}
    class="grid grid-cols-2 gap-6"
  >
    <!-- Chuẩn hóa mã -->
    <div class="glass-panel p-6 shadow-sm space-y-4 relative overflow-hidden">
      <h2 class="text-xs font-bold text-slate-800 dark:text-slate-200 flex items-center gap-2 border-b border-slate-100 dark:border-zinc-800 pb-3">
        <Sparkles size={14} class="text-blue-600 dark:text-cyan-400" />
        Chuẩn hóa mã phụ tùng
      </h2>

      <div class="space-y-4 text-xs {isProductCodeUnmapped ? 'opacity-40 pointer-events-none select-none' : ''}">
        <!-- Basic normal -->
        <label class="flex items-center gap-3 p-3.5 input-glass hover:bg-slate-500/5 rounded-xl cursor-pointer">
          <input type="checkbox" bind:checked={file.normalize_basic} disabled={isProductCodeUnmapped} />
          <div class="space-y-0.5">
            <span class="font-bold text-slate-800 dark:text-slate-200 block text-xs">Chuẩn hóa cơ bản</span>
            <span class="text-[10px] text-slate-500 dark:text-slate-400 block">Loại bỏ khoảng trắng, dấu gạch ngang (-) và viết HOA</span>
          </div>
        </label>

        <!-- Special normal -->
        <label class="flex items-center gap-3 p-3.5 input-glass hover:bg-slate-500/5 rounded-xl cursor-pointer">
          <input type="checkbox" bind:checked={file.normalize_special} disabled={isProductCodeUnmapped} />
          <div class="space-y-0.5">
            <span class="font-bold text-slate-800 dark:text-slate-200 block text-xs">Chuẩn hóa đặc biệt</span>
            <span class="text-[10px] text-slate-500 dark:text-slate-400 block">Thêm ký tự phân định để tránh trùng lặp</span>
          </div>
        </label>

        {#if file.normalize_special && !isProductCodeUnmapped}
          <div 
            transition:slide
            class="p-4 border border-slate-200 dark:border-zinc-800 rounded-xl bg-white dark:bg-zinc-950 grid grid-cols-2 gap-4"
          >
            <div>
              <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5">Vị trí</span>
              <Select 
                bind:value={file.normalize_position} 
                options={[
                  { value: 'Prefix', label: 'Thêm vào đầu' },
                  { value: 'Suffix', label: 'Thêm vào cuối' }
                ]} 
              />
            </div>
            <div>
              <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1.5">Ký tự thêm vào</span>
              <input 
                type="text" 
                bind:value={file.normalize_suffix}
                placeholder="VD: M" 
                class="w-full input-glass rounded-lg px-3 py-1.5 text-xs text-slate-800 dark:text-slate-200 font-semibold placeholder:text-slate-400 dark:placeholder:text-zinc-600" 
              />
            </div>
          </div>
        {/if}
      </div>

      {#if isProductCodeUnmapped}
        <div class="absolute inset-0 bg-white/60 dark:bg-zinc-950/60 backdrop-blur-[1px] flex items-center justify-center rounded-xl z-10">
          <span class="text-rose-500 font-bold text-xs bg-rose-500/10 px-4 py-2 rounded-lg border border-rose-500/20">Chưa mapping Mã phụ tùng.</span>
        </div>
      {/if}
    </div>

    <!-- Tạo giá bán/giá vốn -->
    <div class="glass-panel p-6 shadow-sm space-y-4 relative overflow-hidden">
      <h2 class="text-xs font-bold text-slate-800 dark:text-slate-200 flex items-center gap-2 border-b border-slate-100 dark:border-zinc-800 pb-3">
        <RefreshCw size={14} class="text-blue-600 dark:text-cyan-400" />
        Tự động tạo giá vốn
      </h2>

      <div class="space-y-4 text-xs {(isCostPriceMapped || isRetailPriceUnmapped) ? 'opacity-40 pointer-events-none select-none' : ''}">
        <label class="flex items-center gap-3 p-3.5 input-glass hover:bg-slate-500/5 rounded-xl cursor-pointer">
          <input type="checkbox" bind:checked={file.generate_cost} disabled={isCostPriceMapped || isRetailPriceUnmapped} />
          <div class="space-y-0.5">
            <span class="font-bold text-slate-800 dark:text-slate-200 block text-xs">Tự động sinh giá vốn</span>
            <span class="text-[10px] text-slate-500 dark:text-slate-400 block">Tự động tính giá vốn từ giá bán lẻ theo chiết khấu %</span>
          </div>
        </label>

        {#if file.generate_cost && !isCostPriceMapped && !isRetailPriceUnmapped}
          <div 
            transition:slide
            class="p-4 border border-slate-200 dark:border-zinc-800 rounded-xl bg-white dark:bg-zinc-950 space-y-2"
          >
            <span class="block text-[10px] text-slate-500 dark:text-slate-400 font-bold mb-1">Phần trăm chiết khấu (%)</span>
            <div class="flex items-center input-glass rounded-lg px-3 py-1.5">
              <input 
                type="number" 
                bind:value={file.cost_discount_percent}
                placeholder="VD: 30" 
                class="w-full bg-transparent border-none text-xs focus:outline-none font-semibold text-slate-800 dark:text-slate-200 placeholder:text-slate-400 dark:placeholder:text-zinc-600" 
              />
              <span class="text-slate-500 dark:text-slate-450 text-xs font-bold">%</span>
            </div>
            <span class="block text-[9px] text-slate-500 dark:text-slate-400 mt-1.5 italic">Công thức: Giá vốn = Giá bán lẻ × (1 - {file.cost_discount_percent / 100})</span>
          </div>
        {/if}
      </div>

      {#if isCostPriceMapped}
        <div class="absolute inset-0 bg-white/60 dark:bg-zinc-950/60 backdrop-blur-[1px] flex items-center justify-center rounded-xl z-10">
          <span class="text-amber-600 dark:text-amber-400 font-bold text-xs bg-amber-500/10 px-4 py-2 rounded-lg border border-amber-500/20">Đã có Giá vốn (Lấy từ file)</span>
        </div>
      {:else if isRetailPriceUnmapped}
        <div class="absolute inset-0 bg-white/60 dark:bg-zinc-950/60 backdrop-blur-[1px] flex items-center justify-center rounded-xl z-10 p-6 text-center">
          <span class="text-rose-500 font-bold text-xs bg-rose-500/10 px-4 py-2 rounded-lg border border-rose-500/20 leading-relaxed block">Chưa chọn [Giá bán lẻ]. Cần mapping để tự động tạo Giá buôn.</span>
        </div>
      {/if}
    </div>

  </div>
{/if}
