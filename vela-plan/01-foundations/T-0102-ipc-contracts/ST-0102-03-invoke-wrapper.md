---
id: ST-0102-03
parent: T-0102
title: Обёртка invoke и реестр событий в UI
status: in-progress
est: 0.4d
owner: ui
depends: [ST-0102-02]
---

# ST-0102-03 — Обёртка invoke и реестр событий

## Цель

UI не вызывает `@tauri-apps/api` напрямую; все вызовы типизированы, логируются,
одинаково обрабатывают ошибки.

## Реализация

1. `shared/api/client.ts`: `call<K extends keyof InvokeMap>(cmd: K, req)` →
   `Promise<InvokeMap[K]['res']>`, разворачивает `Envelope`, бросает `VelaError`.
2. Логирование: `{cmd, ms, ok, code}` в dev; в prod — только ошибки, через
   кольцевой буфер на 200 записей (полезно для баг-репорта).
3. `shared/api/events.ts`: `EventMap` с типизированным `on<E>()`, автоочистка
   подписок через `useEffect`-хелпер `useEvent`.
4. Dev-валидация payload: отложена до P2 (`D-18`) — в P1 нет транспорта и
   реальных команд, валидировать нечего. При появлении первой команды
   добавить медленный `zod`-парсер только в `import.meta.env.DEV`.
5. Запрет прямого импорта `@tauri-apps/api/core` — ESLint `no-restricted-imports`.

## Критерии приёмки

- AC-1 Прямой `invoke` из компонента — ошибка линта.
- AC-2 Ошибка `PermissionDenied` приходит как типизированный объект с `code`.
- AC-3 Подписка на событие снимается при размонтировании (тест на утечку).
- AC-4 В dev-консоли видно `layout.get 3.2ms ok`.

## Тест-план

Vitest: мок `window.__TAURI__`, проверка разворачивания конверта, ошибки,
снятия подписки.

## Доказательства (2026-11-06)

- AC-1 — ESLint `no-restricted-imports` запрещает `@tauri-apps/api/core`
  вне `shared/api`; `pnpm lint` зелёный.
- AC-2 — `client.test.ts` «сохраняет код, сообщение и флаг retryable».
- AC-3 — `events.test.ts` (12 тестов) и `useEvent.test.ts` (4 теста),
  включая снятие подписки при размонтировании.
- AC-4 — CDP на живом Vite: `ipc probe.echo 0ms ok` в dev-консоли.

## Риски

Типизация `call` через mapped types даёт сложные сообщения об ошибках —
добавить `JSDoc` с примером.
