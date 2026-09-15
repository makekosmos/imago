import { buttonVariants, type ButtonSharedProps } from "@core/components/button";

export default function Button({
  variant,
  size,
  loading = false,
  disabled = false,
  type = "button",
  ...props
}: ButtonSharedProps) {
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
