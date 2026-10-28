import { expect, open, test } from "nsh/test";

test("exports shared titlebar and SVG icons", async () => {
  const app = await open("./App.tsx");
  expect(app.texts()).toContain("Imago");
  expect(app.texts()).toContain("Shared NSH components");
  await app.press("Календарь дневника");
  expect(app.texts()).toContain("1");
});
