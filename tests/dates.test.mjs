import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

// dates.ts — чистый TS-модуль без зависимостей; транспилируем тем же
// typescript, что гоняет typecheck, и импортируем как ESM (как
// escape-layer.test.mjs).
const dir = mkdtempSync(join(import.meta.dirname, ".tmp-dates-"));

try {
  const transpiled = ts.transpileModule(
    readFileSync(join(import.meta.dirname, "../packages/vue/src/components/dates.ts"), "utf8"),
    { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } },
  ).outputText;
  writeFileSync(join(dir, "dates.mjs"), transpiled);

  const { formatIsoYear, localDate, localDateTime, pad } = await import(
    pathToFileURL(join(dir, "dates.mjs")).href
  );

  // Regression KOS-204: 29 февраля високосного года. Дата раньше собиралась
  // как `new Date(0, m, d)` (год 1900 — НЕвисокосный) + setFullYear(y) —
  // переполнение вычислялось до установки года и 29.02 съезжало на 01.03.
  {
    const d = localDate(2024, 1, 29);
    assert.equal(d.getFullYear(), 2024);
    assert.equal(d.getMonth(), 1, "leap-day stays in February");
    assert.equal(d.getDate(), 29, "leap day not shifted to March 1");
  }

  // Невисокосный год: переполнение сохраняет прежнюю семантику (→ 1 марта).
  {
    const d = localDate(2023, 1, 29);
    assert.equal(d.getFullYear(), 2023);
    assert.equal(d.getMonth(), 2);
    assert.equal(d.getDate(), 1);
  }

  // Года < 100 не съезжают на 1900+y (setFullYear, не new Date(y, …)).
  {
    const d = localDate(50, 0, 15);
    assert.equal(d.getFullYear(), 50);
    assert.equal(d.getMonth(), 0);
    assert.equal(d.getDate(), 15);
  }

  // localDateTime: то же правило года плюс локальное время.
  {
    const d = localDateTime({ year: 2024, month: 1, day: 29, hour: 13, minute: 30 });
    assert.equal(d.getFullYear(), 2024);
    assert.equal(d.getMonth(), 1);
    assert.equal(d.getDate(), 29);
    assert.equal(d.getHours(), 13);
    assert.equal(d.getMinutes(), 30);
    assert.equal(d.getSeconds(), 0);
  }

  {
    assert.equal(formatIsoYear(2024), "2024");
    assert.equal(formatIsoYear(50), "0050");
    assert.equal(formatIsoYear(-1), "-000001");
    assert.equal(pad(7), "07");
    assert.equal(pad(12), "12");
  }

  console.log("dates tests passed");
} finally {
  rmSync(dir, { recursive: true, force: true });
}
