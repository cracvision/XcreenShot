import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));

// Tauri expects a fixed port and no screen clearing.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Locale files live in crates/xs-core/locales and are shared with Rust.
    fs: { allow: [repoRoot] },
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
  test: {
    environment: "jsdom",
  },
});
