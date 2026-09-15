import cva from "@core/cva";

export const textInputVariants = cva("text-input", {
  variants: {
    type: {
      text: "text-input--text",
      password: "text-input--password",
      email: "text-input--email",
      url: "text-input--url",
      search: "text-input--search",
    },
    inputmode: {
      text: "text-input--text",
      email: "text-input--email",
      url: "text-input--url",
      search: "text-input--search",
      numeric: "text-input--numeric",
      decimal: "text-input--decimal",
      tel: "text-input--tel",
    },
    size: {
      md: "text-input--md",
      sm: "text-input--sm",
    },
  },
});

export type textInputVariantsProps = Parameters<typeof textInputVariants>[0];
