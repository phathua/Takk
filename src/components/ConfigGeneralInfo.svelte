<script>
  import { Info, Tag, Building2, Calendar } from 'lucide-svelte';
  import { fly } from 'svelte/transition';
  import { appState } from '../lib/state.svelte.js';

  function selectMapping(file, mapping) {
    file.brand = mapping.brand;
    file.provider = mapping.provider;
    if (mapping.provider.toLowerCase().includes("minh đông")) {
      file.normalize_special = true;
      file.normalize_position = "Suffix";
      if (!file.normalize_suffix) {
        file.normalize_suffix = "M";
      }
    }
  }
</script>

{#if appState.currentFile}
  {@const file = appState.currentFile}
  <!-- THÔNG TIN CHUNG TỆP -->
  <div 
    in:fly={{ y: 20, duration: 350 }}
    class="glass-panel p-6 shadow-sm space-y-5"
  >
    <h2 class="text-xs font-bold text-slate-800 dark:text-slate-200 flex items-center gap-2 border-b border-slate-100 dark:border-zinc-800 pb-3">
      <Info size={14} class="text-blue-600 dark:text-cyan-400" />
      Thông tin Hãng & Nhà cung cấp
    </h2>

    <div class="grid grid-cols-3 gap-6">
      <!-- Nhập Hãng -->
      <div class="relative">
        <label for="brand-input" class="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase mb-1.5 tracking-wider">Hãng (*)</label>
        <div class="flex items-center input-glass rounded-lg px-3 py-2">
          <Tag size={13} class="text-slate-500 dark:text-slate-400 mr-2 shrink-0" />
          <input 
            id="brand-input"
            type="text" 
            bind:value={file.brand}
            onfocus={() => appState.brandInputFocus = true}
            onblur={() => setTimeout(() => appState.brandInputFocus = false, 200)}
            placeholder="Nhập hoặc chọn hãng..."
            class="bg-transparent border-none text-xs w-full focus:outline-none text-slate-800 dark:text-slate-200 font-semibold placeholder:text-slate-400 dark:placeholder:text-zinc-650"
          />
        </div>
        {#if appState.brandInputFocus && appState.filteredBrands.length > 0}
          <div class="absolute z-20 w-full mt-1 bg-white dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg shadow-xl overflow-hidden max-h-60 overflow-y-auto backdrop-blur-xl">
            {#each appState.filteredBrands as mapping}
              <button 
                onclick={() => selectMapping(file, mapping)}
                class="w-full text-left px-4 py-2 text-xs hover:bg-blue-600 hover:text-white dark:hover:bg-cyan-500 dark:hover:text-zinc-950 transition font-semibold text-slate-800 dark:text-slate-200"
              >
                {mapping.brand}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Nhập Nhà Cung Cấp -->
      <div class="relative">
        <label for="provider-input" class="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase mb-1.5 tracking-wider">Nhà cung cấp (*)</label>
        <div class="flex items-center input-glass rounded-lg px-3 py-2">
          <Building2 size={13} class="text-slate-500 dark:text-slate-400 mr-2 shrink-0" />
          <input 
            id="provider-input"
            type="text" 
            bind:value={file.provider}
            onfocus={() => appState.providerInputFocus = true}
            onblur={() => setTimeout(() => appState.providerInputFocus = false, 200)}
            placeholder="Nhập NCC..."
            class="bg-transparent border-none text-xs w-full focus:outline-none text-slate-800 dark:text-slate-200 font-semibold placeholder:text-slate-400 dark:placeholder:text-zinc-650"
          />
        </div>
        {#if appState.providerInputFocus && appState.filteredProviders.length > 0}
          <div class="absolute z-20 w-full mt-1 bg-white dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-lg shadow-xl overflow-hidden max-h-60 overflow-y-auto backdrop-blur-xl">
            {#each appState.filteredProviders as mapping}
              <button 
                onclick={() => selectMapping(file, mapping)}
                class="w-full text-left px-4 py-2 text-xs hover:bg-blue-600 hover:text-white dark:hover:bg-cyan-500 dark:hover:text-zinc-950 transition font-semibold text-slate-800 dark:text-slate-200 group"
              >
                {mapping.provider} <span class="text-slate-500 dark:text-slate-400 text-[10px] font-normal group-hover:text-blue-100 dark:group-hover:text-cyan-950">(của {mapping.brand})</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Ngày Tạo -->
      <div>
        <label for="created-at-input" class="block text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase mb-1.5 tracking-wider">Ngày tạo</label>
        <div class="flex items-center input-glass rounded-lg px-3 py-2">
          <Calendar size={13} class="text-slate-500 dark:text-slate-400 mr-2 shrink-0" />
          <input 
            id="created-at-input"
            type="text" 
            bind:value={file.created_at}
            placeholder="DD/MM/YYYY"
            class="bg-transparent border-none text-xs w-full focus:outline-none text-slate-800 dark:text-slate-200 font-semibold font-mono placeholder:text-slate-400 dark:placeholder:text-zinc-650"
          />
        </div>
      </div>
    </div>
  </div>
{/if}
