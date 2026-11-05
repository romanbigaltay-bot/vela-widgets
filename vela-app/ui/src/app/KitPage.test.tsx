import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { KitPage } from './KitPage';

beforeEach(() => {
  localStorage.clear();
  document.documentElement.removeAttribute('data-theme');
  document.documentElement.removeAttribute('data-theme-mode');
  document.documentElement.style.colorScheme = '';
});

describe('витрина ui-kit', () => {
  it('использует все семь размеров типографической шкалы', () => {
    const { container } = render(<KitPage />);

    for (const className of [
      't-hero',
      't-display',
      't-title',
      't-body',
      't-label',
      't-caption',
      't-micro',
    ]) {
      expect(container.querySelectorAll(`.${className}`).length).toBeGreaterThan(0);
    }
  });

  it('переключает тему кнопками и сохраняет режим', () => {
    render(<KitPage />);

    fireEvent.click(screen.getByRole('button', { name: 'Светлая' }));

    expect(document.documentElement.dataset['theme']).toBe('light');
    expect(document.documentElement.dataset['themeMode']).toBe('light');
    expect(localStorage.getItem('vela.theme')).toBe('light');
    expect(screen.getByRole('button', { name: 'Светлая' }).getAttribute('aria-pressed')).toBe(
      'true',
    );
  });

  it('переключает Switch по клику', () => {
    render(<KitPage />);
    const toggle = screen.getByRole('switch', { name: 'Уведомления' });

    expect(screen.getByText('вкл')).toBeTruthy();
    fireEvent.click(toggle);

    expect(screen.getByText('выкл')).toBeTruthy();
    expect(toggle.getAttribute('aria-checked')).toBe('false');
  });
});
