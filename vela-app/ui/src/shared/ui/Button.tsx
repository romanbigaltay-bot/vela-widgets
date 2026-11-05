import type { ButtonHTMLAttributes, ReactNode } from 'react';

export type ButtonVariant = 'primary' | 'ghost' | 'icon';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  children?: ReactNode;
}

export function Button({
  variant = 'primary',
  type = 'button',
  className,
  children,
  ...rest
}: ButtonProps): React.JSX.Element {
  const classes = ['ui-button', `ui-button--${variant}`];
  if (className) classes.push(className);

  return (
    <button type={type} className={classes.join(' ')} {...rest}>
      {children}
    </button>
  );
}

export interface IconButtonProps extends Omit<ButtonProps, 'variant' | 'children'> {
  label: string;
  children: ReactNode;
}

export function IconButton({
  label,
  className,
  children,
  ...rest
}: IconButtonProps): React.JSX.Element {
  return (
    <Button {...rest} variant="icon" aria-label={label} className={className}>
      {children}
    </Button>
  );
}
