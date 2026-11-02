---
id: ST-0703-01
parent: T-0703
title: Клиент API, кэш, офлайн-поведение
status: ready
est: 0.5d
owner: rust
---

# ST-0703-01 — Клиент погоды

## Цель

Данные есть всегда (или честная пометка, что их нет).

## Реализация

1. `weather::provider`: трейт `WeatherProvider { fetch(lat, lon, units) -> WeatherSnapshot }`;
   реализация `OpenMeteo`.
2. Кэш в `settings.weather_cache` (JSON + `fetched_at`), TTL 30 мин.
3. Ошибки: `Network`, `Timeout`, `InvalidResponse` → UI показывает кэш.
4. Единицы: `temperature_unit`, `wind_speed_unit` из настроек.
5. Лимит: не чаще 1 запроса в 30 мин на город (проверка `fetched_at`).

## Критерии приёмки

- AC-1 4 сценария ошибок обработаны.
- AC-2 Кэш переживает перезапуск.
- AC-3 Не более 2 запросов в час.

## Тест-план

TC-0703-01, TC-0703-02.

## Риски

Провайдер может потребовать ключ в будущем → интерфейс провайдера и настройка
«свой endpoint».
