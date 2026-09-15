import cva from "@core/cva";

const buttonVariantMap = {
  primary: "button--primary",
  ghost: "button--ghost",
  surface: "button--surface",
  success: "button--success",
  danger: "button--danger",
} as const;

const buttonSizeMap = {
  md: "button--md",
  sm: "button--sm",
} as const;

export const buttonVariants = cva("button", {
  variants: {
    variant: buttonVariantMap,
    size: buttonSizeMap,
  },

  defaultVariants: {
    variant: "primary",
    size: "md",
  },
});

export type buttonVariantsProps = {
  variant?: keyof typeof buttonVariantMap;
  size?: keyof typeof buttonSizeMap;
};
