import { useState } from 'react';
import { useTheme } from '@/shared/theme/useTheme';
import {
  Button,
  Divider,
  Field,
  Glass,
  IconButton,
  Kbd,
  Skeleton,
  Slider,
  Switch,
} from '@/shared/ui';

const TYPE_SCALE = [
  { className: 't-hero', name: 'hero', meta: '56 / 500' },
  { className: 't-display', name: 'display', meta: '36 / 500' },
  { className: 't-title', name: 'title', meta: '24 / 500' },
  { className: 't-body', name: 'body', meta: '16 / 400' },
  { className: 't-label', name: 'label', meta: '15 / 500' },
  { className: 't-caption', name: 'caption', meta: '14 / 500' },
  { className: 't-micro', name: 'micro', meta: '12 / 500' },
] as const;

const SWATCHES = [
  'bg',
  'bg-elev',
  'panel',
  'panel-hi',
  'stroke',
  'text',
  'text-dim',
  'text-faint',
  'accent',
  'ok',
  'warn',
  'danger',
] as const;

const THEME_MODES = [
  { mode: 'dark', label: 'Тёмная' },
  { mode: 'light', label: 'Светлая' },
  { mode: 'system', label: 'Системная' },
] as const;

/**
 * Витрина ui-kit. Доступна только в dev по адресу `/_kit`; в production
 * маршрут не регистрируется, а модуль вырезается сборщиком.
 */
export function KitPage(): React.JSX.Element {
  const { mode, theme, setMode } = useTheme();
  const [notifications, setNotifications] = useState(true);
  const [volume, setVolume] = useState(60);

  return (
    <main className="kit">
      <header className="kit__header">
        <div className="kit__stack">
          <h1 className="t-display">Vela ui-kit</h1>
          <p className="t-body kit__lead">
            Витрина токенов, типографики и примитивов P1. Здесь проверяются обе темы, фокус и
            состояния элементов до того, как появится док в P2.
          </p>
        </div>
        <div className="kit__stack">
          <div className="kit__row" role="group" aria-label="Тема">
            {THEME_MODES.map((item) => (
              <Button
                key={item.mode}
                variant={mode === item.mode ? 'primary' : 'ghost'}
                aria-pressed={mode === item.mode}
                onClick={() => setMode(item.mode)}
              >
                {item.label}
              </Button>
            ))}
          </div>
          <span className="t-caption kit__muted num">
            режим: {mode} · тема: {theme}
          </span>
        </div>
      </header>

      <div className="kit__sections">
        <section className="kit__section kit__section--type">
          <h2 className="t-title">Типографика</h2>
          <div className="kit__stack">
            {TYPE_SCALE.map((item) => (
              <div key={item.name} className="kit__type-row">
                <span className={item.className}>Vela — виджетный слой</span>
                <span className="t-caption kit__muted num kit__type-meta">
                  {item.name} · {item.meta}
                </span>
              </div>
            ))}
          </div>
        </section>

        <section className="kit__section">
          <h2 className="t-title">Кнопки</h2>
          <div className="kit__row">
            <Button>Primary</Button>
            <Button variant="ghost">Ghost</Button>
            <IconButton label="Закрыть">×</IconButton>
            <Button disabled>Disabled</Button>
            <Button variant="ghost" disabled>
              Disabled
            </Button>
          </div>
          <Divider />
          <div className="kit__row">
            <span className="t-caption kit__muted">Сочетания клавиш</span>
            <Kbd>Ctrl</Kbd>
            <Kbd>K</Kbd>
          </div>
        </section>

        <section className="kit__section">
          <h2 className="t-title">Контролы</h2>
          <div className="kit__stack">
            <div className="kit__row">
              <span className="t-label">Уведомления</span>
              <Switch
                checked={notifications}
                onCheckedChange={setNotifications}
                label="Уведомления"
                id="kit-notifications"
              />
              <span className="t-caption kit__muted">{notifications ? 'вкл' : 'выкл'}</span>
            </div>
            <div className="kit__row">
              <span className="t-label">Выключенный</span>
              <Switch checked={false} onCheckedChange={() => {}} label="Выключенный" disabled />
            </div>
            <Field label="Имя профиля" htmlFor="kit-name" hint="Отображается в доке и настройках.">
              <input
                id="kit-name"
                className="ui-field__input"
                placeholder="Например, Vela"
                aria-describedby="kit-name-hint"
              />
            </Field>
            <Field label="Громкость" htmlFor="kit-volume" hint="Значение сохраняется в настройках.">
              <div className="kit__row">
                <Slider
                  value={volume}
                  onChange={setVolume}
                  label="Громкость"
                  min={0}
                  max={100}
                  step={5}
                />
                <span className="t-caption num">{volume}%</span>
              </div>
            </Field>
          </div>
        </section>

        <section className="kit__section">
          <h2 className="t-title">Стекло</h2>
          <div className="kit__stack">
            <Glass elevation={0} padding="md">
              <span className="t-label">Elevation 0</span>
              <p className="t-caption kit__muted">Только внутренняя грань, без внешней тени.</p>
            </Glass>
            <Glass elevation={1} padding="md">
              <span className="t-label">Elevation 1</span>
              <p className="t-caption kit__muted">Обычная панель.</p>
            </Glass>
            <Glass elevation={2} padding="md">
              <span className="t-label">Elevation 2</span>
              <p className="t-caption kit__muted">Активная или перетаскиваемая панель.</p>
            </Glass>
            <Glass elevation={1} padding="md" interactive>
              <span className="t-label">Interactive</span>
              <p className="t-caption kit__muted">Наведите курсор: граница и тень реагируют.</p>
            </Glass>
          </div>
        </section>

        <section className="kit__section">
          <h2 className="t-title">Цвета</h2>
          <div className="kit__swatches">
            {SWATCHES.map((name) => (
              <div key={name} className="kit__swatch">
                <span className={`kit__swatch-chip kit__swatch-chip--${name}`} />
                <span className="t-caption kit__muted">{name}</span>
              </div>
            ))}
          </div>
        </section>

        <section className="kit__section">
          <h2 className="t-title">Заглушки</h2>
          <div className="kit__stack">
            <Skeleton height="var(--sp-4)" />
            <Skeleton width="72%" />
            <Skeleton width="48%" />
          </div>
        </section>
      </div>
    </main>
  );
}
