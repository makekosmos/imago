import "nsh/types";

declare module "nsh/types" {
  namespace JSX {
    interface IntrinsicAttributes {
      key?: string | number;
    }
  }
}
