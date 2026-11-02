import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';

/**
 * Сборка фронтенда Vela.
 *
 * `build.target: 'chrome110'` — нижняя граница WebView2, которую мы поддерживаем
 * (Windows 11 22H2+). Всё, что новее, ломает запуск у части пользователей,
 * поэтому указано явно, а не «esnext» по умолчанию.
 *
 * `server.strictPort` — Tauri в dev-режиме обращается к фиксированному порту;
 * «тихо переехать на 5174» привело бы к белому окну вместо понятной ошибки.
 */
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  build: {
    target: 'chrome110',
    // Стекло и сцена требуют WebGL2, но бандл должен быть предсказуемым:
    // отдельные чанки для сцены подключаются динамически (P6).
    sourcemap: false,
    reportCompressedSize: true,
    rollupOptions: {
      output: {
        manualChunks: {
          react: ['react', 'react-dom'],
        },
      },
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  // Tauri перезапускает окно само; очистка экрана съедает логи Rust.
  clearScreen: false,
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
