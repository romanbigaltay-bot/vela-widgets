/**
 * Тесты каркаса фронтенда.
 *
 * Три вещи, которые должны быть сломаны, если каркас собран неправильно:
 * 1) алиас `@/` резолвится и в vite, и в vitest;
 * 2) приложение рендерится без ошибок;
 * 3) тема читается из DOM, а не из константы.
 */
import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { App } from '@/app/App';
import { health } from '@/shared/lib/health';

describe('каркас', () => {
  it('резолвит алиас @/ в тестах', () => {
    expect(typeof health).toBe('function');
  });

  it('рендерит заголовок и список подключённых частей', () => {
    render(<App />);
    expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Vela');
    expect(screen.getByText('ui')).toBeTruthy();
  });

  it('читает тему из data-theme на html', () => {
    document.documentElement.dataset['theme'] = 'light';
    const theme = health().find((p) => p.name === 'theme');
    expect(theme?.value).toBe('light');
    document.documentElement.dataset['theme'] = 'dark';
  });

  it('после повторного рендера остаётся ровно один заголовок в контейнере', () => {
    // `render` из testing-library создаёт новый контейнер на каждый вызов,
    // поэтому два вызова дают два заголовка во всём документе — это корректно.
    // Проверяем то, что действительно является инвариантом компонента:
    // внутри своего контейнера App рендерит ровно один h1.
    const first = render(<App />);
    expect(first.container.querySelectorAll('h1')).toHaveLength(1);
    // И не оставляет мусора при повторном рендере в тот же контейнер.
    first.rerender(<App />);
    expect(first.container.querySelectorAll('h1')).toHaveLength(1);
  });
});
