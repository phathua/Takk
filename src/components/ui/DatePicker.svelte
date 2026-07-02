<script>
  import { Calendar, ChevronLeft, ChevronRight } from 'lucide-svelte';
  import { fade, scale } from 'svelte/transition';
  import Select from './Select.svelte';

  let { value = $bindable("") } = $props();

  let showDropdown = $state(false);
  let current = $state(new Date());

  // Định nghĩa danh sách các tháng và năm
  const monthOptions = Array.from({ length: 12 }, (_, i) => ({
    value: i,
    label: `Tháng ${i + 1}`
  }));

  // Tạo dải năm từ 2020 đến 2035 (hoặc +- 10 năm quanh năm hiện tại)
  const currentYear = new Date().getFullYear();
  const yearOptions = Array.from({ length: 20 }, (_, i) => {
    const y = currentYear - 10 + i;
    return { value: y, label: `${y}` };
  });

  let selectedMonthVal = $state(new Date().getMonth());
  let selectedYearVal = $state(new Date().getFullYear());

  // Đồng bộ từ current sang dropdown khi current thay đổi
  $effect(() => {
    selectedMonthVal = current.getMonth();
    selectedYearVal = current.getFullYear();
  });

  // Đồng bộ khi thay đổi dropdown tháng/năm
  $effect(() => {
    if (selectedMonthVal !== current.getMonth() || selectedYearVal !== current.getFullYear()) {
      current = new Date(selectedYearVal, selectedMonthVal, 1);
    }
  });

  // Trích xuất ngày hiện tại từ value
  $effect(() => {
    if (value) {
      const parts = value.split('/');
      if (parts.length === 3) {
        const d = parseInt(parts[0], 10);
        const m = parseInt(parts[1], 10) - 1;
        const y = parseInt(parts[2], 10);
        if (!isNaN(d) && !isNaN(m) && !isNaN(y)) {
          const parsed = new Date(y, m, d);
          if (!isNaN(parsed.getTime())) {
            current = parsed;
          }
        }
      }
    }
  });

  const getDaysInMonth = (year, month) => new Date(year, month + 1, 0).getDate();
  const getFirstDayOfMonth = (year, month) => {
    let day = new Date(year, month, 1).getDay();
    return day === 0 ? 6 : day - 1; // Bắt đầu từ Thứ Hai
  };

  const daysOfWeek = ['T2', 'T3', 'T4', 'T5', 'T6', 'T7', 'CN'];

  // Tạo mảng các ô lịch
  const calendarCells = $derived.by(() => {
    const year = current.getFullYear();
    const month = current.getMonth();
    const totalDays = getDaysInMonth(year, month);
    const startOffset = getFirstDayOfMonth(year, month);

    const cells = [];
    
    // Ngày tháng trước
    const prevMonth = month === 0 ? 11 : month - 1;
    const prevYear = month === 0 ? year - 1 : year;
    const prevMonthDays = getDaysInMonth(prevYear, prevMonth);
    for (let i = startOffset - 1; i >= 0; i--) {
      cells.push({
        day: prevMonthDays - i,
        month: prevMonth,
        year: prevYear,
        isCurrentMonth: false
      });
    }

    // Ngày tháng hiện tại
    for (let d = 1; d <= totalDays; d++) {
      cells.push({
        day: d,
        month,
        year,
        isCurrentMonth: true
      });
    }

    // Ngày tháng sau cho đủ bội số của 7 (đáp ứng 6 dòng x 7 ô = 42 ô)
    const remaining = 42 - cells.length;
    const nextMonth = month === 11 ? 0 : month + 1;
    const nextYear = month === 11 ? year + 1 : year;
    for (let d = 1; d <= remaining; d++) {
      cells.push({
        day: d,
        month: nextMonth,
        year: nextYear,
        isCurrentMonth: false
      });
    }

    return cells;
  });

  const selectDate = (cell) => {
    const pad = (n) => String(n).padStart(2, '0');
    value = `${pad(cell.day)}/${pad(cell.month + 1)}/${cell.year}`;
    showDropdown = false;
  };

  const prevMonth = () => {
    current = new Date(current.getFullYear(), current.getMonth() - 1, 1);
  };

  const nextMonth = () => {
    current = new Date(current.getFullYear(), current.getMonth() + 1, 1);
  };

  // Close dropdown khi bấm ra ngoài
  let containerRef;
  const handleWindowClick = (e) => {
    // Nếu click vào bên trong DatePicker thì không đóng
    if (containerRef && containerRef.contains(e.target)) {
      return;
    }
    
    // Nếu click trúng dropdown options của Select con (được render tuyệt đối, có thể nằm ngoài containerRef)
    // thì ta không đóng DatePicker để tránh xung đột
    if (e.target.closest('.z-50') || e.target.closest('button[type="button"]')) {
      return;
    }

    showDropdown = false;
  };
