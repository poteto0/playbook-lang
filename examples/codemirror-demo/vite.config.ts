import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    fs: {
      allow: ['..', '../../code-mirror', '../../pkg-linter']
    }
  },
  resolve: {
    dedupe: [
      'codemirror',
      '@codemirror/state',
      '@codemirror/view',
      '@codemirror/language',
      '@lezer/common',
      '@lezer/highlight',
      '@lezer/lr'
    ]
  }
});
