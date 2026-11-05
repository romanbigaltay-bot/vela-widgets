/**
 * ThemeService (ST-0103-02).
 *
 * # Что делает
 *
 * Разрешает режим (`dark` / `light` / `system`) в конкретную тему, применяет
 * её к `<html data-theme>` и сообщает подписчикам. Тема ставится inline-
 * скриптом в `index.html` до первого кадра, поэтому холодный старт не даёт
 * белой вспышки.
 *
 * # Почему localStorage, а не SQLite, в P1
 *
 * Плана «UI → IPC → SQLite» требует транспорт и команду `app.settings_set`.
 * В P1 Tauri и `InvokeMap` ещё пусты: объявлять команду «на будущее» запрещено
 * (AGENTS.md). Поэтому сейчас режим переживает перезапуск через localStorage,
 * а `storage`-событие синхронизирует несколько поверхностей одного origin.
 * В P2 `setThemeFromHost` подключается к `app.theme_changed`, а запись — к
 * типизированной команде; это зафиксировано в ST-0103-02.
 */

import { log } from '@/shared/lib/log';

export type ThemeMode = 'dark' | 'light' | 'system';
export type ResolvedTheme = 'dark' | 'light';

export const THEME_STORAGE_KEY = 'vela.theme';

type ThemeListener = () => void;

const listeners = new Set<ThemeListener>();

function isMode(value: string | null | undefined): value is ThemeMode {
  return value === 'dark' || value === 'light' || value === 'system';
}

function isResolvedTheme(value: string | null | undefined): value is ResolvedTheme {
  return value === 'dark' || value === 'light';
}

function storageOrNull(): Storage | null {
  if (typeof window === 'undefined') return null;
  try {
    return window.localStorage;
  } catch {
    // Приватный режим или запрет хранилища — тема всё равно должна работать.
    return null;
  }
}

/** Режим из localStorage; `null`, если значения нет или оно испорчено. */
export function readStoredMode(): ThemeMode | null {
  const storage = storageOrNull();
  if (!storage) return null;
  try {
    const value = storage.getItem(THEME_STORAGE_KEY);
    return isMode(value) ? value : null;
  } catch {
    return null;
  }
}

/**
 * Текущий режим. Если localStorage недоступен, используется атрибут, который
 * выставил inline-скрипт: так режим не теряется после перезагрузки.
 */
export function getMode(): ThemeMode {
  const stored = readStoredMode();
  if (stored) return stored;

  if (typeof document !== 'undefined') {
    const fromDom = document.documentElement.dataset['themeMode'];
    if (isMode(fromDom)) return fromDom;
  }

  return 'system';
}

function systemTheme(): ResolvedTheme {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'dark';
  }
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

/** Разрешает `system` в фактическую тему; чистый режим возвращается как есть. */
export function resolveTheme(mode: ThemeMode): ResolvedTheme {
  return mode === 'system' ? systemTheme() : mode;
}

/** Фактическая тема, уже применённая к документу (если он есть). */
export function currentTheme(): ResolvedTheme {
  if (typeof document !== 'undefined') {
    const fromDom = document.documentElement.dataset['theme'];
    if (isResolvedTheme(fromDom)) return fromDom;
  }
  return resolveTheme(getMode());
}

function applyResolved(theme: ResolvedTheme): void {
  if (typeof document === 'undefined') return;
  document.documentElement.dataset['theme'] = theme;
  // Нативные контролы WebView2 (скроллбары, поля) должны знать тему.
  document.documentElement.style.colorScheme = theme;
}

function notify(): void {
  for (const listener of [...listeners]) {
    try {
      listener();
    } catch (err) {
      // Один упавший подписчик не должен оставлять остальные с устаревшей темой.
      log('error', 'подписчик ThemeService упал', err);
    }
  }
}

/**
 * Подписывает на изменения темы. Возвращает синхронную отписку.
 */
export function subscribe(listener: ThemeListener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/**
 * Устанавливает режим, сохраняет его и применяет. `system` — не тема, а
 * режим: фактическая тема вычисляется из `prefers-color-scheme`.
 */
export function setMode(mode: ThemeMode): void {
  const storage = storageOrNull();
  if (storage) {
    try {
      storage.setItem(THEME_STORAGE_KEY, mode);
    } catch (err) {
      // Тема применится, но после перезапуска вернётся системная.
      log('warn', 'не удалось сохранить режим темы', err);
    }
  }

  if (typeof document !== 'undefined') {
    document.documentElement.dataset['themeMode'] = mode;
  }
  applyResolved(resolveTheme(mode));
  notify();
}

/** Переключает между светлой и тёмной, фиксируя явный режим. */
export function toggle(): ResolvedTheme {
  const next: ResolvedTheme = currentTheme() === 'dark' ? 'light' : 'dark';
  setMode(next);
  return next;
}

/**
 * Применяет тему, пришедшую от ядра (P2: событие `app.theme_changed`).
 *
 * Режим не меняется: если пользователь выбрал `system`, ядро присылает только
 * фактическую тему, а не подменяет выбор.
 */
export function setThemeFromHost(theme: ResolvedTheme): void {
  applyResolved(theme);
  notify();
}

/**
 * Инициализация после загрузки модулей: применяет текущий режим и подписывается
 * на системные изменения и `storage` (синхронизация поверхностей).
 *
 * Возвращает функцию очистки — обязательна в тестах и при закрытии окна.
 */
export function initTheme(): () => void {
  const mode = getMode();
  if (typeof document !== 'undefined') {
    document.documentElement.dataset['themeMode'] = mode;
  }
  applyResolved(resolveTheme(mode));

  if (typeof window === 'undefined') {
    return () => {};
  }

  const media =
    typeof window.matchMedia === 'function'
      ? window.matchMedia('(prefers-color-scheme: light)')
      : null;

  const onSystemChange = (): void => {
    if (getMode() !== 'system') return;
    applyResolved(systemTheme());
    notify();
  };

  if (media) {
    if (typeof media.addEventListener === 'function') {
      media.addEventListener('change', onSystemChange);
    } else if (typeof media.addListener === 'function') {
      media.addListener(onSystemChange);
    }
  }

  const onStorage = (event: StorageEvent): void => {
    if (event.key !== null && event.key !== THEME_STORAGE_KEY) return;
    const nextMode: ThemeMode = isMode(event.newValue) ? event.newValue : 'system';
    if (typeof document !== 'undefined') {
      document.documentElement.dataset['themeMode'] = nextMode;
    }
    applyResolved(resolveTheme(nextMode));
    notify();
  };

  window.addEventListener('storage', onStorage);

  return () => {
    if (media) {
      if (typeof media.removeEventListener === 'function') {
        media.removeEventListener('change', onSystemChange);
      } else if (typeof media.removeListener === 'function') {
        media.removeListener(onSystemChange);
      }
    }
    window.removeEventListener('storage', onStorage);
  };
}
