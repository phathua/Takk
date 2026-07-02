import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

const rawHost = process.env.TAURI_DEV_HOST;
const host = rawHost && rawHost !== "$null" ? rawHost : undefined;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    svelte(),
    tailwindcss(),
  ],
  build: {
    emptyOutDir: false,
  },
  
  resolve: {
    conditions: ["browser", "import", "module", "default"],
  },

  optimizeDeps: {
    entries: ["index.html"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/$null", "**/_tmp/**"],
    },
  },
}));
