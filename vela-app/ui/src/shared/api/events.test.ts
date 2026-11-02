/**
 * Тесты шины событий.
 *
 * Главное, что здесь проверяется, — не доставка (она тривиальна), а отсутствие
 * утечек: подписка, не снятая при размонтировании, держит компонент живым и
 * заставляет ядро слать события в пустоту. Это самый частый дефект в
 * приложениях с шиной.
 */
import { beforeEach, describe, expect, it } from 'vitest';
import type { Transport, Unsubscribe } from './transport';
import { __setTransportForTests } from './client';
import { emitLocal, on, resetAllSubscriptions, subscriberCount } from './events';

// Расширение карты событий — ровно так это будет делать реальный модуль
// (ST-0302-03 объявит `layout.changed`). Без расширения `on('probe.tick', ...)`
// не типизируется: `keyof EventMap` — это `never`, и это правильно, потому
// что типизировать подписку на несуществующее событие нельзя.
declare module './events' {
  interface EventMap {
    'probe.tick': { n?: number; from?: string };
    'a.one': Record<string, never>;
    'b.two': Record<string, never>;
  }
}

/** Транспорт, запоминающий нативные подписки и отписки. */
function recordingTransport(): Transport & {
  readonly listeners: Map<string, Set<(payload: unknown) => void>>;
  readonly unlistenCalls: string[];
} {
  const listeners = new Map<string, Set<(payload: unknown) => void>>();
  const unlistenCalls: string[] = [];

  return {
    listeners,
    unlistenCalls,
    invoke: () => Promise.resolve({ ok: true, data: null, error: null }),
    listen(event: string, handler: (payload: unknown) => void): Promise<Unsubscribe> {
      let set = listeners.get(event);
      if (!set) {
        set = new Set();
        listeners.set(event, set);
      }
      set.add(handler);
      return Promise.resolve(() => {
        unlistenCalls.push(event);
        set?.delete(handler);
      });
    },
  };
}

/** Отправляет событие так, как это сделало бы ядро. */
function nativeEmit(transport: ReturnType<typeof recordingTransport>, event: string, payload: unknown): void {
  for (const handler of transport.listeners.get(event) ?? []) {
    handler(payload);
  }
}

beforeEach(() => {
  resetAllSubscriptions();
  __setTransportForTests(recordingTransport());
});

describe('подписка и доставка', () => {
  it('доставляет payload всем подписчикам', () => {
    const received: { n?: number; from?: string }[] = [];
    on('probe.tick', (p) => received.push(p));
    on('probe.tick', (p) => received.push(p));

    emitLocal('probe.tick', { n: 1 });

    expect(received).toEqual([{ n: 1 }, { n: 1 }]);
  });

  it('держит один нативный слушатель на событие при нескольких подписчиках', async () => {
    // Иначе каждая панель создавала бы свой слушатель, и снятие одного
    // не освобождало бы ресурс.
    const transport = recordingTransport();
    __setTransportForTests(transport);

    on('probe.tick', () => {});
    on('probe.tick', () => {});
    on('probe.tick', () => {});
    await Promise.resolve();

    expect(transport.listeners.get('probe.tick')?.size).toBe(1);
  });

  it('доставляет событие от ядра через нативный слушатель', async () => {
    const transport = recordingTransport();
    __setTransportForTests(transport);
    const received: { n?: number; from?: string }[] = [];
    on('probe.tick', (p) => received.push(p));
    await Promise.resolve();

    nativeEmit(transport, 'probe.tick', { from: 'core' });

    expect(received).toEqual([{ from: 'core' }]);
  });
});

