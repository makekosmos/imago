export default function clsx() {
  let i = 0;
  let len = arguments.length;
  let str = "";
  let temp: unknown;

  while (i < len) {
    if ((temp = arguments[i])) {
      if (typeof temp === "string") {
        str += (str && " ") + temp;
      }
    }

    i++;
  }

  return str;
}
