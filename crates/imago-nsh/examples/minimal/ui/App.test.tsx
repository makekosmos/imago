import { expect, open, test } from "nsh/test";

test("minimal app imports the package and shows its page", async () => {
  const app = await open("./App.tsx");
  expect(app.texts()).toContain("Одна страница приложения");
  expect(app.texts()).toContain("Главная");
});
