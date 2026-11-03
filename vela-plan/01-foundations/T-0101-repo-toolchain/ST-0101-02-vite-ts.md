---
id: ST-0101-02
parent: T-0101
title: Vite + React + TS со строгим tsconfig
status: in-progress
est: 0.3d
owner: ui
depends: [ST-0101-01]
---

# ST-0101-02 — Vite + React + TS со строгим tsconfig

## Цель

Фронтенд-каркас, который стартует за < 1 с и не даёт писать небезопасный TS.

## Реализация

1. `ui/package.json`: `vite`, `react`, `react-dom`, `typescript`, `zustand`,
   `@tauri-apps/api`, `vitest`, `@testing-library/react`.
2. `tsconfig.json`: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
   `noImplicitOverride`, `paths: { "@/*": ["src/*"] }`.
3. `vite.config.ts`: alias `@`, `build.target: "chrome110"` (WebView2 baseline),
   `server.strictPort: true`, `clearScreen: false`.
4. `src/app/main.tsx` — `createRoot` + `<App/>`, `App` печатает `Hello Vela`.
5. `index.html` — `lang="ru"`, `color-scheme` мета, пустой `<div id="root">`.

## Критерии приёмки

- AC-1 `pnpm build` даёт `< 120 КБ` gzip для пустого приложения.
- AC-2 `tsc --noEmit` падает на `const x: string = undefined`.
- AC-3 Алиас `@/shared/ui` резолвится и в vite, и в tsc, и в vitest.

## Тест-план

`vitest run` с единственным тестом, проверяющим, что `App` рендерится.

## Риски

Vite 5 + Tauri требуют `build.target` не ниже возможностей WebView2 —
проверяется на реальном окне в P2.
