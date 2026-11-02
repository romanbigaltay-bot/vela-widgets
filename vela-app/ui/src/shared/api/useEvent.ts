/**
 * Хук подписки на событие ядра.
 *
 * # Зачем отдельный хук
 *
 * `on()` возвращает синхронную отписку, но в `useEffect` её легко забыть
 * вернуть — и тогда подписка живёт до перезагрузки страницы. При переключении
 * профилей и смене раскладки это накапливается: обработчиков становится всё
 * больше, каждый ререндерит свой компонент, и приложение начинает тормозить
 * тем сильнее, чем дольше работает.
 *
 * Хук делает отписку обязательной частью контракта, а не дисциплиной
 * разработчика (ST-0102-03, AC-3).
 *
 * # Почему обработчик хранится в ref
 *
 * Если подписываться на каждое изменение `handler`, подписка будет
 * пересоздаваться на каждом рендере — а вместе с ней может прийти и потеря
 * события в момент отписки. Через ref подписка создаётся один раз, а свежий
 * обработчик вызывается всегда.
 */

import { useEffect, useRef } from 'react';

import { on } from './events';
import type { EventMap, Handler } from './events';

/**
 * Подписывается на событие и отписывается при размонтировании.
 *
 * @example
 * ```tsx
 * useEvent('layout.changed', (payload) => applySnapshot(payload));
 * ```
 */
export function useEvent<E extends keyof EventMap & string>(
  event: E,
  handler: Handler<E>,
  options: { enabled?: boolean } = {},
): void {
  const { enabled = true } = options;

  // Ref хранит последнюю версию обработчика, чтобы подписка не пересоздавалась.
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    // `on` уже возвращает синхронную отписку — возвращаем её напрямую,
    // чтобы React снял подписку при размонтировании.
    return on(event, (payload: EventMap[E]) => {
      handlerRef.current(payload);
    });
  }, [event, enabled]);
}
