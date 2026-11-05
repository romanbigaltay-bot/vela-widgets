import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  THEME_STORAGE_KEY,
  currentTheme,
  getMode,
  initTheme,
  readStoredMode,
  resolveTheme,
  setMode,
  setThemeFromHost,
  subscribe,
  toggle,
} from './theme';

type ChangeListener = (event: MediaQueryListEvent) => void;

let systemLight = false;
let mediaListeners = new Set<ChangeListener>();
const cleanups: Array<() => void> = [];

function installMatchMedia(): void {
  mediaListeners = new Set();
  const media = {
    get matches(): boolean {
      return systemLight;
    },
    media: '(prefers-color-scheme: light)',
    onchange: null,
    addEventListener: (_type: string, listener: ChangeListener): void => {
      mediaListeners.add(listener);
    },
    removeEventListener: (_type: string, listener: ChangeListener): void => {
      mediaListeners.delete(listener);
    },
    addListener: (listener: ChangeListener): void => {
      mediaListeners.add(listener);
    },
    removeListener: (listener: ChangeListener): void => {
      mediaListeners.delete(listener);
    },
    dispatchEvent: (): boolean => true,
  };

  Object.defineProperty(window, 'matchMedia', {
    configurable: true,
    writable: true,
    value: () => media,
  });
}

function emitSystemChange(light: boolean): void {
  systemLight = light;
  const event = {
    matches: light,
    media: '(prefers-color-scheme: light)',
  } as unknown as MediaQueryListEvent;
  for (const listener of [...mediaListeners]) {
    listener(event);
  }
}

function startTheme(): void {
  cleanups.push(initTheme());
}

beforeEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute('data-theme');
  document.documentElement.removeAttribute('data-theme-mode');
  document.documentElement.style.colorScheme = '';
  systemLight = false;
  installMatchMedia();
});

afterEach(() => {
  while (cleanups.length > 0) {
    cleanups.pop()?.();
  }
  vi.restoreAllMocks();
  localStorage.clear();
});

describe('ThemeService', () => {
  it('по умолчанию использует системный режим', () => {
    expect(getMode()).toBe('system');
    expect(resolveTheme('system')).toBe('dark');
    systemLight = true;
    expect(resolveTheme('system')).toBe('light');
  });

  it('сохраняет режим и применяет фактическую тему', () => {
    setMode('light');

    expect(readStoredMode()).toBe('light');
    expect(getMode()).toBe('light');
    expect(currentTheme()).toBe('light');
    expect(document.documentElement.dataset['theme']).toBe('light');
    expect(document.documentElement.dataset['themeMode']).toBe('light');
    expect(document.documentElement.style.colorScheme).toBe('light');
  });

  it('переключает тему и фиксирует явный режим', () => {
    setMode('dark');

    expect(toggle()).toBe('light');
    expect(getMode()).toBe('light');
    expect(toggle()).toBe('dark');
    expect(getMode()).toBe('dark');
  });

  it('игнорирует испорченное значение в localStorage', () => {
    localStorage.setItem(THEME_STORAGE_KEY, 'neon');

    expect(readStoredMode()).toBeNull();
    expect(getMode()).toBe('system');
  });

  it('уведомляет подписчиков и снимает подписку', () => {
    const listener = vi.fn();
    const unsubscribe = subscribe(listener);

    setMode('light');
    expect(listener).toHaveBeenCalledTimes(1);

    unsubscribe();
    setMode('dark');
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it('подхватывает системную тему в режиме system', () => {
    startTheme();
    expect(document.documentElement.dataset['theme']).toBe('dark');

    emitSystemChange(true);

    expect(document.documentElement.dataset['theme']).toBe('light');
    expect(document.documentElement.dataset['themeMode']).toBe('system');
  });

  it('не подменяет явно выбранную тему системным событием', () => {
    setMode('dark');
    startTheme();

    emitSystemChange(true);

    expect(document.documentElement.dataset['theme']).toBe('dark');
  });

  it('синхронизирует тему из storage-события', () => {
    startTheme();

    localStorage.setItem(THEME_STORAGE_KEY, 'light');
    window.dispatchEvent(
      new StorageEvent('storage', { key: THEME_STORAGE_KEY, newValue: 'light' }),
    );

    expect(getMode()).toBe('light');
    expect(document.documentElement.dataset['theme']).toBe('light');
  });

  it('снимает слушателей при очистке', () => {
    const cleanup = initTheme();
    cleanup();

    emitSystemChange(true);

    expect(document.documentElement.dataset['theme']).toBe('dark');
    expect(mediaListeners.size).toBe(0);
  });

  it('применяет тему от ядра, не подменяя выбранный режим', () => {
    setMode('system');

    setThemeFromHost('light');

    expect(getMode()).toBe('system');
    expect(currentTheme()).toBe('light');
  });
});
