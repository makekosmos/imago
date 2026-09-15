import clsx from "@core/clsx";
import { buttonVariants, type ButtonSharedProps } from "@core/components/button";
import type { ComponentPropsWithoutRef } from "react";

type ButtonProps = ButtonSharedProps &
  Omit<ComponentPropsWithoutRef<"button">, keyof ButtonSharedProps>;

export default function Button({
  variant,
  size,
  loading = false,
  disabled = false,
  type = "button",
  className,
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled || loading}
      data-loading={loading}
      className={clsx(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}