describe('отписка и утечки', () => {
  it('снимает подписчика при отписке', () => {
    const received: { n?: number; from?: string }[] = [];
    const off = on('probe.tick', (p) => received.push(p));
    off();

    emitLocal('probe.tick', { n: 1 });

    expect(received).toEqual([]);
    expect(subscriberCount('probe.tick')).toBe(0);
  });

  it('AC-3: уход последнего подписчика снимает нативный слушатель', async () => {
    const transport = recordingTransport();
    __setTransportForTests(transport);

    const off1 = on('probe.tick', () => {});
    const off2 = on('probe.tick', () => {});
    await Promise.resolve();

    off1();
    await Promise.resolve();
    expect(transport.unlistenCalls).toEqual([]);
    expect(transport.listeners.get('probe.tick')?.size).toBe(1);

    off2();
    await Promise.resolve();
    expect(transport.unlistenCalls).toEqual(['probe.tick']);
    expect(transport.listeners.get('probe.tick')?.size).toBe(0);
  });

  it('повторная отписка безопасна', async () => {
    // Обработчик может вызываться дважды (StrictMode в React монтирует
    // дважды), и это не должно снимать чужую подписку.
    const transport = recordingTransport();
    __setTransportForTests(transport);

    const off = on('probe.tick', () => {});
    await Promise.resolve();
    off();
    off();
    await Promise.resolve();

    expect(transport.unlistenCalls).toEqual(['probe.tick']);
  });

  it('50 циклов подписка/отписка не оставляют слушателей', async () => {
    const transport = recordingTransport();
    __setTransportForTests(transport);

    for (let i = 0; i < 50; i += 1) {
      const off = on('probe.tick', () => {});
      off();
    }
    await Promise.resolve();

    expect(subscriberCount('probe.tick')).toBe(0);
    expect(transport.listeners.get('probe.tick')?.size ?? 0).toBe(0);
    expect(transport.unlistenCalls).toHaveLength(50);
  });
});

describe('устойчивость', () => {
  it('падение одного обработчика не срывает доставку остальным', () => {
    const received: string[] = [];
    on('probe.tick', () => {
      throw new Error('обработчик упал');
    });
    on('probe.tick', () => received.push('второй'));
    on('probe.tick', () => received.push('третий'));

    emitLocal('probe.tick', {});

    expect(received).toEqual(['второй', 'третий']);
  });

  it('отписка во время доставки не пропускает следующего подписчика', () => {
    // Мутация Set прямо в цикле пропустила бы элемент — поэтому доставка
    // идёт по копии набора.
    const received: string[] = [];
    const off = on('probe.tick', () => {
      received.push('первый');
      off();
    });
    on('probe.tick', () => received.push('второй'));

    emitLocal('probe.tick', {});

    expect(received).toEqual(['первый', 'второй']);
  });

  it('сбой нативной подписки не роняет приложение', async () => {
    const failing: Transport = {
      invoke: () => Promise.resolve({ ok: true, data: null, error: null }),
      listen: () => Promise.reject(new Error('нет ядра')),
    };
    __setTransportForTests(failing);

    const off = on('probe.tick', () => {});
    await Promise.resolve();
    await Promise.resolve();

    // Подписка «мертва», но отписка обязана остаться рабочей.
    expect(() => off()).not.toThrow();
    expect(subscriberCount('probe.tick')).toBe(0);
  });

  it('resetAllSubscriptions очищает все каналы', () => {
    on('a.one', () => {});
    on('b.two', () => {});
    expect(subscriberCount('a.one')).toBe(1);

    resetAllSubscriptions();

    expect(subscriberCount('a.one')).toBe(0);
    expect(subscriberCount('b.two')).toBe(0);
  });
});

describe('реакция на монтирование компонента', () => {
  it('подписка в компоненте снимается при размонтировании', async () => {
    // Симуляция того, что делает useEffect: подписка при монтировании,
    // отписка в очистке. Если AC-3 нарушен, счётчик останется ненулевым.
    const transport = recordingTransport();
    __setTransportForTests(transport);

    const off = on('probe.tick', () => {});
    await Promise.resolve();
    expect(subscriberCount('probe.tick')).toBe(1);

    off(); // очистка эффекта
    await Promise.resolve();

    expect(subscriberCount('probe.tick')).toBe(0);
    expect(transport.listeners.get('probe.tick')?.size ?? 0).toBe(0);
  });
});
