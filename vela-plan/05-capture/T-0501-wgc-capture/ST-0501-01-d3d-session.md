---
id: ST-0501-01
parent: T-0501
title: D3D-устройство, frame pool, сессия
status: ready
est: 0.8d
owner: rust
---

# ST-0501-01 — D3D и сессия захвата

## Цель

Один кадр, быстро, без утечек и без чёрного экрана.

## Реализация

1. `capture::device`: ленивая инициализация `ID3D11Device` + `ID3D11DeviceContext`
   (`D3D11_CREATE_DEVICE_BGRA_SUPPORT`), хранится в `OnceCell`.
2. `capture::grab(monitor_or_window)`:
   - `Direct3D11CaptureFramePool::CreateFreeThreaded(device, B8G8R8A8_UNorm, 1, size)`;
   - `session.start()`; ждём первый `frame_arrived` с таймаутом 150 мс;
   - `CopyResource` → staging → `Map` → `Vec<u8>` RGBA;
   - `session.close()`, `pool.close()`.
3. Все вызовы — в выделенном потоке (`std::thread`, не tokio), результат через
   `oneshot` канал.
4. Таймаут и ошибки: `PlatformUnavailable` с причиной (`device`, `timeout`, `closed`).
5. Прогрев устройства при старте приложения (в фоне) — чтобы первый захват был быстрым.

## Критерии приёмки

- AC-1 Захват ≤ 40 мс после прогрева (без кодирования).
- AC-2 После 100 захватов число хендлов стабильно.
- AC-3 Таймаут даёт `PlatformUnavailable`, не зависание.

## Тест-план

TC-0501-01, TC-0501-02.

## Риски

`CreateFreeThreaded` требует поток с COM `MTA` → инициализировать
`CoInitializeEx(COINIT_MULTITHREADED)` в этом потоке.
