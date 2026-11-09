/**
 * Панель поверхностей: первый живой round-trip IPC.
 *
 * # Зачем она в T-0201
 *
 * Ядро уже создаёт окно поверхности, но UI об этом не знает. Панель вызывает
 * `window.surfaces_status`, показывает ответ и подписывается на
 * `window:surfaces_changed`. Это доказывает всю цепочку
 * «React → call() → Tauri → Rust → Envelope → React» на настоящем окне,
 * а не на моке.
 *
 * # Почему в браузере видна заглушка
 *
 * `pnpm dev` открывается и без Tauri. В этом случае транспорт отвечает
 * `platform_unavailable`; показывать красную ошибку бессмысленно — ядро
 * просто не запущено. Поэтому панель объясняет, где её смотреть.
 */

import { useEffect, useState } from 'react';

import type { SurfaceKind, SurfaceStatus } from '@/shared/api';
import { call, useEvent, VelaFailure } from '@/shared/api';
import { Button, Skeleton } from '@/shared/ui';

import './SurfacesPanel.css';

/** Состояние загрузки панели. */
type PanelState =
  | { readonly status: 'loading' }
  | { readonly status: 'ready'; readonly surfaces: readonly SurfaceStatus[] }
  | { readonly status: 'unavailable'; readonly message: string }
  | { readonly status: 'error'; readonly message: string };

/** Человеческие названия типов поверхностей. */
const KIND_LABELS: Record<SurfaceKind, string> = {
  scene: 'Сцена',
  ui: 'Виджеты',
  overlay: 'Оверлей',
};

/** Подпись видимости. */
function stateLabel(status: SurfaceStatus): string {
  return status.state === 'visible' ? 'видима' : 'скрыта';
}

export function SurfacesPanel(): React.JSX.Element {
  const [state, setState] = useState<PanelState>({ status: 'loading' });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setState({ status: 'loading' });

    call('window.surfaces_status', undefined)
      .then((surfaces) => {
        if (cancelled) return;
        setState({ status: 'ready', surfaces });
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        if (error instanceof VelaFailure && error.code === 'platform_unavailable') {
          setState({ status: 'unavailable', message: error.message });
          return;
        }
        setState({
          status: 'error',
          message:
            error instanceof VelaFailure
              ? error.message
              : 'Не удалось получить состояние поверхностей.',
        });
      });

    // Ответ может прийти после размонтирования: обновлять состояние уже
    // некому, а `setState` на мёртвом компоненте — утечка и лишний рендер.
    return () => {
      cancelled = true;
    };
  }, [attempt]);

  useEvent('window:surfaces_changed', (surfaces) => {
    setState({ status: 'ready', surfaces });
  });

  return (
    <section className="surfaces" aria-labelledby="surfaces-title">
      <header className="surfaces__header">
        <div>
          <h2 id="surfaces-title" className="t-title">
            Поверхности
          </h2>
          <p className="surfaces__hint t-caption">Окна, которые ядро создало в этом процессе.</p>
        </div>
        <Button
          variant="ghost"
          onClick={() => setAttempt((value) => value + 1)}
          disabled={state.status === 'loading'}
        >
          Обновить
        </Button>
      </header>

      {state.status === 'loading' ? (
        <div className="surfaces__loading" aria-label="Загружаю состояние поверхностей">
          <Skeleton height="var(--sp-7)" />
          <Skeleton height="var(--sp-7)" width="70%" />
        </div>
      ) : null}

      {state.status === 'unavailable' ? (
        <div className="surfaces__empty">
          <p className="t-body">Tauri недоступен: панель работает в окне приложения.</p>
          <p className="surfaces__message t-caption">{state.message}</p>
        </div>
      ) : null}

      {state.status === 'error' ? (
        <div className="surfaces__empty" role="alert">
          <p className="t-body">Не удалось получить состояние поверхностей.</p>
          <p className="surfaces__message t-caption">{state.message}</p>
        </div>
      ) : null}

      {state.status === 'ready' && state.surfaces.length === 0 ? (
        <p className="surfaces__empty t-body">Поверхностей пока нет.</p>
      ) : null}

      {state.status === 'ready' && state.surfaces.length > 0 ? (
        <ul className="surfaces__list">
          {state.surfaces.map((surface) => (
            <li key={`${surface.kind}-${surface.monitor}`} className="surfaces__row">
              <span className="surfaces__kind t-label">{KIND_LABELS[surface.kind]}</span>
              <span className="surfaces__monitor t-caption num">{surface.monitor}</span>
              <span className="surfaces__size t-caption num">
                {surface.width} × {surface.height}
              </span>
              <span
                className="surfaces__state t-micro"
                data-state={surface.state}
                title={surface.state === 'visible' ? 'Окно видно' : 'Окно скрыто'}
              >
                {stateLabel(surface)}
              </span>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
