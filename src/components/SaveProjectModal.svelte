<script>
  import { fade, scale } from 'svelte/transition';
  import { HardDrive, Link, X, HelpCircle, ArrowRight, Check } from 'lucide-svelte';
  import { appState } from '../lib/state.svelte.js';

  // Lấy trạng thái lưu từ appState
  let dialog = $derived(appState.saveProjectDialog);
  let selectedFormat = $state('bgx'); // Mặc định chọn .bgx để quảng bá tiết kiệm dung lượng

  function handleCancel() {
    if (dialog.resolve) {
      dialog.resolve(null);
    }
  }

  function handleSelect() {
    if (dialog.resolve) {
      dialog.resolve(selectedFormat);
    }
  }
</script>

{#if dialog.show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    transition:fade={{ duration: 150 }}
    class="fixed inset-0 bg-slate-950/70 backdrop-blur-[4px] z-[999999] flex items-center justify-center p-4"
    onclick={handleCancel}
  >
    <div 
      transition:scale={{ duration: 200, start: 0.96 }}
      class="w-full max-w-2xl bg-[var(--card-bg)] border border-[var(--border)] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b border-[var(--border)] flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-lg bg-[var(--accent-glow)]/20 text-[var(--accent)] flex items-center justify-center">
            <HelpCircle size={16} />
          </div>
          <div>
            <h3 class="text-sm font-bold text-[var(--text)]">
              {dialog.saveAs ? 'Lưu thành dự án mới' : 'Lưu dự án mới'}
            </h3>
            <p class="text-[10px] text-[var(--text-muted)]">
              Chọn định dạng lưu trữ phù hợp với nhu cầu của bạn
            </p>
          </div>
        </div>
        <button 
          onclick={handleCancel}
          class="p-1.5 rounded-lg hover:bg-slate-500/10 text-[var(--text-muted)] hover:text-[var(--text)] transition cursor-pointer"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 overflow-y-auto space-y-6 flex-1">
        <!-- Ảnh minh họa trực quan & So sánh dung lượng -->
        <div class="relative bg-slate-500/5 border border-[var(--border)] rounded-xl overflow-hidden p-4 flex flex-col md:flex-row items-center gap-4">
          <div class="w-full md:w-1/2 flex items-center justify-center bg-white/5 dark:bg-black/20 rounded-lg p-2 overflow-hidden border border-zinc-500/10">
            <img 
              src="/file-association.webp" 
              alt="Đề xuất & Lịch sử gần đây" 
              class="max-h-[100px] object-contain rounded-md"
            />
          </div>
          <div class="w-full md:w-1/2 space-y-2 text-center md:text-left">
            <div class="text-[10px] uppercase tracking-wider font-bold text-[var(--accent)]">So sánh tối ưu dung lượng</div>
            <div class="flex items-center justify-center md:justify-start gap-2.5">
              <span class="text-xs text-slate-400 line-through">Dự án Đóng gói (.bg): 13.8 MB</span>
              <ArrowRight size={12} class="text-slate-400" />
              <span class="text-xs font-extrabold text-emerald-500 bg-emerald-500/10 px-2 py-0.5 rounded-md">
                Dự án Tham chiếu (.bgx): 1.1 KB
              </span>
            </div>
            <p class="text-[11px] text-[var(--text-muted)] leading-relaxed">
              Takk hỗ trợ hai cách thức tổ chức dự án khác nhau giúp bạn tối ưu hóa không gian lưu trữ và dễ dàng chia sẻ dữ liệu khi cần thiết.
            </p>
          </div>
        </div>

        <!-- Hai thẻ lựa chọn -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- Card BGX -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            onclick={() => selectedFormat = 'bgx'}
            class="group p-5 rounded-xl border-2 cursor-pointer transition-all duration-200 flex flex-col justify-between h-full relative select-none
              {selectedFormat === 'bgx' 
                ? 'border-[var(--accent)] bg-[var(--accent-glow)]/10 shadow-lg shadow-[var(--accent)]/5' 
                : 'border-[var(--border)] hover:border-zinc-500/30 bg-[var(--card-bg)] hover:bg-slate-500/5'}"
          >
            {#if selectedFormat === 'bgx'}
              <div class="absolute top-3 right-3 w-5 h-5 rounded-full bg-[var(--accent)] text-white flex items-center justify-center">
                <Check size={12} strokeWidth={3} />
              </div>
            {/if}

            <div class="space-y-3">
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg flex items-center justify-center
                  {selectedFormat === 'bgx' ? 'bg-[var(--accent)] text-white' : 'bg-slate-500/10 text-[var(--text-muted)] group-hover:text-[var(--text)]'}">
                  <Link size={16} />
                </div>
                <div>
                  <h4 class="text-xs font-bold text-[var(--text)]">Dự án Tham chiếu (.bgx)</h4>
                  <span class="text-[10px] text-emerald-500 font-semibold bg-emerald-500/10 px-1.5 py-0.5 rounded">Khuyên dùng</span>
                </div>
              </div>

              <p class="text-[11px] text-[var(--text-muted)] leading-relaxed">
                Chỉ ghi nhớ cấu hình bảng giá và liên kết đến các file Excel/CSV gốc trên máy tính. Dữ liệu thô không được đóng gói kèm.
              </p>

              <div class="space-y-1.5 pt-2">
                <div class="text-[10px] font-bold uppercase text-[var(--text)] tracking-wider">Ưu điểm chính:</div>
                <ul class="text-[11px] text-[var(--text-muted)] space-y-1 list-disc pl-4">
                  <li>Dung lượng siêu nhẹ (chỉ khoảng <strong class="text-[var(--text)]">1 KB</strong>)</li>
                  <li>Lưu và mở cực nhanh</li>
                  <li>Tránh trùng lặp file dữ liệu trên ổ cứng</li>
                </ul>
              </div>
            </div>

            <div class="mt-4 pt-3 border-t border-[var(--border)] text-[10px] text-[var(--text-muted)]">
              👉 <strong class="text-[var(--text)]">Phù hợp nhất:</strong> Làm việc hàng ngày trên máy tính cá nhân.
            </div>
          </div>

          <!-- Card BG -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            onclick={() => selectedFormat = 'bg'}
            class="group p-5 rounded-xl border-2 cursor-pointer transition-all duration-200 flex flex-col justify-between h-full relative select-none
              {selectedFormat === 'bg' 
                ? 'border-[var(--accent)] bg-[var(--accent-glow)]/10 shadow-lg shadow-[var(--accent)]/5' 
                : 'border-[var(--border)] hover:border-zinc-500/30 bg-[var(--card-bg)] hover:bg-slate-500/5'}"
          >
            {#if selectedFormat === 'bg'}
              <div class="absolute top-3 right-3 w-5 h-5 rounded-full bg-[var(--accent)] text-white flex items-center justify-center">
                <Check size={12} strokeWidth={3} />
              </div>
            {/if}

            <div class="space-y-3">
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg flex items-center justify-center
                  {selectedFormat === 'bg' ? 'bg-[var(--accent)] text-white' : 'bg-slate-500/10 text-[var(--text-muted)] group-hover:text-[var(--text)]'}">
                  <HardDrive size={16} />
                </div>
                <div>
                  <h4 class="text-xs font-bold text-[var(--text)]">Dự án Đóng gói (.bg)</h4>
                  <span class="text-[10px] text-amber-500 font-semibold bg-amber-500/10 px-1.5 py-0.5 rounded">Tự chứa dữ liệu</span>
                </div>
              </div>

              <p class="text-[11px] text-[var(--text-muted)] leading-relaxed">
                Đóng gói toàn bộ các file Excel/CSV thô trực tiếp vào bên trong tệp tin dự án. Bạn có thể mở nó ở bất kỳ đâu.
              </p>

              <div class="space-y-1.5 pt-2">
                <div class="text-[10px] font-bold uppercase text-[var(--text)] tracking-wider">Ưu điểm chính:</div>
                <ul class="text-[11px] text-[var(--text-muted)] space-y-1 list-disc pl-4">
                  <li>Không bao giờ sợ mất liên kết hoặc thiếu tệp tin</li>
                  <li>Dễ dàng di chuyển dự án giữa các máy tính khác nhau</li>
                  <li>Sao lưu toàn vẹn mọi dữ liệu đầu vào</li>
                </ul>
              </div>
            </div>

            <div class="mt-4 pt-3 border-t border-[var(--border)] text-[10px] text-[var(--text-muted)]">
              👉 <strong class="text-[var(--text)]">Phù hợp nhất:</strong> Chia sẻ cho đồng nghiệp, gửi email, chuyển máy làm việc.
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="px-6 py-4 bg-slate-500/5 border-t border-[var(--border)] flex items-center justify-end gap-3 shrink-0">
        <button 
          onclick={handleCancel}
          class="px-4 py-2 rounded-lg text-xs font-bold text-[var(--text-muted)] hover:text-[var(--text)] bg-transparent hover:bg-slate-500/10 cursor-pointer transition active:scale-95"
        >
          Hủy bỏ
        </button>
        <button 
          onclick={handleSelect}
          class="px-5 py-2 rounded-lg text-xs font-bold text-white bg-[var(--accent)] hover:bg-[var(--accent)]/90 cursor-pointer transition active:scale-95 shadow-md shadow-[var(--accent)]/10"
        >
          Xác nhận & Tiếp tục
        </button>
      </div>
    </div>
  </div>
{/if}
