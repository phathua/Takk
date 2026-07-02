<script>
  import { CheckCircle, AlertTriangle, AlertCircle, Info, X } from 'lucide-svelte';
  import { fly } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { appState } from '../../lib/state.svelte.js';
</script>

{#if appState.toasts.length > 0}
  <div class="fixed top-6 right-6 z-50 flex flex-col gap-3 max-w-sm pointer-events-none">
    {#each appState.toasts as toast (toast.id)}
      <div 
        transition:fly={{ x: 200, y: 0, duration: 300 }}
        animate:flip={{ duration: 250 }}
        class="glass-panel border border-[var(--border)] shadow-xl rounded-xl overflow-hidden flex items-stretch min-w-[320px] max-w-sm pointer-events-auto"
      >
        <!-- Left vertical color accent bar -->
        <div class="w-1.5 shrink-0
          {toast.type === 'success' ? 'bg-emerald-500' : ''}
          {toast.type === 'error' ? 'bg-rose-500' : ''}
          {toast.type === 'warning' ? 'bg-amber-500' : ''}
          {toast.type === 'info' || toast.type === 'debug' ? 'bg-blue-500' : ''}"
        ></div>

        <!-- Content Area -->
        <div class="flex-grow p-3.5 flex items-center justify-between gap-3 min-w-0">
          <div class="flex items-center gap-3 min-w-0 flex-1">
            <!-- Icon -->
            <div class="shrink-0 flex items-center justify-center">
              {#if toast.type === 'success'}<CheckCircle size={18} class="text-emerald-500" />{/if}
              {#if toast.type === 'error'}<AlertCircle size={18} class="text-rose-500" />{/if}
              {#if toast.type === 'warning'}<AlertTriangle size={18} class="text-amber-500" />{/if}
              {#if toast.type === 'info' || toast.type === 'debug'}<Info size={18} class="text-blue-500" />{/if}
            </div>

            <!-- Message Text -->
            <span class="text-[11px] font-semibold text-[var(--text)] leading-relaxed select-text break-all flex-1 min-w-0">{toast.text}</span>
          </div>
          
          <!-- Close Button -->
          <button 
            onclick={() => appState.toasts = appState.toasts.filter(t => t.id !== toast.id)}
            class="shrink-0 p-1.5 hover:bg-slate-500/10 rounded-lg cursor-pointer text-[var(--text-muted)] hover:text-[var(--text)] transition-colors duration-150"
          >
            <X size={13} />
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
