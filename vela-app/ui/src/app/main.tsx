import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import '@/styles/base.css';

const container = document.getElementById('root');

if (!container) {
  // Не `!`-assertion: если корня нет, приложение всё равно должно сообщить
  // причину, а не упасть с невнятным TypeError.
  throw new Error('root container #root is missing in index.html');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
