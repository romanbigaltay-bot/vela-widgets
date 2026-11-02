---
id: ST-0404-01
parent: T-0404
title: Модель заметки, репозиторий, IPC
status: ready
est: 0.5d
owner: rust
---

# ST-0404-01 — Модель заметки

## Цель

Заметки как данные: валидация, сохранение, поиск.

## Реализация

1. `vela-core::note`: `Note`, `ChecklistItem`, валидация (лимит 100 КБ текста,
   500 пунктов).
2. `NotesRepo`: `get(element_id)`, `save(note)`, `search(query)` (LIKE + индексы
   `FTS5` для body — задел на P7).
3. IPC: `notes.get`, `notes.save`, `notes.search`.
4. Каскад: удаление элемента удаляет заметку (FK `ON DELETE CASCADE`).
5. `search`: `FTS5` с `tokenize = 'unicode61 remove_diacritics 2'`; при отсутствии —
   фолбэк на `LIKE`.

## Критерии приёмки

- AC-1 `save` 100 КБ текста ≤ 10 мс.
- AC-2 `search` по 1000 заметкам ≤ 20 мс.
- AC-3 Удаление элемента удаляет заметку.

## Тест-план

TC-0404-01 + бенч поиска на 1000 записях.

## Риски

FTS5 требует сборки `rusqlite` с фичей `bundled-full` → проверить размер бинаря.
