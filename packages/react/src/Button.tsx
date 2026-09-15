import { buttonVariants, type ButtonProps } from "@core/button";

export default function Button({
  variant,
  size,
  loading = false,
  disabled = false,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled || loading}
      data-loading={loading}
      className={buttonVariants({ variant, size })}
      {...props}
    />
  );
}
