import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";
import * as sfcNs from "@vue/compiler-sfc";
import { effectScope, nextTick } from "vue";

// Поведенческие тесты <script setup> компонентов без браузера: SFC
// компилируется через @vue/compiler-sfc (тот же компилятор, что у vite),
// типы снимаются typescript'ом, setup() гоняется в effectScope — это чистая
// логика состояния, DOM нужен только для mount, которого здесь нет.
const { parse, compileScript } = sfcNs.default ?? sfcNs;

// Overlay-компоненты вешают document/window listener'ы при open — минимальные
// шимы; реального DOM не требуется (панели teleport'ятся и не маунтятся).
globalThis.document = {
  addEventListener() {},
  removeEventListener() {},
  documentElement: { dataset: {} },
};
globalThis.window = {
  addEventListener() {},
  removeEventListener() {},
  innerWidth: 1024,
  innerHeight: 768,
};

// setup() вне экземпляра компонента даёт ожидаемое предупреждение про
// lifecycle hooks — глушим только его, остальные warn'ы остаются видимыми.
const realWarn = console.warn;
console.warn = (...args) => {
  if (String(args[0]).includes("no active component instance")) return;
  realWarn(...args);
};

const repoRoot = join(import.meta.dirname, "..");
const dir = mkdtempSync(join(import.meta.dirname, ".tmp-vue-behavior-"));

function writeModule(outName, code) {
  writeFileSync(join(dir, outName), code);
  return pathToFileURL(join(dir, outName)).href;
}

