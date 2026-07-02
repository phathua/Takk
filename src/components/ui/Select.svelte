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
  // Action svelte giúp tự động đo và cập nhật trạng thái tràn chữ (overflow) thực tế của phần tử DOM
  function checkOverflow(node, text) {
    const update = () => {
      // Cho một chút thời gian để layout ổn định
      requestAnimationFrame(() => {
        const isOverflow = node.scrollWidth > node.clientWidth;
        if (isOverflow) {
          node.classList.add('should-marquee');
          node.style.setProperty('--marquee-width', `${node.scrollWidth}px`);
          node.style.setProperty('--container-width', `${node.clientWidth}px`);
        } else {
          node.classList.remove('should-marquee');
          node.style.removeProperty('--marquee-width');
          node.style.removeProperty('--container-width');
        }
      });
    };

    update();
    
    return {
      update() {
        update();
      }
    };
  }
</script>

<div bind:this={selectEl} class="relative w-full inline-block text-left {customClass}">
  <button
    type="button"
    onclick={toggleDropdown}
    class="w-full flex items-center justify-between input-glass cursor-pointer hover:bg-slate-500/5 transition {size === 'sm' ? 'rounded-md px-2 py-1 text-[10px]' : 'rounded-xl px-3.5 py-2 text-xs'} text-[var(--text)] font-semibold"
  >
    <div class="flex items-center gap-2 overflow-hidden flex-1 select-none">
      {#if selectedOption && selectedOption.icon}
        {@const Icon = selectedOption.icon}
        <Icon size={12} class={selectedOption.iconColor || ''} />
      {/if}
      <div class="relative overflow-hidden w-full h-4 flex items-center select-none text-left container-marquee">
        <span 
          use:checkOverflow={selectedOption ? selectedOption.label : placeholder}
          class="whitespace-nowrap transition-transform duration-[2000ms] ease-in-out block text-marquee"
        >
          {selectedOption ? selectedOption.label : placeholder}
        </span>
      </div>
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
            <div class="flex items-center gap-2 overflow-hidden flex-1 select-none container-marquee">
              {#if opt.icon}
                {@const Icon = opt.icon}
                <Icon size={12} class="{(opt.iconColor || '')} group-hover:text-white transition-colors duration-150" />
              {/if}
              <span 
                use:checkOverflow={opt.label}
                class="whitespace-nowrap transition-transform duration-[2000ms] ease-in-out block text-marquee"
              >
                {opt.label}
              </span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .container-marquee {
    display: flex;
    align-items: center;
    width: 100%;
    overflow: hidden;
  }

  .text-marquee {
    display: inline-block;
    white-space: nowrap;
    width: max-content;
    min-w: 100%;
    text-overflow: ellipsis;
    overflow: hidden;
  }

  @keyframes marquee-scroll {
    0%, 15% {
      transform: translateX(0);
    }
    50%, 65% {
      /* translate -100% là dịch sang trái toàn bộ chiều dài chữ. +100% của container cha (được truyền qua biến CSS) */
      /* Sẽ giúp dịch chuyển chính xác đến điểm dừng cuối cùng của chữ mà không bị tràn */
      transform: translateX(calc(-100% + var(--container-width, 140px) - 16px));
    }
    95%, 100% {
      transform: translateX(0);
    }
  }

  /* Chỉ kích hoạt animation chạy chữ khi hover nếu text dài hơn khung hiển thị (có class should-marquee được add động qua JS) */
  button:hover .text-marquee:global(.should-marquee) {
    animation: marquee-scroll 4.5s ease-in-out infinite;
    text-overflow: clip;
    overflow: visible;
  }
</style>
