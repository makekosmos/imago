export default function clsx(...args: unknown[]) {
  let i = 0;
  let len = args.length;
  let str = "";
  let temp: unknown;

  while (i < len) {
    if ((temp = args[i])) {
      if (typeof temp === "string") {
        str += (str && " ") + temp;
      }
    }

    i++;
  }

  return str;
}
