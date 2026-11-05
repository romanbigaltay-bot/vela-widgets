import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Switch } from './Switch';

describe('Switch', () => {
  it('отражает состояние через aria-checked', () => {
    render(<Switch checked={false} onCheckedChange={() => {}} label="Звук" />);

    expect(screen.getByRole('switch', { name: 'Звук' }).getAttribute('aria-checked')).toBe('false');
  });

  it('передаёт новое состояние по клику', () => {
    const onCheckedChange = vi.fn();
    render(<Switch checked={false} onCheckedChange={onCheckedChange} label="Звук" />);

    fireEvent.click(screen.getByRole('switch', { name: 'Звук' }));

    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });

  it('не переключается в disabled', () => {
    const onCheckedChange = vi.fn();
    render(<Switch checked={false} onCheckedChange={onCheckedChange} label="Звук" disabled />);

    fireEvent.click(screen.getByRole('switch', { name: 'Звук' }));

    expect(onCheckedChange).not.toHaveBeenCalled();
  });

  it('использует нативную кнопку для Space/Enter', () => {
    render(<Switch checked onCheckedChange={() => {}} label="Звук" />);
    const control = screen.getByRole('switch', { name: 'Звук' });

    expect(control.tagName).toBe('BUTTON');
    expect(control.getAttribute('type')).toBe('button');
  });
});
