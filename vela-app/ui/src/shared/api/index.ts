/**
 * Точка входа контрактов IPC.
 *
 * Единственный модуль, который импортируют потребители: приложение не должно
 * знать, что за `call()` стоит `client.ts`, а за `on()` — `events.ts`.
 * Это позволяет переставить внутренности, не трогая вызывающий код.
 *
 * Файл рукописный, поэтому перечислен в `handwritten` в
 * `crates/vela-ipc/contract.json` — `ipc-gen` его не тронет.
 */

// ---------------------------------------------------------------------------
// Сгенерированные типы (ts-rs). Не редактируются вручную.
// ---------------------------------------------------------------------------
export type { ElementId } from './ElementId';
export type { MonitorId } from './MonitorId';
export type { ProfileId } from './ProfileId';
export type { WidgetId } from './WidgetId';
export type { ErrorCode } from './ErrorCode';
export type { VelaError } from './VelaError';
export type { Envelope } from './Envelope';

// ---------------------------------------------------------------------------
// Карты команд и событий. Расширяются через `declare module` в фичах:
//
//   declare module '@/shared/api' {
//     interface CommandMap {
//       'layout.get': { req: { monitor: string }; res: Element[] };
//     }
//   }
// ---------------------------------------------------------------------------
export type { CommandMap, InvokeMap } from './commands';
export type { EventMap, Handler } from './events';

// ---------------------------------------------------------------------------
// Вызовы и подписки
// ---------------------------------------------------------------------------
export { call, callWithRetry, VelaFailure, currentTransport } from './client';
export { on, emitLocal, subscriberCount } from './events';
export { useEvent } from './useEvent';
export { IPC_VERSION } from './version';

// ---------------------------------------------------------------------------
// Транспорт — нужен только тестам и диагностике.
// ---------------------------------------------------------------------------
export type { Transport, Unsubscribe } from './transport';
