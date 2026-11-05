import type { ReactNode } from 'react';

export interface KbdProps {
  children: ReactNode;
}

export function Kbd({ children }: KbdProps): React.JSX.Element {
  return <kbd className="ui-kbd">{children}</kbd>;
}
