import cva from "@core/cva";

export const buttonVariants = cva("button", {
  variants: {
    variant: {
      primary: "button--primary",
      ghost: "button--ghost",
      surface: "button--surface",
      success: "button--success",
      danger: "button--danger",
    },
    size: {
      md: "button--md",
      sm: "button--sm",
    },
  },

  defaultVariants: {
    variant: "primary",
    size: "md",
  },
});

export type ButtonProps = Parameters<typeof buttonVariants>[0] & {
  loading?: boolean;
  disabled?: boolean;
  type?: "button" | "submit";
};
