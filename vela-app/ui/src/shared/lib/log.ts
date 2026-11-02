/**
 * Логгер фронтенда. Единственное место, где разрешён console.*.
 *
 * Задача: к ошибкам UI должен быть доступ и в консоли разработчика, и в
 * кольцевом буфере, который попадает в отчёт диагностики (T-0805, AC-2).
 *
 * # Почему три уровня, а не два
 *
 * Буфер ограничен 200 записями и существует ради разбора инцидентов. Если
 * писать в него всё подряд — а трассировка каждого IPC-вызова в dev идёт
 * потоком, — настоящая ошибка вытесняется за секунды, и отчёт бесполезен.
 * Поэтому `debug` идёт только в консоль: он нужен разработчику здесь и сейчас,
 * а не в архиве с чужой машины.
 */

export type LogLevel = 'debug' | 'warn' | 'error';

export interface LogEntry {
  readonly at: number;
  readonly level: LogLevel;
  readonly message: string;
  readonly details?: unknown;
}

/** Кольцевой буфер последних записей. Размер фиксирован: логи не растут. */
const CAPACITY = 200;
const buffer: LogEntry[] = [];

/**
 * Уровни, попадающие в буфер диагностики.
 *
 * `debug` исключён намеренно: он шумит и вытесняет то, ради чего буфер нужен.
 */
function isBuffered(level: LogLevel): boolean {
  return level !== 'debug';
}

export function log(level: LogLevel, message: string, details?: unknown): void {
  if (isBuffered(level)) {
    const entry: LogEntry = {
      at: Date.now(),
      level,
      message,
      ...(details === undefined ? {} : { details }),
    };
    if (buffer.length === CAPACITY) {
      buffer.shift();
    }
    buffer.push(entry);
  }

  // В консоль — чтобы запись была видна сразу. Уровни разделены намеренно:
  // предупреждения не должны выглядеть как падения, а трассировка — как
  // предупреждения.
  switch (level) {
    case 'error':
      console.error(message, details);
      break;
    case 'warn':
      console.warn(message, details);
      break;
    case 'debug':
      // `console.debug` скрыт по умолчанию в фильтре DevTools — ровно то,
      // что нужно для трассировки отдельных вызовов.
      console.debug(message, details);
      break;
  }
}

/** Возвращает копию буфера — для отчёта диагностики. */
export function recentLogs(): readonly LogEntry[] {
  return [...buffer];
}

/** Очищает буфер. Используется тестами и при выходе из приватного режима. */
export function clearLogs(): void {
  buffer.length = 0;
}

/** Сколько записей сейчас в буфере — нужно тестам и dev-панели. */
export function bufferSize(): number {
  return buffer.length;
}
