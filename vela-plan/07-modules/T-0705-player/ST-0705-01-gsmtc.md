---
id: ST-0705-01
parent: T-0705
title: Обёртка WinRT GSMTC и события
status: ready
est: 0.5d
owner: rust
---

# ST-0705-01 — GSMTC

## Цель

Данные о текущем треке и управление — из одного системного источника.

## Реализация

1. `media::session`: `GlobalSystemMediaTransportControlsSessionManager::RequestAsync()`,
   выбор текущей сессии, подписка на `MediaPropertiesChanged`,
   `PlaybackInfoChanged`, `TimelinePropertiesChanged`.
2. Модель `NowPlaying { title, artist, album, art: Option<Vec<u8>>, playing, position_ms, duration_ms }`.
3. Обложка: `TryGetThumbnailAsync` → `IRandomAccessStreamReference` → байты → кэш по
   хешу (не пересылать каждый раз).
4. Управление: `TryTogglePlayPauseAsync`, `TrySkipNextAsync`, `TrySkipPreviousAsync`,
   `TryChangePlaybackPositionAsync` (где поддерживается).
5. Событие `media.now_playing` в UI, только при изменениях (не поток).

## Критерии приёмки

- AC-1 Обновление ≤ 1 с после переключения трека.
- AC-2 Обложка не пересылается повторно при том же треке.
- AC-3 `Try*` возвращает `false` — UI не показывает ошибку, просто не срабатывает.

## Тест-план

TC-0705-01/02, ручной на 3 плеерах.

## Риски

WinRT требует STA/MTA-поток — использовать MTA-поток и `async` через `block_on`
вне главного потока.
