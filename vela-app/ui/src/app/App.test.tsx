/**
 * Тесты каркаса фронтенда.
 *
 * Три вещи, которые должны быть сломаны, если каркас собран неправильно:
 * 1) алиас `@/` резолвится и в vite, и в vitest;
 * 2) приложение рендерится без ошибок;
 * 3) тема читается из DOM, а не из константы.
 *
 * Панель поверхностей вызывает IPC, поэтому транспорт подменяется: без этого
 * тест зависел бы от того, запущен ли он внутри Tauri.
 */
import { beforeEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { App } from '@/app/App';
import type { SurfaceStatus } from '@/shared/api';
import { __setTransportForTests } from '@/shared/api/client';
import { resetAllSubscriptions } from '@/shared/api/events';
import type { Transport } from '@/shared/api/transport';
import { health } from '@/shared/lib/health';

const SURFACES: SurfaceStatus[] = [
  { kind: 'ui', monitor: 'primary', state: 'hidden', width: 800, height: 600 },
];

/** Транспорт, который отвечает снимком поверхностей и не шлёт события. */
function surfacesTransport(): Transport {
  return {
    invoke: () => Promise.resolve({ ok: true, data: SURFACES, error: null }),
    listen: () => Promise.resolve(() => {}),
  };
}

beforeEach(() => {
  resetAllSubscriptions();
  __setTransportForTests(surfacesTransport());
});

describe('каркас', () => {
  it('резолвит алиас @/ в тестах', () => {
    expect(typeof health).toBe('function');
  });

  it('рендерит заголовок и список подключённых частей', async () => {
    render(<App />);
    expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Vela');
    expect(screen.getByText('ui')).toBeTruthy();
    // Панель поверхностей отвечает асинхронно; ждём её, иначе обновление
    // состояния придёт после конца теста и React справедливо предупредит.
    expect(await screen.findByText('Виджеты')).toBeTruthy();
  });

  it('показывает поверхность из ответа ядра', async () => {
    render(<App />);
    expect(await screen.findByText('Виджеты')).toBeTruthy();
    expect(screen.getByText('primary')).toBeTruthy();
    expect(screen.getByText('скрыта')).toBeTruthy();
  });

  it('читает тему из data-theme на html', () => {
    document.documentElement.dataset['theme'] = 'light';
    const theme = health().find((p) => p.name === 'theme');
    expect(theme?.value).toBe('light');
    document.documentElement.dataset['theme'] = 'dark';
  });

  it('после повторного рендера остаётся ровно один заголовок в контейнере', async () => {
    // `render` из testing-library создаёт новый контейнер на каждый вызов,
    // поэтому два вызова дают два заголовка во всём документе — это корректно.
    // Проверяем то, что действительно является инвариантом компонента:
    // внутри своего контейнера App рендерит ровно один h1.
    const first = render(<App />);
    expect(first.container.querySelectorAll('h1')).toHaveLength(1);
    // Дожидаемся ответа панели до повторного рендера: иначе обновление
    // состояния придёт уже после теста.
    await screen.findByText('Виджеты');
    // И не оставляет мусора при повторном рендере в тот же контейнер.
    first.rerender(<App />);
    expect(first.container.querySelectorAll('h1')).toHaveLength(1);
  });
});
