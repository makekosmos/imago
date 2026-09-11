interface ButtonProps extends React.ComponentProps<"button">, VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  variant: string;
}

export default function Button({
  className,
  variant = "default",
  asChild = false,
  ...props
}: ButtonProps) {
  const Comp = asChild ? Slot.root : "button";

  return <Comp className={className} variant={variant} {...props} />;
}
