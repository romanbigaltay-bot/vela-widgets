/**
 * Шина событий: типизированные подписки от ядра к UI.
 *
 * # Почему шина, а не прямой `listen` в компонентах
 *
 * У события может быть несколько подписчиков (раскладку слушают все поверхности
 * монитора, тему — все слои). Прямой `listen` создавал бы по нативному
 * слушателю на компонент, и каждый пришлось бы снимать вручную — самая частая
 * утечка в подобных приложениях.
 *
 * Здесь один нативный слушатель на событие, подписчики — внутри процесса.
 * Когда уходит последний подписчик, нативный слушатель снимается: ядро
 * перестаёт слать событие, на которое никто не смотрит.
 *
 * # Почему подписка и отписка не полагаются на промисы
 *
 * `transport.listen()` асинхронный: нативный слушатель появляется только
 * после загрузки модуля Tauri. Первая версия шины делала отписку через
 * `release.then(unlisten)` — и при быстром цикле «подписался-отписался»
 * отписка не успевала выполниться, оставляя живой нативный слушатель
 * (замер: 50 циклов давали 49 отписок и 1 утёкший слушатель).
 *
 * Исправление: состояние канала хранит три флага вместо одного промиса —
 * `pending`, `unlisten`, `disposeRequested`. Отписка выполняется синхронно,
 * если функция уже получена, и откладывается, если ещё нет. Любое число
 * циклов не оставляет слушателей, независимо от таймингов.
 */

import type { Unsubscribe } from './transport';
import { currentTransport } from './client';
import { log } from '../lib/log';

/**
 * Карта событий: имя → тип payload.
 *
 * Расширяется объявлением модуля — так же, как `InvokeMap`. Например,
 * в модуле холста:
 *
 * ```ts
 * declare module '@/shared/api/events' {
 *   interface EventMap {
 *     'layout.changed': { monitor: string; revision: number };
 *   }
 * }
 * ```
 *
 * Пустой интерфейс здесь — не забывчивость, а требование механизма слияния
 * объявлений: `type` слить нельзя, только `interface`.
 */
export interface EventMap {}

/** Обработчик типизированного события. */
export type Handler<E extends keyof EventMap> = (payload: EventMap[E]) => void;

/** Состояние канала события. */
interface Channel {
  readonly handlers: Set<(payload: unknown) => void>;
  /** Нативная отписка, если она уже получена. */
  unlisten: Unsubscribe | null;
  /** Признак «промис подписки ещё не разрешился». */
  pending: boolean;
  /** Отписка запрошена до того, как пришла нативная функция. */
  disposeRequested: boolean;
}

const channels = new Map<string, Channel>();

function channelFor(event: string): Channel {
  let channel = channels.get(event);
  if (!channel) {
    channel = { handlers: new Set(), unlisten: null, pending: false, disposeRequested: false };
    channels.set(event, channel);
  }
  return channel;
}

/**
 * Подписывается на событие.
 *
 * Возвращает синхронную отписку — этим она удобна в `useEffect` и в
 * обработчиках, где ждать промис нельзя.
 */
export function on<E extends keyof EventMap & string>(
  event: E,
  handler: Handler<E>,
): Unsubscribe {
  const channel = channelFor(event);
  const wrapped = (payload: unknown) => {
    try {
      handler(payload as EventMap[E]);
    } catch (err) {
      // Исключение подписчика не должно срывать доставку остальным: у
      // раскладки может быть три подписчика, и падение первого не повод
      // оставить два других без обновления.
      log('error', `обработчик события ${event} упал`, err);
    }
  };

  channel.handlers.add(wrapped);

  if (!channel.pending && channel.unlisten === null && !channel.disposeRequested) {
    channel.pending = true;
    currentTransport()
      .listen(event, (payload) => {
        // Копия набора: обработчик вправе отписаться во время доставки, и
        // мутация Set прямо в цикле пропустила бы следующего подписчика.
        for (const subscriber of [...channel.handlers]) {
          subscriber(payload);
        }
      })
      .then(
        (unlisten) => {
          channel.pending = false;
          if (channel.disposeRequested) {
            // Отписка пришла раньше, чем нативная функция: снимаем сразу,
            // не дожидаясь следующего события.
            unlisten();
            return;
          }
          channel.unlisten = unlisten;
        },
        (err: unknown) => {
          channel.pending = false;
          log('error', `не удалось подписаться на ${event}`, err);
        },
      );
  }

  let subscribed = true;
  return () => {
    if (!subscribed) return;
    subscribed = false;
    channel.handlers.delete(wrapped);

    // Снимаем нативный слушатель, когда смотреть больше некому: иначе ядро
    // продолжит слать событие, а у нас останется живая ссылка на канал.
    if (channel.handlers.size > 0) return;

    channels.delete(event);

    if (channel.unlisten !== null) {
      // Обычный путь: функция уже есть, отписка синхронная.
      channel.unlisten();
      channel.unlisten = null;
    } else if (channel.pending) {
      // Промис ещё не разрешился — отметим и снимем слушатель, когда придёт.
      channel.disposeRequested = true;
    }
  };
}

/**
 * Отправляет событие локальным подписчикам.
 *
 * Нужно для оптимистичных обновлений: UI получает событие, не дожидаясь
 * подтверждения ядра, и не создаёт второй путь доставки.
 */
export function emitLocal<E extends keyof EventMap & string>(
  event: E,
  payload: EventMap[E],
): void {
  const channel = channels.get(event);
  if (!channel) return;
  for (const subscriber of [...channel.handlers]) {
    subscriber(payload);
  }
}

/**
 * Снимает все подписки.
 *
 * Вызывается при закрытии поверхности и в тестах: глобальное состояние шины
 * переживает размонтирование компонентов, и без сброса тесты влияли бы друг
 * на друга.
 */
export function resetAllSubscriptions(): void {
  for (const [event, channel] of channels) {
    channel.handlers.clear();
    if (channel.unlisten !== null) {
      channel.unlisten();
      channel.unlisten = null;
    } else if (channel.pending) {
      channel.disposeRequested = true;
    }
    channels.delete(event);
  }
}

/** Сколько подписчиков у события — для тестов на утечки. */
export function subscriberCount(event: string): number {
  return channels.get(event)?.handlers.size ?? 0;
}
