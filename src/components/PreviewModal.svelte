<script>
  import { fade, scale } from 'svelte/transition';
  import { Eye, X, RefreshCw, Play } from 'lucide-svelte';

  // Nhận props từ bên ngoài
  let { 
    show = $bindable(false),
    previewRows = [],
    isGeneratingPreview = false,
    previewError = null,
    formatCurrency,
    onProceed
  } = $props();

  const handleProceed = () => {
    show = false;
    if (onProceed) onProceed();
  };
</script>

{#if show}
  <div 
    transition:fade={{ duration: 200 }}
    class="fixed inset-0 bg-slate-950/60 backdrop-blur-sm z-[99999] flex items-center justify-center p-4"
  >
    <div 
      transition:scale={{ duration: 250, start: 0.95 }}
      class="w-full max-w-6xl bg-[var(--card-bg)] border border-[var(--border)] rounded-2xl shadow-2xl flex flex-col max-h-[85vh] overflow-hidden"
    >
      <!-- Modal Header -->
      <div class="p-6 border-b border-[var(--border)] flex items-center justify-between shrink-0">
        <div class="space-y-1">
          <h3 class="text-sm font-bold text-[var(--text)] uppercase tracking-wider flex items-center gap-2">
            <Eye size={16} class="text-[var(--accent)]" />
            Xem trước dữ liệu gộp
          </h3>
          <p class="text-xs text-[var(--text-muted)]">Xem trước tối đa 10 dòng đầu của từng file đã cấu hình để kiểm tra kết quả chuẩn hóa & mapping.</p>
        </div>
        <button 
          onclick={() => show = false}
          class="p-2 hover:bg-slate-500/10 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] cursor-pointer transition"
        >
          <X size={18} />
        </button>
      </div>

      <!-- Modal Content (Table) -->
      <div class="flex-1 overflow-y-auto p-6 min-h-[300px] flex flex-col justify-center">
        {#if isGeneratingPreview}
          <div class="flex flex-col items-center justify-center py-12 space-y-3">
            <RefreshCw size={24} class="text-[var(--accent)] animate-spin" />
            <span class="text-xs text-[var(--text-muted)] font-medium">Đang xử lý dữ liệu xem trước...</span>
          </div>
        {:else if previewError}
          <div class="flex flex-col items-center justify-center py-12 text-center max-w-md mx-auto space-y-3">
            <div class="w-12 h-12 rounded-full bg-rose-500/10 text-rose-500 flex items-center justify-center">
              <X size={20} />
            </div>
            <h4 class="text-xs font-bold text-[var(--text)]">Không thể tải bản xem trước</h4>
            <p class="text-xs text-[var(--text-muted)] leading-relaxed">{previewError}</p>
          </div>
        {:else if previewRows.length === 0}
          <div class="flex flex-col items-center justify-center py-12 text-center max-w-sm mx-auto space-y-2">
            <span class="text-xs text-[var(--text-muted)] italic">Không tìm thấy dữ liệu hợp lệ để hiển thị.</span>
          </div>
        {:else}
          <div class="w-full flex-grow overflow-auto border border-[var(--border)] rounded-xl bg-[var(--background)]">
            <table class="w-full text-left border-collapse text-xs">
              <thead>
                <tr class="bg-[var(--card-bg)] text-[10px] font-bold text-[var(--text-muted)] uppercase tracking-wider border-b border-[var(--border)] sticky top-0 z-10">
                  <th class="p-3 bg-[var(--card-bg)]">Mã sản phẩm</th>
                  <th class="p-3 bg-[var(--card-bg)]">Mã cũ/thay thế</th>
                  <th class="p-3 bg-[var(--card-bg)]">Tên sản phẩm</th>
                  <th class="p-3 bg-[var(--card-bg)]">Hãng</th>
                  <th class="p-3 bg-[var(--card-bg)]">Nhà cung cấp</th>
                  <th class="p-3 text-right bg-[var(--card-bg)]">Giá vốn</th>
                  <th class="p-3 text-right bg-[var(--card-bg)]">Giá bán lẻ</th>
                  <th class="p-3 bg-[var(--card-bg)]">Đời xe</th>
                  <th class="p-3 bg-[var(--card-bg)]">Mã màu</th>
                  <th class="p-3 bg-[var(--card-bg)]">Ghi chú</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-[var(--border)]">
                {#each previewRows as row, i}
                  <tr class="hover:bg-slate-500/5 transition text-[var(--text)] font-semibold {i % 2 === 0 ? 'bg-transparent' : 'bg-slate-500/[0.02]' }">
                    <td class="p-3 font-mono text-[var(--accent)] font-bold">{row.product_code}</td>
                    <td class="p-3 text-[var(--text-muted)]">{row.alt_code || '-'}</td>
                    <td class="p-3 truncate max-w-[200px]" title={row.name}>{row.name}</td>
                    <td class="p-3">
                      <span class="px-1.5 py-0.5 rounded text-[9px] font-bold bg-zinc-500/10 text-[var(--text-muted)]">{row.brand}</span>
                    </td>
                    <td class="p-3 text-[var(--text-muted)]">{row.provider}</td>
                    <td class="p-3 text-right font-mono text-emerald-600 dark:text-emerald-400 font-bold">{formatCurrency(row.cost_price)}</td>
                    <td class="p-3 text-right font-mono text-slate-700 dark:text-slate-300">{formatCurrency(row.retail_price)}</td>
                    <td class="p-3 text-[var(--text-muted)]">{row.model || '-'}</td>
                    <td class="p-3 text-[var(--text-muted)]">{row.color_code || '-'}</td>
                    <td class="p-3 text-[var(--text-muted)] truncate max-w-[150px]" title={row.note || ''}>{row.note || '-'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="p-6 border-t border-[var(--border)] flex items-center justify-between shrink-0 bg-slate-500/5">
        <div class="text-xs text-[var(--text-muted)] font-semibold">
          {#if !isGeneratingPreview && !previewError && previewRows.length > 0}
            Tổng số dòng xem trước: <span class="text-[var(--text)] font-bold">{previewRows.length} dòng</span>
          {/if}
        </div>
        <div class="flex items-center gap-3">
          <button 
            onclick={() => show = false}
            class="btn-ghost text-xs font-bold px-4 py-2 rounded-md cursor-pointer transition active:scale-95"
          >
            HỦY / ĐIỀU CHỈNH LẠI
          </button>
          <button 
            onclick={handleProceed}
            disabled={isGeneratingPreview || !!previewError || previewRows.length === 0}
            class="btn-primary flex items-center gap-2 text-white font-bold text-xs px-4 py-2 rounded-md cursor-pointer transition active:scale-95 disabled:opacity-40 disabled:pointer-events-none"
          >
            <Play size={12} fill="currentColor" /> TIẾN HÀNH XỬ LÝ
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
