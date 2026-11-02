/**
 * Клиент IPC: единственный путь из UI в ядро.
 *
 * # Что здесь происходит
 *
 * 1. Команда уходит через транспорт.
 * 2. Конверт разворачивается: `ok && data` → данные, иначе — `VelaFailure`.
 * 3. Всё логируется: в dev — каждая команда с длительностью, в prod — только
 *    ошибки. Логи идут в кольцевой буфер из `log.ts` и попадают в отчёт
 *    диагностики (T-0805), поэтому `console.*` напрямую запрещён линтом.
 *
 * # Почему ошибка — объект, а не исключение с текстом
 *
 * UI принимает решения по `code`, а не по `message`: `permission_denied` —
 * показать «нет прав», `platform_unavailable` — заглушку, `busy` — повторить
 * через секунду. Текст ошибки локализуется отдельно и для логики бесполезен.
 */

import type { Envelope } from './Envelope';
import type { ErrorCode } from './ErrorCode';
import type { VelaError } from './VelaError';
import type { InvokeMap } from './commands';
import { detectTransport, type Transport } from './transport';
import { log } from '../lib/log';

/** Код ошибки для случая, который ядро не предусматривало. */
const FALLBACK_CODE: ErrorCode = 'internal';

/**
 * Ошибка вызова IPC.
 *
 * Наследуется от `Error`, чтобы работать с `try/catch` и попадать в трейсы,
 * но несёт машинный `code` и флаг `retryable`.
 */
export class VelaFailure extends Error {
  /** Машинный код: по нему принимаются решения. */
  readonly code: ErrorCode;
  /** Имя команды — для диагностики и тостов. */
  readonly command: string;
  /** Имеет ли смысл повторить. */
  readonly retryable: boolean;
  /** Исходная ошибка транспорта, если конверта не было. */
  override readonly cause?: unknown;

  constructor(
    command: string,
    code: ErrorCode,
    message: string,
    retryable: boolean,
    cause?: unknown,
  ) {
    super(message);
    this.name = 'VelaFailure';
    this.command = command;
    this.code = code;
    this.retryable = retryable;
    if (cause !== undefined) {
      this.cause = cause;
    }
  }
}

/** Признак объекта ошибки из конверта. Проверяется структурно: JSON не даёт классов. */
function isVelaError(value: unknown): value is VelaError {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate['code'] === 'string' && typeof candidate['message'] === 'string';
}

/** Признак конверта. */
function isEnvelope(value: unknown): value is Envelope<unknown> {
  if (typeof value !== 'object' || value === null) return false;
  return typeof (value as Record<string, unknown>)['ok'] === 'boolean';
}

/** Длительность в миллисекундах с одним знаком после запятой. */
function elapsedMs(started: number): number {
  return Math.round((performance.now() - started) * 10) / 10;
}

/**
 * Текущий транспорт. Заменяемый ради тестов: подмена глобального объекта
 * заставляла бы каждый тест думать о порядке инициализации.
 */
let transport: Transport = detectTransport();

/**
 * Подменяет транспорт.
 *
 * Существует для тестов и для онбординга, которому нужно показать интерфейс
 * без ядра. В продакшн-коде не вызывается — это подчёркнуто именем.
 */
export function __setTransportForTests(next: Transport): void {
  transport = next;
}

/** Возвращает текущий транспорт — только для диагностики. */
export function currentTransport(): Transport {
  return transport;
}

/**
 * Вызывает команду IPC и возвращает развёрнутые данные.
 *
 * @example
 * ```ts
 * const snapshot = await call('layout.get', { monitor: 'm1' });
 * ```
 *
 * @throws {VelaFailure} если ядро вернуло ошибку или транспорт недоступен.
 */
export async function call<K extends keyof InvokeMap & string>(
  command: K,
  payload: InvokeMap[K]['req'],
): Promise<InvokeMap[K]['res']> {
  const started = performance.now();

  let raw: unknown;
  try {
    raw = await transport.invoke(command, payload);
  } catch (cause) {
    // Транспорт упал до получения конверта: Tauri не загрузился, окно
    // закрылось, команда не зарегистрирована. Это всегда `platform_unavailable`:
    // отличить «нет такой команды» от «нет ядра» отсюда нельзя, а для UI
    // реакция одна — показать заглушку.
    const ms = elapsedMs(started);
    const failure = new VelaFailure(
      command,
      'platform_unavailable',
      cause instanceof Error ? cause.message : String(cause),
      false,
      cause,
    );
    log('error', `ipc ${command} ${ms}ms transport-failed`, { code: failure.code });
    throw failure;
  }

  const ms = elapsedMs(started);

  if (!isEnvelope(raw)) {
    // Ядро ответило чем-то, что не является конвертом. Это ошибка контракта,
    // а не пользователя: молча вернуть `undefined` значило бы спрятать
    // рассинхрон версий бинаря и UI.
    const failure = new VelaFailure(
      command,
      FALLBACK_CODE,
      `ответ команды ${command} не является конвертом`,
      false,
    );
    log('error', `ipc ${command} ${ms}ms bad-envelope`);
    throw failure;
  }

  if (raw.ok && raw.data !== null) {
    if (import.meta.env.DEV) {
      // Формат из AC-4: `layout.get 3.2ms ok`. Уровень `debug`: успешный
      // вызов не должен вытеснять из буфера диагностики настоящие ошибки.
      log('debug', `ipc ${command} ${ms}ms ok`);
    }
    return raw.data as InvokeMap[K]['res'];
  }

  if (raw.ok) {
    // `ok: true` без данных — валидный ответ для команд без результата.
    // Возвращаем `undefined`, а не бросаем: контракт `Envelope<void>`.
    if (import.meta.env.DEV) {
      log('debug', `ipc ${command} ${ms}ms ok (empty)`);
    }
    return undefined as InvokeMap[K]['res'];
  }

  const error = raw.error;
  const code = isVelaError(error) ? error.code : FALLBACK_CODE;
  const message = isVelaError(error) ? error.message : 'неизвестная ошибка';
  const retryable = isVelaError(error) ? error.retryable : false;

  log('error', `ipc ${command} ${ms}ms ${code}`, { message });
  throw new VelaFailure(command, code, message, retryable, error);
}

/** Повторяет вызов при `retryable`-ошибке. */
export async function callWithRetry<K extends keyof InvokeMap & string>(
  command: K,
  payload: InvokeMap[K]['req'],
  attempts = 3,
): Promise<InvokeMap[K]['res']> {
  let lastFailure: VelaFailure | undefined;

  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      return await call(command, payload);
    } catch (err) {
      if (!(err instanceof VelaFailure) || !err.retryable) throw err;
      lastFailure = err;
      // Линейная пауза: экспонента на трёх попытках даёт максимум 300 мс,
      // а пользователь уже смотрит на спиннер.
      await new Promise((resolve) => setTimeout(resolve, attempt * 100));
    }
  }

  throw lastFailure ?? new VelaFailure(command, FALLBACK_CODE, 'вызов не выполнен', false);
}
