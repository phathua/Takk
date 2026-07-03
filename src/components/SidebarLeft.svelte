<script>
  import {
    LayoutDashboard,
    Database,
    Sun,
    Moon,
    Laptop,
    RefreshCw,
  } from "lucide-svelte";
  import { appState } from "../lib/state.svelte.js";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { listen } from "@tauri-apps/api/event";
  import HungTayGaLogo from "./icons/HungTayGaLogo.svelte";

  let isChecking = $state(false);

  export async function checkUpdate(silent = false) {
    if (isChecking) return;
    isChecking = true;
    let infoToastId = null;
    if (!silent) {
      infoToastId = appState.addLog(
        "Info",
        "Đang kiểm tra phiên bản mới trên GitHub...",
      );
    }
    let progressUnsubscribe = null;
    try {
      const result = await invoke("check_for_updates");
      if (infoToastId) {
        appState.removeToast(infoToastId);
      }
      if (result.has_update) {
        appState.addLog(
          "Success",
          `Phát hiện phiên bản mới: ${result.latest_version}`,
        );

        if (!result.download_url) {
          appState.addLog(
            "Warning",
            "Không tìm thấy file cài đặt (.exe) trong bản phát hành.",
          );
          alert(
            `Phát hiện phiên bản mới ${result.latest_version} nhưng không tìm thấy file cài đặt .exe. Bạn có thể tải thủ công tại website.`,
          );
          await openUrl(result.url);
          return;
        }

        const confirmGo = confirm(
          `Phát hiện phiên bản mới: ${result.latest_version}.\nHệ thống sẽ tự động tải về và chạy cài đặt cập nhật. Bạn muốn tiến hành không?`,
        );
        if (confirmGo) {
          appState.addLog("Info", "Bắt đầu tải xuống bản cập nhật...");

          let lastLoggedPercent = 0;
          // Đăng ký lắng nghe tiến trình tải
          progressUnsubscribe = await listen("update-progress", (event) => {
            const percent = event.payload;
            appState.statusText = `Đang tải bản cập nhật: ${percent}%`;
            if (percent === 100 || percent - lastLoggedPercent >= 10) {
              appState.addLog(
                "Info",
                `Đang tải bản cập nhật: ${percent}%`,
                false,
              );
              lastLoggedPercent = percent;
            }
          });

          await invoke("download_and_install", { url: result.download_url });
        }
      } else {
        if (!silent) {
          appState.addLog(
            "Info",
            `Bạn đang sử dụng phiên bản mới nhất (${result.current_version}).`,
          );
          alert(
            `Ứng dụng đang ở phiên bản mới nhất (${result.current_version}).`,
          );
        }
      }
    } catch (err) {
      if (infoToastId) {
        appState.removeToast(infoToastId);
      }
      if (!silent) {
        appState.addLog("Error", `Lỗi kiểm tra cập nhật: ${err}`);
        alert(`Lỗi kiểm tra cập nhật: ${err}`);
      }
    } finally {
      isChecking = false;
      if (progressUnsubscribe) {
        progressUnsubscribe();
      }
    }
  }

  function cycleTheme() {
    console.log("cycleTheme clicked! Current theme in state:", appState.theme);
    if (appState.theme === "light") {
      appState.setTheme("dark");
    } else if (appState.theme === "dark") {
      appState.setTheme("auto");
    } else {
      appState.setTheme("light");
    }
    console.log("Theme after cycle:", appState.theme);
  }
</script>

<aside
  class="w-16 flex flex-col items-center pb-6 bg-blue-600 dark:bg-[var(--accent)] text-white shrink-0 select-none z-20 shadow-lg justify-between"
>
  <div class="w-full flex flex-col items-center">
    <!-- Top Logo Box: Cố định h-16 và border-b đồng bộ -->
    <div class="h-16 w-full flex items-center justify-center border-b border-white/10 dark:border-black/10 shrink-0">
      <HungTayGaLogo size={28} />
    </div>

    <!-- Nav Icons -->
    <div class="flex flex-col items-center gap-4 mt-6">
      <div
        class="w-10 h-10 bg-white/10 rounded-xl flex items-center justify-center text-white cursor-pointer"
        title="Dashboard"
      >
        <LayoutDashboard size={20} />
      </div>
      <button
        onclick={() =>
          appState.showToast(
            "info",
            "Tính năng quản lý Database đang được phát triển",
          )}
        class="w-10 h-10 hover:bg-white/5 rounded-xl flex items-center justify-center text-white/70 hover:text-white cursor-pointer transition border-0 bg-transparent p-0"
        title="Database"
      >
        <Database size={20} />
      </button>
    </div>
  </div>

  <!-- Bottom Section (Update + Theme) -->
  <div class="flex flex-col items-center gap-4">
    <!-- Nút kiểm tra cập nhật -->
    <button
      onclick={() => checkUpdate(false)}
      disabled={isChecking}
      class="w-10 h-10 hover:bg-white/10 rounded-xl flex items-center justify-center text-white cursor-pointer transition relative group disabled:opacity-50"
      title="Kiểm tra bản cập nhật"
    >
      <RefreshCw
        size={20}
        class={isChecking
          ? "animate-spin text-blue-300"
          : "text-slate-200 group-hover:rotate-180 transition-transform duration-500"}
      />

      <!-- Tooltip kiểm tra cập nhật -->
      <span
        class="absolute left-14 bg-zinc-800 dark:bg-zinc-700 text-white text-[10px] px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition whitespace-nowrap pointer-events-none z-30 shadow-md"
      >
        Kiểm tra cập nhật
      </span>
    </button>

    <!-- Bottom Theme Toggle Icon -->
    <button
      onclick={cycleTheme}
      class="w-10 h-10 hover:bg-white/10 rounded-xl flex items-center justify-center text-white cursor-pointer transition relative group"
      title="Chuyển chế độ Sáng / Tối / Hệ thống"
    >
      {#if appState.theme === "light"}
        <Sun size={20} class="text-yellow-100/90" />
      {:else if appState.theme === "dark"}
        <Moon size={20} class="text-blue-200" />
      {:else}
        <Laptop size={20} class="text-emerald-200" />
      {/if}

      <!-- Tooltip hiển thị chế độ hiện tại -->
      <span
        class="absolute left-14 bg-zinc-800 dark:bg-zinc-700 text-white text-[10px] px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition whitespace-nowrap pointer-events-none z-30 shadow-md"
      >
        Theme: {appState.theme === "light"
          ? "Sáng"
          : appState.theme === "dark"
            ? "Tối"
            : "Tự động (Máy)"}
      </span>
    </button>
  </div>
</aside>
