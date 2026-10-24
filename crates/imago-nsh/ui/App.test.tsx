import { expect, open, test } from "nsh/test";

test("exports shared titlebar and SVG icons", async () => {
  const app = await open("./App.tsx");
  expect(app.texts()).toContain("Imago");
  expect(app.texts()).toContain("Shared NSH components");
});
