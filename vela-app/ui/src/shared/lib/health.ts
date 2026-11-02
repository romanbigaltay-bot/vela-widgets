/**
 * Связность фронтенда: какие части каркаса реально подключены.
 *
 * Функция, а не константа, чтобы dev-сборка и тесты читали актуальные значения
 * (например, версию контракта IPC после `ipc:gen`).
 */

export interface HealthPart {
  readonly name: string;
  readonly value: string;
}

export function health(): readonly HealthPart[] {
  return [
    { name: 'ui', value: 'ok' },
    { name: 'theme', value: document.documentElement.dataset['theme'] ?? 'unknown' },
    { name: 'renderer', value: 'chromium/WebView2' },
  ];
}
