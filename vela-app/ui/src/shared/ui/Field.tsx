import type { ReactNode } from 'react';

export interface FieldProps {
  label: string;
  htmlFor: string;
  hint?: string;
  children: ReactNode;
}

export function Field({ label, htmlFor, hint, children }: FieldProps): React.JSX.Element {
  const hintId = `${htmlFor}-hint`;

  return (
    <div className="ui-field">
      <label className="ui-field__label t-label" htmlFor={htmlFor}>
        {label}
      </label>
      {children}
      {hint ? (
        <span className="ui-field__hint t-caption" id={hintId}>
          {hint}
        </span>
      ) : null}
    </div>
  );
}
