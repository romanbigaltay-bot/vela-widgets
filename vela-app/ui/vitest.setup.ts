/**
 * Общая настройка тестов: jsdom, матчеры, localStorage.
 *
 * `@testing-library/jest-dom` намеренно не подключён: он тянет лишние 200 КБ
 * и меняет глобальные матчеры, а нам в P1 достаточно `expect` из vitest.
 * Когда появятся компонентные тесты (P4), матчеры добавятся точечно.
 */
import { afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

/**
 * Node 22+ и 25 предоставляют собственный экспериментальный `localStorage`.
 * Без валидного `--localstorage-file` он не работает (`clear is not a
 * function`), и Vitest выставляет его глобально вместо jsdom-реализации.
 * Тесты не должны зависеть от флагов Node, поэтому подменяем хранилище на
 * явную in-memory реализацию — одинаковую для `globalThis` и `window`.
 */
function createMemoryStorage(): Storage {
  const data = new Map<string, string>();

  return {
    get length(): number {
      return data.size;
    },
    clear(): void {
      data.clear();
    },
    getItem(key: string): string | null {
      const value = data.get(String(key));
      return value === undefined ? null : value;
    },
    key(index: number): string | null {
      return [...data.keys()][index] ?? null;
    },
    removeItem(key: string): void {
      data.delete(String(key));
    },
    setItem(key: string, value: string): void {
      data.set(String(key), String(value));
    },
  } as Storage;
}

function installMemoryStorage(): void {
  if (typeof window === 'undefined') return;
  const storage = createMemoryStorage();

  try {
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: storage,
    });
  } catch {
    // jsdom может запретить переопределение — тогда остаётся его реализация.
  }

  try {
    Object.defineProperty(globalThis, 'localStorage', {
      configurable: true,
      value: window.localStorage,
    });
  } catch {
    // В старых Node глобального localStorage нет — и не нужно.
  }
}

installMemoryStorage();

afterEach(() => {
  cleanup();
});
