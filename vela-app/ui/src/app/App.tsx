/**
 * Корневой компонент. В P1 — проверка того, что каркас собран правильно:
 * алиас `@`, токены темы, шрифт. Реальный холст появляется в P3.
 */
import { health } from '@/shared/lib/health';

export function App(): React.JSX.Element {
  const parts = health();

  return (
    <main className="skeleton">
      <h1 className="skeleton__title">Vela</h1>
      <p className="skeleton__subtitle">Каркас собран. Холст появится в P3.</p>
      <dl className="skeleton__grid">
        {parts.map((part) => (
          <div key={part.name} className="skeleton__row">
            <dt>{part.name}</dt>
            <dd>{part.value}</dd>
          </div>
        ))}
      </dl>
    </main>
  );
}
