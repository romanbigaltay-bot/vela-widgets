export interface SwitchProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  /** Доступное имя: видимая подпись может стоять рядом и быть связана иначе. */
  label: string;
  disabled?: boolean;
  id?: string;
}

/**
 * Переключатель как кнопка с `role="switch"`: клавиатурные Space/Enter
 * обрабатываются нативно, отдельный keydown не нужен.
 */
export function Switch({
  checked,
  onCheckedChange,
  label,
  disabled = false,
  id,
}: SwitchProps): React.JSX.Element {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      id={id}
      disabled={disabled}
      className="ui-switch"
      onClick={() => onCheckedChange(!checked)}
    >
      <span className="ui-switch__thumb" aria-hidden="true" />
    </button>
  );
}
