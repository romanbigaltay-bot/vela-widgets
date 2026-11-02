/**
 * Транспорт: то, что умеет доставлять команды в ядро и принимать события.
 *
 * # Зачем абстракция, а не прямой `invoke`
 *
 * `invoke` существует только внутри Tauri: в браузере (`pnpm dev` без окна),
 * в vitest и в скриншот-тестах его нет. Прямой вызов в компоненте падает с
 * `invoke is not a function` — сообщение, по которому невозможно понять, что
 * приложение запущено не в том окружении.
 *
 * Вторая причина: обёртку надо тестировать. С транспортом-интерфейсом тест
 * подставляет фейк и проверяет разворачивание конверта, ошибки и тайминги,
 * не поднимая WebView2.
 *
 * # Почему `invoke` возвращает `unknown`, а не `Envelope<T>`
 *
 * Транспорт не должен знать про контракт: его задача — доставить и вернуть
 * сырой ответ. Разбор конверта, проверка согласованности и превращение в
 * `VelaFailure` живут в `client.ts`. Иначе тест транспорта пришлось бы писать
 * против типа конверта, а тест клиента — против формы ответа.
 */

/** Отписка от события. Идемпотентна: повторный вызов ничего не делает. */
export type Unsubscribe = () => void;

/** Обработчик события. Payload на этом уровне неизвестен: типизацию даёт шина. */
export type RawHandler = (payload: unknown) => void;

/** Доставка команд и событий между UI и ядром. */
export interface Transport {
  /** Отправляет команду и возвращает сырой ответ ядра. */
  invoke(command: string, payload: unknown): Promise<unknown>;

  /** Подписывается на событие; возвращает функцию отписки. */
  listen(event: string, handler: RawHandler): Promise<Unsubscribe>;
}

/**
 * Транспорт, работающий через Tauri.
 *
 * Модуль `@tauri-apps/api` импортируется динамически: при запуске в браузере
 * его загрузка не должна ломать приложение целиком, а должна давать понятную
 * ошибку на конкретном вызове.
 */
export const tauriTransport: Transport = {
  async invoke(command, payload) {
    const core = await import('@tauri-apps/api/core');
    return core.invoke(command, payload as Record<string, unknown> | undefined);
  },

  async listen(event, handler) {
    const ev = await import('@tauri-apps/api/event');
    const unlisten = await ev.listen(event, (e) => handler(e.payload));
    return () => {
      // `unlisten` возвращает Promise; отписка обязана быть синхронной,
      // поэтому ошибку гасим осознанно: при закрытии окна слушатель уже
      // уничтожен вместе с процессом, и ждать тут нечего.
      void unlisten();
    };
  },
};

/**
 * Транспорт для окружений без Tauri.
 *
 * Не бросает при создании: приложение в браузере должно отрисоваться и
 * показать интерфейс, а не белый экран. Ошибка возникает на конкретном
 * вызове, с кодом `platform_unavailable` — по нему UI решает, показать
 * заглушку или просто проигнорировать фоновой запрос.
 */
export const unavailableTransport: Transport = {
  invoke() {
    return Promise.reject(
      new Error('Tauri недоступен: приложение запущено вне окна (браузер или тест)'),
    );
  },
  listen() {
    // Подписка без ядра — не ошибка: события просто никогда не придут.
    // Возвращать reject здесь значило бы требовать от каждого компонента
    // обработки случая, который ничего не меняет.
    return Promise.resolve(() => {});
  },
};

/**
 * Выбирает транспорт по окружению.
 *
 * Проверка `__TAURI_INTERNALS__`, а не `__TAURI__`: в Tauri 2 глобальный объект
 * называется именно так, и `withGlobalTauri: false` (наш случай) оставляет
 * доступным только его.
 */
export function detectTransport(): Transport {
  const hasTauri =
    typeof globalThis !== 'undefined' &&
    '__TAURI_INTERNALS__' in (globalThis as Record<string, unknown>);
  return hasTauri ? tauriTransport : unavailableTransport;
}
