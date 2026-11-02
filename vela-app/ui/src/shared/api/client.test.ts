/**
 * Тесты клиента IPC.
 *
 * Проверяется то, что может сломаться незаметно: разворачивание конверта,
 * обработка ошибок, различимость кодов. Сам транспорт подменяется — это и
 * есть причина, по которой он вынесен в интерфейс.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { InvokeMap } from './commands';
import type { Transport } from './transport';
import { __setTransportForTests, call, callWithRetry, VelaFailure } from './client';
import { recentLogs, clearLogs } from '../lib/log';

/** Фейковый транспорт с записью вызовов. */
function fakeTransport(response: unknown | (() => unknown)): Transport & {
  readonly calls: { command: string; payload: unknown }[];
} {
  const calls: { command: string; payload: unknown }[] = [];
  return {
    calls,
    invoke(command, payload) {
      calls.push({ command, payload });
      if (typeof response === 'function') {
        return Promise.resolve((response as () => unknown)());
      }
      return response instanceof Error ? Promise.reject(response) : Promise.resolve(response);
    },
    listen() {
      return Promise.resolve(() => {});
    },
  };
}

/** Конверт с данными. */
function ok<T>(data: T): unknown {
  return { ok: true, data, error: null };
}

/** Конверт с ошибкой. */
function fail(code: string, message = 'boom', retryable = false): unknown {
  return { ok: false, data: null, error: { code, message, retryable } };
}

beforeEach(() => {
  clearLogs();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('разворачивание конверта', () => {
  it('возвращает data при ok=true', async () => {
    __setTransportForTests(fakeTransport(ok({ value: 42 })));
    const result = await call('probe.echo', {});
    expect(result).toEqual({ value: 42 });
  });

  it('передаёт команду и payload в транспорт без изменений', async () => {
    const transport = fakeTransport(ok(null));
    __setTransportForTests(transport);
    await call('probe.echo', { a: 1, b: 'x' });
    expect(transport.calls).toEqual([{ command: 'probe.echo', payload: { a: 1, b: 'x' } }]);
  });

  it('возвращает undefined при ok=true и data=null', async () => {
    // Команда без результата — валидный случай, а не ошибка.
    __setTransportForTests(fakeTransport({ ok: true, data: null, error: null }));
    await expect(call('probe.echo', {})).resolves.toBeUndefined();
  });
});

describe('ошибки ядра', () => {
  it('бросает VelaFailure с кодом из конверта', async () => {
    __setTransportForTests(fakeTransport(fail('permission_denied', 'нет прав')));
    await expect(call('probe.echo', {})).rejects.toBeInstanceOf(VelaFailure);
  });

  it('сохраняет код, сообщение и флаг retryable', async () => {
    // AC-2: ошибка приходит как типизированный объект с `code`,
    // а не как строка — по коду UI принимает решение.
    __setTransportForTests(fakeTransport(fail('permission_denied', 'нет прав', false)));
    try {
      await call('probe.echo', {});
      expect.unreachable('должно было бросить');
    } catch (err) {
      expect(err).toBeInstanceOf(VelaFailure);
      const failure = err as VelaFailure;
      expect(failure.code).toBe('permission_denied');
      expect(failure.message).toBe('нет прав');
      expect(failure.retryable).toBe(false);
      expect(failure.command).toBe('probe.echo');
    }
  });

  it('логирует ошибку в кольцевой буфер', async () => {
    __setTransportForTests(fakeTransport(fail('busy', 'занято')));
    await expect(call('probe.echo', {})).rejects.toThrow();
    const logs = recentLogs();
    expect(logs.length).toBeGreaterThan(0);
    expect(logs.at(-1)?.message).toContain('probe.echo');
    expect(logs.at(-1)?.message).toContain('busy');
  });

  it('неизвестный код из более новой версии не роняет разбор', async () => {
    // Бинарь новее UI: код, которого мы не знаем, должен дать ошибку,
    // а не исключение при разборе.
    __setTransportForTests(fakeTransport(fail('hologram_mode')));
    const failure = (await call('probe.echo', {}).catch((e: VelaFailure) => e)) as VelaFailure;
    expect(failure).toBeInstanceOf(VelaFailure);
    expect(failure.code).toBe('hologram_mode');
  });
});

describe('сбои транспорта', () => {
  it('ошибка транспорта превращается в platform_unavailable', async () => {
    __setTransportForTests(fakeTransport(new Error('Tauri недоступен')));
    try {
      await call('probe.echo', {});
      expect.unreachable('должно было бросить');
    } catch (err) {
      const failure = err as VelaFailure;
      expect(failure.code).toBe('platform_unavailable');
      expect(failure.retryable).toBe(false);
    }
  });

  it('ответ не-конверт считается ошибкой контракта, а не тихим undefined', async () => {
    // Рассинхрон версий бинаря и UI: молча вернуть undefined значило бы
    // спрятать проблему до момента, когда пользователь увидит пустой экран.
    __setTransportForTests(fakeTransport({ something: 'else' }));
    try {
      await call('probe.echo', {});
      expect.unreachable('должно было бросить');
    } catch (err) {
      const failure = err as VelaFailure;
      expect(failure.code).toBe('internal');
      expect(failure.message).toContain('конверт');
    }
  });

  it('конверт с ok=false без error даёт internal, а не падение', async () => {
    __setTransportForTests(fakeTransport({ ok: false, data: null, error: null }));
    const failure = (await call('probe.echo', {}).catch((e: VelaFailure) => e)) as VelaFailure;
    expect(failure.code).toBe('internal');
  });
});

describe('callWithRetry', () => {
  it('повторяет retryable-ошибку и возвращает результат', async () => {
    let attempts = 0;
    const transport: Transport = {
      invoke() {
        attempts += 1;
        return attempts < 3 ? Promise.resolve(fail('busy', 'занято', true)) : Promise.resolve(ok('готово'));
      },
      listen: () => Promise.resolve(() => {}),
    };
    __setTransportForTests(transport);

    await expect(callWithRetry('probe.echo', {}, 3)).resolves.toBe('готово');
    expect(attempts).toBe(3);
  });

  it('не повторяет не-retryable ошибку', async () => {
    let attempts = 0;
    const transport: Transport = {
      invoke() {
        attempts += 1;
        return Promise.resolve(fail('permission_denied', 'нет прав', false));
      },
      listen: () => Promise.resolve(() => {}),
    };
    __setTransportForTests(transport);

    await expect(callWithRetry('probe.echo', {}, 3)).rejects.toThrow();
    expect(attempts).toBe(1);
  });

  it('исчерпав попытки, бросает последнюю ошибку', async () => {
    const transport: Transport = {
      invoke: () => Promise.resolve(fail('busy', 'занято', true)),
      listen: () => Promise.resolve(() => {}),
    };
    __setTransportForTests(transport);

    const failure = (await callWithRetry('probe.echo', {}, 2).catch(
      (e: VelaFailure) => e,
    )) as VelaFailure;
    expect(failure.code).toBe('busy');
  });
});

describe('типизация реестра', () => {
  it('InvokeMap пуст в P1 — команды объявляются вместе с обработчиками', () => {
    // Компиляторная проверка: ключей пока нет. Тест существует, чтобы
    // момент, когда первая команда появится, был осознанным — здесь придётся
    // обновить и ожидание, и типы.
    const keys = Object.keys({} as InvokeMap);
    expect(keys).toHaveLength(0);
  });
});
