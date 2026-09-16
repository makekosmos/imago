import type { CheckboxProps } from "@core/components/checkbox";
export default function Checkbox({
  checked,
  onCheckedChange,
  disabled = false,
  ariaLabel,
}: CheckboxProps) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-label={ariaLabel}
      aria-checked={checked}
      disabled={disabled}
      className={checked ? "checkbox checkbox--checked" : "checkbox"}
      onClick={() => onCheckedChange?.(!checked)}
    >
      {checked && <span aria-hidden="true" className="checkbox__indicator" />}
    </button>
  );
}
