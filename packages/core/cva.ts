import clsx from "@core/clsx";

type VariantsSchema = Record<string, Record<string, string>>;
type VariantProps<V extends VariantsSchema> = {
  [K in keyof V]?: keyof V[K];
};
type CvaConfig<V extends VariantsSchema> = {
  variants: V;
  defaultVariants?: VariantProps<V>;
};

export default function cva<V extends VariantsSchema>(base: string, config: CvaConfig<V>) {
  return function (props: VariantProps<V>) {
    const resultClasses = [];
    const variants = config.variants;

    for (const key in variants) {
      const variant = variants[key];

      const selectedValue = props[key] ?? config.defaultVariants?.[key];

      if (selectedValue) {
        resultClasses.push(variant[selectedValue]);
      }
    }
    return clsx(base, ...resultClasses);
  };
}
