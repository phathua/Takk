<script>
  import { fade, scale } from 'svelte/transition';
  import { AlertTriangle, Info, HelpCircle, X } from 'lucide-svelte';
  import { appState } from '../lib/state.svelte.js';

  // Lấy trạng thái dialog từ appState
  let dialog = $derived(appState.confirmDialog);

  function handleCancel() {
    if (dialog.resolve) {
      dialog.resolve(false);
    }
  }

  function handleConfirm() {
    if (dialog.resolve) {
      dialog.resolve(true);
    }
  }
</script>

{#if dialog.show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    transition:fade={{ duration: 150 }}
    class="fixed inset-0 bg-slate-950/60 backdrop-blur-[2px] z-[999999] flex items-center justify-center p-4"
    onclick={handleCancel}
  >
    <div 
      transition:scale={{ duration: 180, start: 0.95 }}
      class="w-full max-w-md bg-[var(--card-bg)] border border-[var(--border)] rounded-2xl shadow-2xl overflow-hidden"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header / Icon -->
      <div class="p-6 flex items-start gap-4">
        {#if dialog.kind === 'danger' || dialog.kind === 'warning'}
          <div class="w-10 h-10 rounded-full bg-rose-500/10 dark:bg-rose-500/20 text-rose-500 flex items-center justify-center shrink-0">
            <AlertTriangle size={20} />
          </div>
        {:else if dialog.kind === 'info'}
          <div class="w-10 h-10 rounded-full bg-blue-500/10 dark:bg-blue-500/20 text-blue-500 flex items-center justify-center shrink-0">
            <Info size={20} />
          </div>
        {:else}
          <div class="w-10 h-10 rounded-full bg-amber-500/10 dark:bg-amber-500/20 text-amber-500 flex items-center justify-center shrink-0">
            <HelpCircle size={20} />
          </div>
        {/if}

        <div class="space-y-1.5 flex-1 min-w-0">
          <h3 class="text-sm font-bold text-[var(--text)] truncate">
            {dialog.title}
          </h3>
          <p class="text-xs text-[var(--text-muted)] leading-relaxed whitespace-pre-wrap pr-2">
            {dialog.message}
          </p>
        </div>

        <button 
          onclick={handleCancel}
          class="p-1 hover:bg-slate-500/10 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] transition cursor-pointer self-start"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Action Buttons -->
      <div class="px-6 py-4 bg-slate-500/5 border-t border-[var(--border)] flex items-center justify-end gap-3">
        {#if dialog.buttons && dialog.buttons.length > 0}
          {#each dialog.buttons as btn}
            <button 
              onclick={() => dialog.resolve(btn.value)}
              class="px-4 py-2 rounded-lg text-xs font-bold cursor-pointer transition active:scale-95 {btn.class || 'text-[var(--text-muted)] hover:text-[var(--text)] bg-transparent hover:bg-slate-500/10'}"
            >
              {btn.text}
            </button>
          {/each}
        {:else}
          <button 
            onclick={handleCancel}
            class="px-4 py-2 rounded-lg text-xs font-bold text-[var(--text-muted)] hover:text-[var(--text)] bg-transparent hover:bg-slate-500/10 cursor-pointer transition active:scale-95"
          >
            {dialog.cancelText}
          </button>
          <button 
            onclick={handleConfirm}
            class="px-4 py-2 rounded-lg text-xs font-bold text-white cursor-pointer transition active:scale-95
              {dialog.kind === 'danger' ? 'bg-rose-500 hover:bg-rose-600' : 'bg-[var(--accent)] hover:bg-[var(--accent)]/90'}"
          >
            {dialog.confirmText}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
