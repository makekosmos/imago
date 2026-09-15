import { buttonVariantsProps } from "@core/components/button/button";

export type ButtonSharedProps = buttonVariantsProps & {
  loading?: boolean;
  disabled?: boolean;
  type?: "button" | "submit" | "reset";
};
