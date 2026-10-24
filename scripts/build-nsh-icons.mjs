import { readFile, writeFile } from "node:fs/promises";

const source = new URL("../crates/imago-nsh/ui/assets/icons/", import.meta.url);
const output = new URL("../crates/imago-nsh/ui/icons.ts", import.meta.url);
const files = {
  about: "calendar-01.svg",
  check: "check.svg",
  inbox: "inbox.svg",
  logbook: "book-open.svg",
  more: "more-circle.svg",
  nextWeek: "calendar-02.svg",
  plus: "plus.svg",
  project: "folder.svg",
  settings: "sliders.svg",
  sidebar: "sidebar-left.svg",
  someday: "clock-01.svg",
  statistics: "analytics-01.svg",
  trash: "delete.svg",
  week: "calendar-01.svg",
  display: "preference-vertical.svg",
};
const entries = await Promise.all(
  Object.entries(files).map(async ([name, file]) => {
    const svg = await readFile(new URL(file, source), "utf8");
    return `  ${name}: "data:image/svg+xml,${encodeURIComponent(svg)}",`;
  }),
);

await writeFile(output, `export const icons = {\n${entries.join("\n")}\n} as const;\n`);
