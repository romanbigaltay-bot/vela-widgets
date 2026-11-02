# 09 — Контракт виджета

Виджет — это модуль, который объявляет манифест и реализует lifecycle.
Контракт един для встроенных и (в будущем) сторонних модулей.

## 1. Манифест

```ts
export interface WidgetManifest {
  id: string;                    // 'core.clock' — namespace.kebab
  name: string;                  // человекочитаемое, локализуемое
  version: string;               // semver
  sizes: { w: number; h: number }[];  // допустимые размеры в единицах сетки
  defaultSize: { w: number; h: number };
  resizable: boolean;
  minSize?: { w: number; h: number };
  glass?: boolean;               // нужен ли стеклянный контейнер
  permissions: Permission[];     // см. §4
  settingsSchema?: JSONSchema;   // настройки виджета, рендерятся автоматически
  hotkeys?: HotkeyDef[];
  category: 'time' | 'info' | 'capture' | 'media' | 'system' | 'creative';
}
```

## 2. Lifecycle

```ts
export interface WidgetModule<P = unknown> {
  manifest: WidgetManifest;
  Component: React.FC<WidgetProps<P>>;   // чистая презентация
  onMount?(ctx: WidgetContext): void;    // подписки, таймеры
  onUnmount?(ctx: WidgetContext): void;  // обязательная очистка
  onVisibilityChange?(visible: boolean, ctx: WidgetContext): void;
  onSettingsChange?(next: P, prev: P, ctx: WidgetContext): void;
  onProfileDeactivate?(ctx: WidgetContext): void;
}
```

Правила:
- `Component` не имеет права сама создавать `setInterval`, слушать `window` или
  ходить в IPC. Всё — через `ctx`.
- Любой ресурс, созданный в `onMount`, освобождается в `onUnmount` — проверяется
  тестом `ST-0401-03` (изоляция) и `TC-0401-03` (утечки).
- Виджет не знает своих мировых координат. Он знает только свой размер.

## 3. Контекст

```ts
export interface WidgetContext {
  id: string;                       // экземпляр, не тип
  settings: Readonly<P>;
  updateSettings(patch: Partial<P>): Promise<void>;
  storage: { get<T>(k: string): Promise<T|null>; set(k: string, v: unknown): Promise<void> };
  ipc: VelaApi;                     // типизированный, allow-list по permissions
  clock: ClockService;              // единый источник времени (для тестируемости)
  theme: ThemeService;
  visibility: 'visible' | 'occluded' | 'offscreen';
  emit(event: string, payload?: unknown): void;   // в шину (для связей виджетов)
  log: Logger;
}
```

## 4. Права (permissions)

| Право | Что даёт | Кому нужно |
|---|---|---|
| `storage` | своё key-value хранилище | всем |
| `net:http` | исходящие запросы к allow-list доменов | погода, ICS, тикер |
| `fs:read` | чтение разрешённых путей | pin-снимки |
| `system:metrics` | CPU/GPU/RAM/диск | системный монитор |
| `media:control` | управление глобальным плеером | плеер |
| `capture:request` | инициировать скриншот | захват |
| `notify` | системные уведомления | cron, таймер |
| `hotkey:local` | хоткей в пределах приложения | палитра, заметки |

Виджет без объявленного права получает ошибку `PermissionDenied` при вызове,
а не тихий `undefined`.

## 5. Жизненный цикл на холсте

```
смонтирован ──visible──► рендерится
     │                      │
     ├──occluded──► пауза таймеров, рендер 0 fps (только сохранение состояния)
     ├──offscreen─► размонтирован визуально, состояние сохранено
     └──unmount───► onUnmount + очистка ресурсов
```

## 6. Правила производительности для модулей

- Таймер виджета не может тикать чаще, чем нужно: часы — раз в секунду,
  системный монитор — раз в 2 с, визуализатор — 30 fps максимум.
- Анимация обязана останавливаться при `visibility !== 'visible'`.
- Никаких синхронных тяжёлых операций в рендере: сортировка/парсинг — в `useMemo`
  или в воркере.
- Виджет обязан переживать отсутствие данных: показывать скелет, а не падать.

## 7. Тестируемость

`ClockService` инжектируется, поэтому тест может «перевести часы» без ожидания.
Все IPC-вызовы — за интерфейсом, мокаются одним объектом `VelaApi`.
