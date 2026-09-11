interface ButtonProps extends React.ComponentProps<"button">, VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export default function Button({ className, ...props }) {
  const Comp = asChild ? Slot.root : "button";

  return <Comp {...props} />;
}
