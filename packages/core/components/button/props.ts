import { buttonVariantsProps } from "./button";

export type ButtonSharedProps = buttonVariantsProps & {
  loading?: boolean;
  disabled?: boolean;
  type?: "button" | "submit" | "reset";
};
