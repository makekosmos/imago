import { TextInputProps, textInputVariants } from "@core/components/textInput";

export default function TextInput({
  value = "",
  onValueChange,
  type = "text",
  size = "sm",
  disabled = false,
  readonly = false,
  ...props
}: TextInputProps) {
  return (
    <input
      type={type}
      value={value}
      disabled={disabled}
      readOnly={readonly}
      className={textInputVariants({ size })}
      onChange={(e) => onValueChange?.(e.target.value)}
      {...props}
    />
  );
}
