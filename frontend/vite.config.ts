import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    // THIS is the correct place for these settings
    outDir: '../backend/public',
    emptyOutDir: true,
  }
})
