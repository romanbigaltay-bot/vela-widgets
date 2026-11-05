import { StrictMode, Suspense, lazy } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { initTheme } from '@/shared/theme/theme';
import '@/styles/tokens.css';
import '@/styles/base.css';
import '@/styles/typography.css';
import '@/styles/glass.css';
import '@/styles/ui-kit.css';

const container = document.getElementById('root');

if (!container) {
  // Не `!`-assertion: если корня нет, приложение всё равно должно сообщить
  // причину, а не упасть с невнятным TypeError.
  throw new Error('root container #root is missing in index.html');
}

// Inline-скрипт уже выставил тему до первого кадра; здесь только подписки
// на системные изменения и storage.
initTheme();

// Витрина существует только в dev: в production `import.meta.env.DEV` — это
// литерал `false`, поэтому ветка с dynamic import вырезается вместе с KitPage.
const KitPage = import.meta.env.DEV
  ? lazy(async () => {
      const module = await import('./KitPage');
      return { default: module.KitPage };
    })
  : null;

const showKit = import.meta.env.DEV && window.location.pathname === '/_kit';

createRoot(container).render(
  <StrictMode>
    {showKit && KitPage ? (
      <Suspense fallback={null}>
        <KitPage />
      </Suspense>
    ) : (
      <App />
    )}
  </StrictMode>,
);
