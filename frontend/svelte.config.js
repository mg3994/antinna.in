import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import("@sveltejs/vite-plugin-svelte").SvelteConfig} */
export default {
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  // for more information about preprocessors
  preprocess: vitePreprocess(),
  build: {
    // This tells Vite to dump the compiled CSR app into the Rust public folder
    outDir: '../backend/public',
    // Ensures old files are deleted so you don't serve stale code
    emptyOutDir: true,
  }
}
