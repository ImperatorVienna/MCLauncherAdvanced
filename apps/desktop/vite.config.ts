import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
export default defineConfig({
  plugins: [react()],
  test: { css: { include: /app\.css/ } },
  server: { port: 1420, strictPort: true, proxy: { "/official-content/": { target: "https://launchercontent.mojang.com", changeOrigin: true, rewrite: path => path.replace(/^\/official-content\//, "/v2/") } } },
  clearScreen: false,
});