function transpileTsModule(relPath) {
  const outName = relPath.split("/").pop().replace(/\.ts$/, ".mjs");
  const code = ts.transpileModule(readFileSync(join(repoRoot, relPath), "utf8"), {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  return writeModule(outName, code);
}

function compileVueModule(relPath) {
  const name = relPath.split("/").pop().replace(/\.vue$/, "");
  const source = readFileSync(join(repoRoot, relPath), "utf8");
  const { descriptor, errors } = parse(source, { filename: relPath });
  if (errors.length) {
    throw new Error(`${relPath}: ${errors.map((e) => e.message).join("; ")}`);
  }
  const compiled = compileScript(descriptor, { id: name });
  let code = ts.transpileModule(compiled.content, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  code = code
    .replaceAll(/from ["']\.\/([A-Za-z]+)\.vue["']/g, 'from "./$1.compiled.mjs"')
    .replaceAll(/from ["']\.\.\/composables\/([A-Za-z]+)["']/g, 'from "./$1.mjs"')
    .replaceAll(/from ["']\.\/([A-Za-z]+)["']/g, 'from "./$1.mjs"');
  return writeModule(`${name}.compiled.mjs`, code);
}

function runSetup(component, props) {
  const scope = effectScope();
  const emitted = [];
  const bindings = scope.run(() =>
    component.setup(props, { expose() {}, emit: (...args) => emitted.push(args) }),
  );
  assert.ok(bindings, "setup() must return bindings");
  return { scope, bindings, emitted };
}

try {
  // Порядок важен: .vue-зависимости компилируются до компонентов, которые их
  // импортируют (Dropdown ← Calendar ← DateTimePicker).
  transpileTsModule("packages/vue/src/components/dates.ts");
  transpileTsModule("packages/vue/src/composables/useEscapeLayer.ts");
  compileVueModule("packages/vue/src/components/Dropdown.vue");
  compileVueModule("packages/vue/src/components/Calendar.vue");
  compileVueModule("packages/vue/src/components/TimeColumn.vue");
  compileVueModule("packages/vue/src/components/DateTimePicker.vue");

  const Calendar = (await import(pathToFileURL(join(dir, "Calendar.compiled.mjs")).href)).default;
  const DateTimePicker = (await import(pathToFileURL(join(dir, "DateTimePicker.compiled.mjs")).href)).default;
  const Dropdown = (await import(pathToFileURL(join(dir, "Dropdown.compiled.mjs")).href)).default;
  const TimeColumn = (await import(pathToFileURL(join(dir, "TimeColumn.compiled.mjs")).href)).default;

  // KOS-204 Calendar: value="2024-02-29" (високосный день) раньше открывал
  // календарь на МАРТЕ и подсвечивал 1 марта — parseIso/localDate собирали
  // дату через год 1900 (невисокосный) до установки целевого года.
  {
    const { scope, bindings } = runSetup(Calendar, {
      value: "2024-02-29",
      today: "2024-02-29",
    });
    assert.equal(bindings.viewMonth.value.getFullYear(), 2024);
    assert.equal(bindings.viewMonth.value.getMonth(), 1, "calendar opens on February");
    const feb29 = bindings.cells.value.find((c) => c.iso === "2024-02-29");
    assert.ok(feb29, "Feb 29 cell exists in the February grid");
    assert.equal(feb29.isSelected, true, "Feb 29 is the selected day");
    assert.equal(feb29.isToday, true);
    scope.stop();
  }

  // KOS-204 DateTimePicker: невалидный value не должен ронять рендер
  // (было: isoToLocal() → null → `s.hour` TypeError). Падаем на placeholder.
  {
    const { scope, bindings } = runSetup(DateTimePicker, {
      value: "not-a-date",
      placeholder: "Выбрать…",
    });
    assert.equal(bindings.displayLabel.value, "Выбрать…");
    scope.stop();
  }

  // KOS-204 DateTimePicker dateOnly: выбор 29.02.2024 emit'ит ISO, который
  // читается назад как 29 февраля, а не 1 марта (localDateTime bug).
  {
    const { scope, bindings, emitted } = runSetup(DateTimePicker, {
      value: null,
      placeholder: "x",
      dateOnly: true,
    });
    bindings.onDatePick("2024-02-29");
    await nextTick();
    const iso = emitted.find(([name]) => name === "update:value")?.[1];
    assert.ok(iso, "dateOnly pick emits update:value");
    const back = new Date(iso);
    assert.equal(back.getFullYear(), 2024);
    assert.equal(back.getMonth(), 1, "picked Feb 29 round-trips to February");
    assert.equal(back.getDate(), 29, "picked Feb 29 round-trips to day 29");
    scope.stop();
  }

  // KOS-204 TimeColumn: step<=0 / нефинитные границы раньше давали
  // бесконечный цикл (фатальный V8 invalid-size crash). Деградируем в
  // колонку из текущего значения; штатный диапазон не меняется.
  {
    const { scope, bindings } = runSetup(TimeColumn, {
      value: 7,
      min: 0,
      max: 59,
      step: 0,
    });
    assert.deepEqual(bindings.items.value, [7]);
    scope.stop();
  }
  {
    const { scope, bindings } = runSetup(TimeColumn, {
      value: 3,
      min: 0,
      max: Number.POSITIVE_INFINITY,
      step: 1,
    });
    assert.deepEqual(bindings.items.value, [3]);
    scope.stop();
  }
  {
    const { scope, bindings } = runSetup(TimeColumn, {
      value: 0,
      min: 0,
      max: 10,
      step: 5,
    });
    assert.deepEqual(bindings.items.value, [0, 5, 10]);
    scope.stop();
  }

  // KOS-204 Dropdown: повторное открытие после поиска. Раньше watcher
  // searchQuery (queued) перезаписывал highlight, выставленный toggle() на
  // выбранной опции → highlight уезжал на первый пункт и Enter выбирал его.
  {
    const options = [
      { value: "a", label: "Apple" },
      { value: "b", label: "Banana" },
      { value: "c", label: "Cherry" },
    ];
    const { scope, bindings } = runSetup(Dropdown, {
      modelValue: "c",
      options,
      searchable: true,
    });

    bindings.toggle();
    await nextTick();
    assert.equal(bindings.highlightIdx.value, 2, "first open highlights the selected option");

    bindings.searchQuery.value = "an";
    await nextTick();
    assert.deepEqual(
      bindings.filteredOptions.value.map((o) => o.value),
      ["b"],
      "search still filters the list",
    );
    assert.equal(bindings.highlightIdx.value, 0, "non-empty query highlights the first match");

    bindings.open.value = false;
    await nextTick();
    bindings.toggle();
    await nextTick();
    const highlighted = bindings.filteredOptions.value[bindings.highlightIdx.value];
    assert.equal(highlighted?.value, "c", "reopen keeps highlight on the selected option");
    scope.stop();
  }

  console.log("vue-behavior tests passed");
} finally {
  console.warn = realWarn;
  rmSync(dir, { recursive: true, force: true });
}
