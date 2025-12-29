import { defineConfig } from 'vite';

export default defineConfig({
  assetsInclude: ['**/*.wasm'],
  server: {
    fs: {
      allow: ['..']
    }
  }
});

