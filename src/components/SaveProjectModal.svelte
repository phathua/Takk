<script>
  import { fade, scale } from 'svelte/transition';
  import { HardDrive, Link, X, HelpCircle, ArrowRight, Check } from 'lucide-svelte';
  import { appState } from '../lib/state.svelte.js';

  // Lấy trạng thái lưu từ appState
  let dialog = $derived(appState.saveProjectDialog);
  let selectedFormat = $state('bgx'); // Mặc định chọn .bgx để quảng bá tiết kiệm dung lượng
  let dontShowAgain = $state(false);

  function handleCancel() {
    if (dialog.resolve) {
      dialog.resolve(null);
    }
  }

  function handleSelect() {
    if (dontShowAgain) {
      localStorage.setItem('takk_skip_save_format_explanation', 'true');
      localStorage.setItem('takk_preferred_save_format', selectedFormat);
    }
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
      class="w-full max-w-xl bg-[var(--card-bg)] border border-[var(--border)] rounded-2xl shadow-2xl overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-[var(--border)] flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-8 h-8 rounded-lg bg-[var(--accent-glow)]/20 text-[var(--accent)] flex items-center justify-center shrink-0">
            <HelpCircle size={16} />
          </div>
          <div>
            <h3 class="text-xs font-bold text-[var(--text)]">
              {dialog.saveAs ? 'Lưu thành dự án mới' : 'Lưu dự án mới'}
            </h3>
            <p class="text-[10px] text-[var(--text-muted)]">
              Chọn định dạng phù hợp với nhu cầu sử dụng của bạn
            </p>
          </div>
        </div>
        <button 
          onclick={handleCancel}
          class="p-1 rounded-lg hover:bg-slate-500/10 text-[var(--text-muted)] hover:text-[var(--text)] transition cursor-pointer"
        >
          <X size={14} />
        </button>
      </div>

      <!-- Content -->
      <div class="p-5 space-y-4">
        <!-- So sánh nhanh dùng ảnh file-association.webp cực kỳ nhỏ gọn -->
        <div class="flex items-center gap-3 p-3 bg-slate-500/5 border border-[var(--border)] rounded-xl">
          <img 
            src="/file-association.webp" 
            alt="Files" 
            class="h-8 object-contain rounded border border-zinc-500/10 bg-white/5 p-0.5 shrink-0"
          />
          <div class="text-[11px] leading-relaxed text-[var(--text-muted)]">
            Định dạng <strong class="text-[var(--accent)]">Tham chiếu (.bgx)</strong> giúp giảm dung lượng tệp lưu trữ từ <span class="line-through">13.8 MB</span> xuống còn <strong class="text-emerald-500">1.1 KB</strong> bằng cách liên kết file thay vì nhân bản dữ liệu.
          </div>
        </div>

        <!-- Hai thẻ lựa chọn -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
          <!-- Card BGX -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            onclick={() => selectedFormat = 'bgx'}
            class="group p-4 rounded-xl border-2 cursor-pointer transition-all duration-200 flex flex-col justify-between relative select-none
              {selectedFormat === 'bgx' 
                ? 'border-[var(--accent)] bg-[var(--accent-glow)]/10 shadow-lg shadow-[var(--accent)]/5' 
                : 'border-[var(--border)] hover:border-zinc-500/30 bg-[var(--card-bg)] hover:bg-slate-500/5'}"
          >
            {#if selectedFormat === 'bgx'}
              <div class="absolute top-2.5 right-2.5 w-4 h-4 rounded-full bg-[var(--accent)] text-white flex items-center justify-center">
                <Check size={10} strokeWidth={3} />
              </div>
            {/if}

            <div class="space-y-2.5">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0
                  {selectedFormat === 'bgx' ? 'bg-[var(--accent)] text-white' : 'bg-slate-500/10 text-[var(--text-muted)] group-hover:text-[var(--text)]'}">
                  <Link size={14} />
                </div>
                <div>
                  <h4 class="text-[11px] font-bold text-[var(--text)]">Dự án Tham chiếu (.bgx)</h4>
                  <div class="flex items-center gap-1.5 mt-0.5">
                    <span class="text-[9px] text-emerald-500 font-bold bg-emerald-500/10 px-1 py-0.2 rounded">Khuyên dùng</span>
                    <span class="text-[9px] text-[var(--text-muted)] font-mono">~1 KB</span>
                  </div>
                </div>
              </div>

              <p class="text-[10px] text-[var(--text-muted)] leading-relaxed">
                Chỉ lưu cấu hình mapping và liên kết đường dẫn tệp. Yêu cầu tệp Excel/CSV gốc phải giữ nguyên vị trí trên máy.
              </p>

              <div class="space-y-1.5">
                <div class="text-[9px] font-bold uppercase text-[var(--text)] tracking-wider">Đặc điểm chính:</div>
                <ul class="text-[10px] text-[var(--text-muted)] space-y-1 list-disc pl-3">
                  <li>Kích thước cực nhỏ (<strong class="text-[var(--text)]">~1.1 KB</strong>)</li>
                  <li>Lưu và tải ngay lập tức</li>
                  <li>Không nhân bản file Excel gốc</li>
                </ul>
              </div>
            </div>

            <div class="mt-3.5 pt-2.5 border-t border-[var(--border)] text-[9px] text-[var(--text-muted)]">
              💡 <strong class="text-[var(--text)]">Phù hợp nhất:</strong> Làm việc cá nhân cố định trên máy.
            </div>
          </div>

          <!-- Card BG -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            onclick={() => selectedFormat = 'bg'}
            class="group p-4 rounded-xl border-2 cursor-pointer transition-all duration-200 flex flex-col justify-between relative select-none
              {selectedFormat === 'bg' 
                ? 'border-[var(--accent)] bg-[var(--accent-glow)]/10 shadow-lg shadow-[var(--accent)]/5' 
                : 'border-[var(--border)] hover:border-zinc-500/30 bg-[var(--card-bg)] hover:bg-slate-500/5'}"
          >
            {#if selectedFormat === 'bg'}
              <div class="absolute top-2.5 right-2.5 w-4 h-4 rounded-full bg-[var(--accent)] text-white flex items-center justify-center">
                <Check size={10} strokeWidth={3} />
              </div>
            {/if}

            <div class="space-y-2.5">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0
                  {selectedFormat === 'bg' ? 'bg-[var(--accent)] text-white' : 'bg-slate-500/10 text-[var(--text-muted)] group-hover:text-[var(--text)]'}">
                  <HardDrive size={14} />
                </div>
                <div>
                  <h4 class="text-[11px] font-bold text-[var(--text)]">Dự án Đóng gói (.bg)</h4>
                  <div class="flex items-center gap-1.5 mt-0.5">
                    <span class="text-[9px] text-amber-500 font-bold bg-amber-500/10 px-1 py-0.2 rounded">Tự chứa dữ liệu</span>
                    <span class="text-[9px] text-[var(--text-muted)] font-mono">~13.8 MB</span>
                  </div>
                </div>
              </div>

              <p class="text-[10px] text-[var(--text-muted)] leading-relaxed">
                Đóng gói bản sao của toàn bộ file Excel/CSV gốc trực tiếp bên trong tệp dự án. Dự án hoàn toàn độc lập.
              </p>

              <div class="space-y-1.5">
                <div class="text-[9px] font-bold uppercase text-[var(--text)] tracking-wider">Đặc điểm chính:</div>
                <ul class="text-[10px] text-[var(--text-muted)] space-y-1 list-disc pl-3">
                  <li>Không bao giờ sợ mất liên kết tệp</li>
                  <li>Dễ dàng di chuyển dự án qua máy khác</li>
                  <li>Sao lưu độc lập an toàn</li>
                </ul>
              </div>
            </div>

            <div class="mt-3.5 pt-2.5 border-t border-[var(--border)] text-[9px] text-[var(--text-muted)]">
              💡 <strong class="text-[var(--text)]">Phù hợp nhất:</strong> Gửi email, chia sẻ cho người khác.
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="px-5 py-3 bg-slate-500/5 border-t border-[var(--border)] flex items-center justify-between gap-3 shrink-0">
        <label class="flex items-center gap-2 cursor-pointer select-none text-[10px] text-[var(--text-muted)] hover:text-[var(--text)]">
          <input 
            type="checkbox" 
            bind:checked={dontShowAgain}
            class="accent-[var(--accent)] cursor-pointer w-3 h-3"
          />
          Không nhắc lại lựa chọn này
        </label>
        <div class="flex items-center gap-2.5">
          <button 
            onclick={handleCancel}
            class="px-3 py-1.5 rounded-lg text-xs font-bold text-[var(--text-muted)] hover:text-[var(--text)] bg-transparent hover:bg-slate-500/10 cursor-pointer transition active:scale-95"
          >
            Hủy bỏ
          </button>
          <button 
            onclick={handleSelect}
            class="px-4 py-1.5 rounded-lg text-xs font-bold text-white bg-[var(--accent)] hover:bg-[var(--accent)]/90 cursor-pointer transition active:scale-95 shadow-md shadow-[var(--accent)]/10"
          >
            Xác nhận & Tiếp tục
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
