import { useEffect, useState } from 'react';
import { currentTheme, getMode, setMode, subscribe, toggle } from './theme';
import type { ResolvedTheme, ThemeMode } from './theme';

export interface ThemeController {
  readonly mode: ThemeMode;
  readonly theme: ResolvedTheme;
  readonly setMode: (mode: ThemeMode) => void;
  readonly toggle: () => ResolvedTheme;
}

/**
 * React-обёртка над ThemeService.
 *
 * Состояние читается из DOM и localStorage, а не дублируется в контексте:
 * тему меняет не только этот компонент, но и inline-скрипт, `storage` и
 * будущее событие ядра.
 */
export function useTheme(): ThemeController {
  const [state, setState] = useState<{ mode: ThemeMode; theme: ResolvedTheme }>(() => ({
    mode: getMode(),
    theme: currentTheme(),
  }));

  useEffect(() => {
    // Синхронизация на случай, если тема изменилась между render и effect.
    setState({ mode: getMode(), theme: currentTheme() });
    return subscribe(() => {
      setState({ mode: getMode(), theme: currentTheme() });
    });
  }, []);

  return {
    mode: state.mode,
    theme: state.theme,
    setMode,
    toggle,
  };
}
