import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({ root: "apps/preview-vue", plugins: [vue()] });
