---
id: ST-0101-03
parent: T-0101
title: Линты и форматирование
status: in-progress
est: 0.2d
owner: both
depends: [ST-0101-01, ST-0101-02]
---

# ST-0101-03 — Линты и форматирование

## Цель

Единый стиль без обсуждений в ревью.

## Реализация

1. `.rustfmt.toml`: `max_width = 100`, `edition = "2021"`.
2. `clippy.toml`: `disallowed-methods` — `std::thread::sleep` (кроме тестов),
   `unwrap` вне тестов через `disallowed_methods`.
3. ESLint flat config: `@typescript-eslint` strict, `react-hooks`, `import/order`,
   запрет `console.log` (кроме `warn/error`).
4. Prettier: `printWidth 100`, `singleQuote`, `trailingComma all`.
5. Husky + lint-staged: на commit — fmt + lint по изменённым файлам.

## Критерии приёмки

- AC-1 Коммит с `console.log` отклоняется хуком.
- AC-2 `cargo fmt --check` в CI падает при неформатированном коде.
- AC-3 Правило `no-console` имеет исключение только для `src/shared/lib/log.ts`.

## Тест-план

Ручной: три коммита-нарушителя, все отклонены.

## Риски

Хуки легко обходятся `--no-verify` — CI остаётся последним рубежом.
