import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Button, IconButton } from './Button';

describe('Button', () => {
  it('вызывает onClick по клику', () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Сохранить</Button>);

    fireEvent.click(screen.getByRole('button', { name: 'Сохранить' }));

    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('не вызывает onClick в disabled', () => {
    const onClick = vi.fn();
    render(
      <Button onClick={onClick} disabled>
        Сохранить
      </Button>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Сохранить' }));

    expect(onClick).not.toHaveBeenCalled();
  });

  it('остаётся нативной кнопкой: клавиатура обрабатывается браузером', () => {
    render(<Button>OK</Button>);
    const button = screen.getByRole('button', { name: 'OK' });

    expect(button.tagName).toBe('BUTTON');
    expect(button.getAttribute('type')).toBe('button');
    expect(button.tabIndex).toBe(0);
  });

  it('IconButton имеет доступное имя', () => {
    render(<IconButton label="Закрыть">×</IconButton>);

    expect(screen.getByRole('button', { name: 'Закрыть' }).className).toContain('ui-button--icon');
  });
});
