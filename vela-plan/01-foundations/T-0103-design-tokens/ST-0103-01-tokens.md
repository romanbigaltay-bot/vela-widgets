---
id: ST-0103-01
parent: T-0103
title: Токены и базовая типографика
status: planned
est: 0.4d
owner: ui
---

# ST-0103-01 — Токены и типографика

## Цель

Один файл `styles/tokens.css` с двумя наборами значений и базовый слой.

## Реализация

1. `tokens.css`: `:root[data-theme="dark"]`, `:root[data-theme="light"]` —
   полный набор групп из README T-0103.
2. `base.css`: `*{box-sizing:border-box}`, `html,body{margin:0;height:100%}`,
   `body{background:var(--bg);color:var(--text);font:400 13px/1.45 var(--font)}`.
3. Шрифт: `Inter Tight` вариативный, self-hosted (`woff2`, `font-display: swap`),
   `--font` со стеком фолбэков на `Segoe UI Variable`.
4. Шкала: классы `.t-caption .t-body .t-label .t-title .t-display .t-hero`
   с фиксированными размером/весом/трекингом. Других размеров нет.
5. `tabular-nums` включён глобально для элементов с классом `.num`.

## Критерии приёмки

- AC-1 Все 7 размеров шкалы используются хотя бы раз в ui-kit-витрине.
- AC-2 Шрифт грузится локально, запросов к внешним CDN нет (проверка в DevTools).
- AC-3 `--font` фолбэк-стек корректен без загруженного Inter Tight.

## Тест-план

Visual-скриншот витрины типографики в обеих темах; проверка отсутствия
внешних запросов в e2e.

## Риски

Вариативный шрифт даёт разную метрику на разных WebView2 — фиксируется
`font-optical-sizing: none` и явный `font-variation-settings`.
