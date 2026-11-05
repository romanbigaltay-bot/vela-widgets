import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { log } from '@/shared/lib/log';
import { Glass } from './Glass';

vi.mock('@/shared/lib/log', () => ({ log: vi.fn() }));

describe('Glass', () => {
  it('собирает классы из пропсов', () => {
    render(
      <Glass padding="lg" radius="inner" elevation={2} interactive>
        Панель
      </Glass>,
    );
    const panel = screen.getByText('Панель');

    expect(panel.className).toContain('glass');
    expect(panel.className).toContain('glass--pad-lg');
    expect(panel.className).toContain('glass--radius-inner');
    expect(panel.className).toContain('glass--elev-2');
    expect(panel.className).toContain('glass--interactive');
  });

  it('предупреждает о вложенном стекле в dev', () => {
    render(
      <Glass>
        <Glass>Внутри</Glass>
      </Glass>,
    );

    expect(vi.mocked(log)).toHaveBeenCalledWith('warn', expect.stringContaining('вложен'));
  });

  it('не предупреждает для соседних панелей', () => {
    vi.mocked(log).mockClear();
    render(
      <>
        <Glass>Первая</Glass>
        <Glass>Вторая</Glass>
      </>,
    );

    expect(vi.mocked(log)).not.toHaveBeenCalledWith('warn', expect.stringContaining('вложен'));
  });
});
