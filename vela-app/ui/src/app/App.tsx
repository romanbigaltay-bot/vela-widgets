/**
 * Корневой компонент. В P1 — проверка того, что каркас собран правильно:
 * алиас `@`, токены темы, шрифт. Реальный холст появляется в P3.
 */
import { health } from '@/shared/lib/health';

export function App(): React.JSX.Element {
  const parts = health();

  return (
    <main className="skeleton">
      <h1 className="skeleton__title t-display">Vela</h1>
      <p className="skeleton__subtitle t-body">Каркас собран. Холст появится в P3.</p>
      <dl className="skeleton__grid">
        {parts.map((part) => (
          <div key={part.name} className="skeleton__row">
            <dt className="t-caption">{part.name}</dt>
            <dd className="t-caption num">{part.value}</dd>
          </div>
        ))}
      </dl>
      {import.meta.env.DEV ? (
        <a className="t-caption" href="/_kit">
          Открыть ui-kit
        </a>
      ) : null}
    </main>
  );
}
