/**
 * Проверка анти-вспышки: inline-скрипт из index.html исполняется до бандла и
 * корректно разрешает режим `system` в конкретную тему. Тест выполняет сам
 * скрипт в изолированных моках, а не ищет строки в файле.
 */

import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

/**
 * `import.meta.url` под Vitest — не file-URL, поэтому путь ищем от cwd.
 * Кандидаты покрывают запуск из `vela-app/ui` и из корня монорепозитория.
 */
function findIndexHtml(): string {
  const candidates = [
    resolve(process.cwd(), 'index.html'),
    resolve(process.cwd(), 'vela-app', 'ui', 'index.html'),
  ];
  const found = candidates.find((file) => existsSync(file));
  if (!found) throw new Error(`не найден index.html; cwd=${process.cwd()}`);
  return found;
}

const html = readFileSync(findIndexHtml(), 'utf8');
const inlineScript = /<script>([\s\S]*?)<\/script>/.exec(html)?.[1] ?? '';

interface BootOptions {
  saved?: string | null;
  systemLight?: boolean;
  throwStorage?: boolean;
}

interface BootResult {
  theme: string | undefined;
  mode: string | undefined;
  colorScheme: string | undefined;
}

function runBoot(options: BootOptions): BootResult {
  const dataset: Record<string, string> = {};
  const style: Record<string, string> = {};
  const documentMock = { documentElement: { dataset, style } };
  const windowMock = {
    matchMedia: () => ({ matches: options.systemLight === true }),
  };
  const storageMock = {
    getItem: (): string | null => {
      if (options.throwStorage === true) throw new Error('storage disabled');
      return options.saved ?? null;
    },
  };

  new Function('localStorage', 'document', 'window', inlineScript)(
    storageMock,
    documentMock,
    windowMock,
  );

  return {
    theme: dataset['theme'],
    mode: dataset['themeMode'],
    colorScheme: style['colorScheme'],
  };
}

describe('анти-вспышка темы', () => {
  it('inline-скрипт стоит до модуля и не тянет внешние шрифты', () => {
    expect(inlineScript).not.toBe('');
    expect(html.indexOf('<script>')).toBeLessThan(html.indexOf('type="module"'));
    expect(html).not.toMatch(/fonts\.(googleapis|gstatic)\.com/);
  });

  it('применяет сохранённую светлую тему', () => {
    expect(runBoot({ saved: 'light', systemLight: false })).toEqual({
      theme: 'light',
      mode: 'light',
      colorScheme: 'light',
    });
  });

  it('разрешает system в светлую тему, сохраняя режим', () => {
    expect(runBoot({ saved: 'system', systemLight: true })).toEqual({
      theme: 'light',
      mode: 'system',
      colorScheme: 'light',
    });
  });

  it('не ставит data-theme=system', () => {
    const result = runBoot({ saved: 'system', systemLight: false });
    expect(result.theme).toBe('dark');
    expect(result.mode).toBe('system');
  });

  it('игнорирует испорченное значение и падает в системный режим', () => {
    expect(runBoot({ saved: 'neon', systemLight: false })).toEqual({
      theme: 'dark',
      mode: 'system',
      colorScheme: 'dark',
    });
  });

  it('работает при запрещённом localStorage', () => {
    expect(runBoot({ throwStorage: true, systemLight: false })).toEqual({
      theme: 'dark',
      mode: 'system',
      colorScheme: 'dark',
    });
  });
});
