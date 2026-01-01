import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/rest": {
        target: "http://localhost:54321",
        changeOrigin: true,
      },
      "/auth": {
        target: "http://localhost:54321",
        changeOrigin: true,
      },
      "/storage": {
        target: "http://localhost:54321",
        changeOrigin: true,
      },
      "/realtime": {
        target: "ws://localhost:54321",
        ws: true,
      },
    },
  },
});