</script>

<svelte:window onclick={handleWindowClick} />

<div class="relative w-full" bind:this={containerRef}>
  <div class="flex items-center input-glass rounded-lg px-3 py-2">
    <button 
      type="button"
      class="hover:bg-slate-500/10 p-0.5 rounded cursor-pointer transition mr-1.5 shrink-0" 
      onclick={(e) => { e.stopPropagation(); showDropdown = !showDropdown; }}
    >
      <Calendar size={13} class="text-slate-500 dark:text-slate-400" />
    </button>
    <input 
      type="text" 
      bind:value={value}
      placeholder="DD/MM/YYYY"
      class="bg-transparent border-none text-xs w-full focus:outline-none text-slate-800 dark:text-slate-200 font-semibold font-mono placeholder:text-slate-400 dark:placeholder:text-zinc-650"
      onfocus={() => showDropdown = true}
    />
  </div>

  {#if showDropdown}
    <div 
      transition:scale={{ duration: 150, start: 0.95 }}
      class="absolute right-0 z-30 mt-1.5 w-[280px] bg-white dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded-xl shadow-2xl p-3 backdrop-blur-xl"
    >
      <!-- Header -->
      <div class="flex items-center justify-between mb-3">
        <button 
          onclick={prevMonth}
          class="p-1 hover:bg-slate-100 dark:hover:bg-zinc-800 rounded-md text-slate-600 dark:text-slate-400 transition"
        >
          <ChevronLeft size={16} />
        </button>
        <div class="flex items-center gap-1.5 max-w-[200px]">
          <Select 
            bind:value={selectedMonthVal} 
            options={monthOptions} 
            size="sm" 
            class="min-w-[90px]"
          />
          <Select 
            bind:value={selectedYearVal} 
            options={yearOptions} 
            size="sm" 
            class="min-w-[70px]"
          />
        </div>
        <button 
          onclick={nextMonth}
          class="p-1 hover:bg-slate-100 dark:hover:bg-zinc-800 rounded-md text-slate-600 dark:text-slate-400 transition"
        >
          <ChevronRight size={16} />
        </button>
      </div>

      <!-- Days of Week -->
      <div class="grid grid-cols-7 gap-1 text-center mb-1">
        {#each daysOfWeek as day}
          <span class="text-[9px] font-bold text-slate-400 dark:text-slate-500">{day}</span>
        {/each}
      </div>

      <!-- Calendar Grid -->
      <div class="grid grid-cols-7 gap-0.5">
        {#each calendarCells as cell}
          {@const isSelected = value === `${String(cell.day).padStart(2, '0')}/${String(cell.month + 1).padStart(2, '0')}/${cell.year}`}
          <button 
            onclick={() => selectDate(cell)}
            class="aspect-square text-[10px] font-semibold rounded-md transition flex items-center justify-center
              {cell.isCurrentMonth ? 'text-slate-800 dark:text-slate-200' : 'text-slate-400 dark:text-zinc-650'}
              {isSelected ? 'bg-blue-600 text-white dark:bg-cyan-500 dark:text-zinc-950 font-bold' : 'hover:bg-slate-100 dark:hover:bg-zinc-800/60'}"
          >
            {cell.day}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
