import { textInputVariants } from "./textInput";

export type TextInputProps = Parameters<typeof textInputVariants>[0] & {
  value?: string;
  onValueChange?: (value: string) => void;

  placeholder?: string;
  disabled?: boolean;
  readonly?: boolean;
  autocomplete?: string;
  invalid?: boolean;
};
