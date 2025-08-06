import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const host = process.env.TAURI_DEV_HOST;
const isTauri = process.env.TAURI === "true";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],
  
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
          overlay: false, // Disable the HMR error overlay
        }
      : {
          overlay: false, // Disable the HMR error overlay
        },
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // Add build configuration for Tauri
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: isTauri ? 'chrome110' : 'esnext',
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  // Prevent prebundling of certain deps
  optimizeDeps: {
    exclude: [
      '@tauri-apps/api',
      '@tauri-apps/api/core',  // Updated for Tauri v2
      '@tauri-apps/api/plugin-dialog',
      '@tauri-apps/api/plugin-fs'
    ]
  }
}));
