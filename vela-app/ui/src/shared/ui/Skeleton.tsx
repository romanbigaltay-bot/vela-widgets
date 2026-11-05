export interface SkeletonProps {
  width?: string;
  height?: string;
  className?: string;
}

export function Skeleton({
  width = '100%',
  height = 'var(--sp-3)',
  className,
}: SkeletonProps): React.JSX.Element {
  const classes = ['ui-skeleton'];
  if (className) classes.push(className);

  return <div className={classes.join(' ')} style={{ width, height }} aria-hidden="true" />;
}
