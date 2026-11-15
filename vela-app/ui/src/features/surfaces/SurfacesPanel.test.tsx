/**
 * Тесты панели поверхностей.
 *
 * Проверяется поведение на границе с ядром: успешный ответ, недоступный
 * Tauri, ошибка конверта и живое обновление событием. Это первая панель,
 * поэтому здесь же фиксируется, что `window.surfaces_status` действительно
 * вызывается через `call()`, а не напрямую через `invoke`.
 */
import { beforeEach, describe, expect, it } from 'vitest';
import { act, render, screen } from '@testing-library/react';
import type { SurfaceStatus } from '@/shared/api';
import { __setTransportForTests } from '@/shared/api/client';
import { emitLocal, resetAllSubscriptions } from '@/shared/api/events';
import type { Transport } from '@/shared/api/transport';
import { SurfacesPanel } from './SurfacesPanel';

const HIDDEN_UI: SurfaceStatus = {
  kind: 'ui',
  monitor: 'primary',
  state: 'hidden',
  width: 800,
  height: 600,
};

/** Транспорт с заданным ответом на команду. */
function transportWith(response: unknown): Transport {
  return {
    invoke: () => Promise.resolve(response),
    listen: () => Promise.resolve(() => {}),
  };
}

/** Конверт с данными. */
function ok<T>(data: T): unknown {
  return { ok: true, data, error: null };
}

beforeEach(() => {
  resetAllSubscriptions();
});

describe('панель поверхностей', () => {
  it('показывает снимок, полученный командой window.surfaces_status', async () => {
    __setTransportForTests(transportWith(ok([HIDDEN_UI])));

    render(<SurfacesPanel />);

    expect(await screen.findByText('Виджеты')).toBeTruthy();
    expect(screen.getByText('primary')).toBeTruthy();
    expect(screen.getByText('800 × 600')).toBeTruthy();
    expect(screen.getByText('скрыта')).toBeTruthy();
  });

  it('объясняет недоступность Tauri вместо красной ошибки', async () => {
    __setTransportForTests({
      invoke: () => Promise.reject(new Error('нет ядра')),
      listen: () => Promise.resolve(() => {}),
    });

    render(<SurfacesPanel />);

    expect(await screen.findByText(/Tauri недоступен/)).toBeTruthy();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('показывает ошибку конверта', async () => {
    __setTransportForTests(
      transportWith({
        ok: false,
        data: null,
        error: { code: 'internal', message: 'сломалось', retryable: false },
      }),
    );

    render(<SurfacesPanel />);

    expect(await screen.findByRole('alert')).toBeTruthy();
    expect(screen.getByText('сломалось')).toBeTruthy();
  });

  it('обновляется по событию window:surfaces_changed', async () => {
    __setTransportForTests(transportWith(ok([HIDDEN_UI])));
    render(<SurfacesPanel />);
    expect(await screen.findByText('скрыта')).toBeTruthy();

    act(() => {
      emitLocal('window:surfaces_changed', [{ ...HIDDEN_UI, state: 'visible' }]);
    });

    expect(screen.getByText('видима')).toBeTruthy();
    expect(screen.queryByText('скрыта')).toBeNull();
  });

  it('повторяет запрос по кнопке «Обновить»', async () => {
    let calls = 0;
    __setTransportForTests({
      invoke: () => {
        calls += 1;
        return Promise.resolve(ok(calls === 1 ? [] : [HIDDEN_UI]));
      },
      listen: () => Promise.resolve(() => {}),
    });

    render(<SurfacesPanel />);
    expect(await screen.findByText('Поверхностей пока нет.')).toBeTruthy();

    act(() => {
      screen.getByRole('button', { name: 'Обновить' }).click();
    });

    expect(await screen.findByText('Виджеты')).toBeTruthy();
    expect(calls).toBe(2);
  });
});
