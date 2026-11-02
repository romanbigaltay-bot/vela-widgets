---
id: ST-0202-01
parent: T-0202
title: Поиск WorkerW и установка родителя
status: ready
est: 0.7d
owner: rust
---

# ST-0202-01 — Поиск WorkerW

## Цель

Функция, которая находит корректный `WorkerW` и привязывает к нему окно,
с диагностикой на каждом шаге.

## Реализация

1. `find_workerw() -> Result<HWND>`:
   - `FindWindowW("Progman", null)`;
   - `SendMessageTimeoutW(progman, 0x052C, 0xD, 0x1, SMTO_NORMAL, 1000, ...)`;
   - `EnumWindows` с колбэком: ищем `WorkerW` (`FindWindowExW(null, top, "WorkerW", null)`),
     у которого `FindWindowExW(workerw, null, "SHELLDLL_DefView", null)` == null.
2. Если ни один не найден — второй проход после `0x052C` с другим флагом (0).
3. `attach(hwnd, workerw)`: `SetParent`, затем `SetWindowPos` монитор-рамки,
   `ShowWindow(SW_SHOWNOACTIVATE)`.
4. Диагностика: `tracing::debug!` на каждом шаге с хендлами; при неудаче —
   структура `WorkerwFailure { stage, hwnds, last_error }` для баг-репорта.
5. Фича-флаг `scene.workerw_enabled` в настройках: выключен → сразу фолбэк.

## Критерии приёмки

- AC-1 На чистой 11 24H2 находит WorkerW за ≤ 120 мс (замер 10 прогонов).
- AC-2 Повторный вызов идемпотентен — не создаёт второй слой.
- AC-3 При отсутствии WorkerW возвращает структурированную ошибку, не паникует.
- AC-4 Отчёт об ошибке содержит имена окон и хендлы (без приватных данных).

## Тест-план

TC-0202-01 + ручной прогон на трёх сборках Windows; логи прилагаются к PR.

## Риски

На некоторых сборках нужен `0xD` в `wParam`; на других `0` — отсюда двухпроходный
поиск. Изменения проверяются на матрице версий до мержа.
