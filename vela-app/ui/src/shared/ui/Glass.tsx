import { createContext, useContext, useEffect, useRef } from 'react';
import type { HTMLAttributes, ReactNode } from 'react';
import { log } from '@/shared/lib/log';

export type GlassRadius = 'chip' | 'inner' | 'panel' | 'pill';
export type GlassElevation = 0 | 1 | 2;
export type GlassPadding = 'none' | 'sm' | 'md' | 'lg';

export interface GlassProps extends HTMLAttributes<HTMLDivElement> {
  radius?: GlassRadius;
  elevation?: GlassElevation;
  interactive?: boolean;
  padding?: GlassPadding;
  children?: ReactNode;
}

/**
 * Глубина вложенности. Нужна только для dev-предупреждения: на глаз
 * вложенное стекло выглядит «просто мутнее», а на деле это два blur-прохода.
 */
const GlassDepthContext = createContext(0);

export function Glass({
  radius = 'panel',
  elevation = 1,
  interactive = false,
  padding = 'none',
  className,
  children,
  ...rest
}: GlassProps): React.JSX.Element {
  const depth = useContext(GlassDepthContext);
  const warned = useRef(false);

  useEffect(() => {
    if (depth > 0 && !warned.current && import.meta.env.DEV) {
      warned.current = true;
      log('warn', 'Glass вложен в Glass: вложенное стекло запрещено (ST-0103-03)');
    }
  }, [depth]);

  const classes = [
    'glass',
    `glass--radius-${radius}`,
    `glass--elev-${elevation}`,
    `glass--pad-${padding}`,
  ];
  if (interactive) classes.push('glass--interactive');
  if (className) classes.push(className);

  return (
    <GlassDepthContext.Provider value={depth + 1}>
      <div className={classes.join(' ')} {...rest}>
        {children}
      </div>
    </GlassDepthContext.Provider>
  );
}
