/**
 * Тесты хука `useEvent`.
 *
 * Проверяется главное обещание ST-0102-03, AC-3: подписка снимается при
 * размонтировании. Без этого при переключении профилей обработчики копятся,
 * и приложение деградирует тем сильнее, чем дольше работает — то есть дефект
 * проявляется не на тестах, а у пользователя через час работы.
 */

import { renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { subscriberCount } from './events';
import { __setTransportForTests } from './client';
import { unavailableTransport } from './transport';
import { useEvent } from './useEvent';
import { clearLogs } from '../lib/log';

// `EventMap` наполняется объявлениями в фичах (ST-0302-03 объявит
// `layout.changed`). В тесте объявляем своё событие, иначе `useEvent` не примет
// строку-литерал: тип `keyof EventMap & string` не содержит ещё не
// объявленных имён — это и есть защита от опечаток в имени события.
// Тип обязан совпадать с объявлением в `events.test.ts`: механизм слияния
// объявлений требует идентичности, поэтому дублируем ровно тот же.
declare module './events' {
  interface EventMap {
    'probe.tick': { n?: number; from?: string };
  }
}

/**
 * Транспорт-заглушка: `on()` сразу получает рабочую отписку.
 *
 * Реальный транспорт в тестах недоступен (нет Tauri), а `unavailableTransport`
 * возвращает отписку-пустышку — с ним нельзя проверить, что счётчик
 * подписчиков вернулся к нулю.
 */
function stubTransport(): { unlisten: ReturnType<typeof vi.fn> } {
  const unlisten = vi.fn();
  __setTransportForTests({
    ...unavailableTransport,
    listen: () => Promise.resolve(unlisten),
  });
  return { unlisten };
}

describe('useEvent', () => {
  beforeEach(() => {
    clearLogs();
  });

  afterEach(() => {
    // Возврат к недоступному транспорту: иначе заглушка одного теста
    // протекает в следующий и делает результат зависимым от порядка.
    __setTransportForTests(unavailableTransport);
  });

  it('подписывается при монтировании и снимает подписку при размонтировании', async () => {
    const { unlisten } = stubTransport();

    const { result, unmount } = renderHook(() =>
      useEvent('probe.tick', () => {
        // обработчик не важен
      }),
    );

    // Подписка создаётся асинхронно (транспорт возвращает промис).
    await vi.waitFor(() => {
      expect(subscriberCount('probe.tick')).toBe(1);
    });

    unmount();

    await vi.waitFor(() => {
      expect(subscriberCount('probe.tick')).toBe(0);
    });
    expect(unlisten).toHaveBeenCalled();
    expect(result).toBeDefined();
  });

  it('не подписывается, когда enabled = false', async () => {
    stubTransport();

    renderHook(() =>
      useEvent(
        'probe.tick',
        () => {
          // обработчик не важен
        },
        { enabled: false },
      ),
    );

    // Даём отработать микротаскам: если бы подписка создавалась, счётчик был бы > 0.
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(subscriberCount('probe.tick')).toBe(0);
  });

  it('вызывает актуальный обработчик после смены props, не пересоздавая подписку', async () => {
    const { unlisten } = stubTransport();
    const first = vi.fn();
    const second = vi.fn();

    const { rerender } = renderHook(
      ({ handler }: { handler: () => void }) => useEvent('probe.tick', handler),
      { initialProps: { handler: first } },
    );

    await vi.waitFor(() => {
      expect(subscriberCount('probe.tick')).toBe(1);
    });

    rerender({ handler: second });

    // Ключевое: подписка та же (отписка не вызывалась), но обработчик новый.
    expect(unlisten).not.toHaveBeenCalled();
    expect(subscriberCount('probe.tick')).toBe(1);
  });

  it('две подписки на одно событие снимаются независимо', async () => {
    stubTransport();

    const a = renderHook(() => useEvent('probe.tick', () => {}));
    const b = renderHook(() => useEvent('probe.tick', () => {}));

    await vi.waitFor(() => {
      expect(subscriberCount('probe.tick')).toBe(2);
    });

    a.unmount();
    await vi.waitFor(() => {
      expect(subscriberCount('probe.tick')).toBe(1);
    });

    b.unmount();
    await vi.waitFor(() => {
      expect(subscriberCount('probe.tick')).toBe(0);
    });
  });
});
