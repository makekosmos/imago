import { open, test } from "nsh/test";
import { Window } from "nsh/desktop";

test("minimal app screenshot", async () => {
  const app = await open("../App.tsx");
  await app.setViewport(900, 600);
  Window.setFrame({ origin: { x: -20000, y: -20000 }, size: { width: 900, height: 600 } });
  await app.screenshot("example.png");
});
