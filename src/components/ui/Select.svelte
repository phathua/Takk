<script>
  import { ChevronDown } from 'lucide-svelte';
  import { onMount } from 'svelte';

  // Svelte 5 Runes Props
  let { 
    value = $bindable(), 
    options = [], // [{ value, label }]
    placeholder = 'Chọn...',
    class: customClass = '',
    size = 'md'
  } = $props();

  let isOpen = $state(false);
  let selectEl = $state(null);
  let openUpward = $state(false);

  const selectedOption = $derived(
    options.find(opt => opt.value === value) || null
  );

  const toggleDropdown = (e) => {
    if (isOpen) {
      isOpen = false;
    } else {
      const rect = selectEl.getBoundingClientRect();
      const spaceBelow = window.innerHeight - rect.bottom;
      openUpward = spaceBelow < 200; // Neu duoi 200px thi mo nguoc len tren
      isOpen = true;
    }
  };

  const selectOption = (optValue) => {
    value = optValue;
    isOpen = false;
  };

  // Dong khi click ra ngoai
  onMount(() => {
    const handleClickOutside = (e) => {
      if (selectEl && !selectEl.contains(e.target)) {
        isOpen = false;
      }
    };
    window.addEventListener('click', handleClickOutside);
    return () => {
      window.removeEventListener('click', handleClickOutside);
    };
  });
</script>

<div bind:this={selectEl} class="relative w-full inline-block text-left {customClass}">
  <button
    type="button"
    onclick={toggleDropdown}
    class="w-full flex items-center justify-between input-glass cursor-pointer hover:bg-slate-500/5 transition {size === 'sm' ? 'rounded-md px-2 py-1 text-[10px]' : 'rounded-xl px-3.5 py-2 text-xs'} text-[var(--text)] font-semibold"
  >
    <div class="flex items-center gap-2 truncate">
      {#if selectedOption && selectedOption.icon}
        {@const Icon = selectedOption.icon}
        <Icon size={12} class={selectedOption.iconColor || ''} />
      {/if}
      <span class="truncate">{selectedOption ? selectedOption.label : placeholder}</span>
    </div>
    <ChevronDown size={12} class="text-[var(--text-muted)] ml-2 shrink-0 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div
      class="absolute z-50 w-full mt-1 bg-[var(--card-bg)] border border-[var(--border)] shadow-2xl overflow-hidden max-h-72 overflow-y-auto backdrop-blur-xl
        {size === 'sm' ? 'rounded-md' : 'rounded-xl'}
        {openUpward ? 'bottom-full mb-1' : 'top-full mt-1'}"
    >
      {#if options.length === 0}
        <div class="px-4 py-2 text-xs text-[var(--text-muted)] italic">Không có lựa chọn nào</div>
      {:else}
        {#each options as opt}
          <button
            type="button"
            onclick={() => selectOption(opt.value)}
            class="w-full text-left px-4 py-2 text-xs hover:bg-[var(--accent)] hover:text-white transition font-semibold flex items-center justify-between group
              {opt.value === value ? 'bg-[var(--accent)]/10 text-[var(--accent)]' : 'text-[var(--text)]'}"
          >
            <div class="flex items-center gap-2 truncate">
              {#if opt.icon}
                {@const Icon = opt.icon}
                <Icon size={12} class="{(opt.iconColor || '')} group-hover:text-white transition-colors duration-150" />
              {/if}
              <span>{opt.label}</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>
