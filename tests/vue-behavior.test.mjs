import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";
// vue/compiler-sfc — официальный export-subpath прямой зависимости `vue`.
// Голый specifier "@vue/compiler-sfc" — транзитивная зависимость (peer
// vue-router / @vitejs/plugin-vue): под pnpm (задекларированный
// packageManager) он не хоистится в корневые node_modules и тест падал с
// ERR_MODULE_NOT_FOUND; под bun/npm он резолвился бы в ЧУЖУЮ версию
// (3.5.x), не совпадающую с vue 3.6, который эти же тесты компилируют.
import * as sfcNs from "vue/compiler-sfc";
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
// DOM-классы для instanceof-проверок в обработчиках (Dropdown onKey и т.п.).
globalThis.Element = class Element {};
globalThis.Node = class Node {};

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
  // импортируют (Dropdown ← Calendar ← {DateTimePicker, DateChip}).
  transpileTsModule("packages/vue/src/components/dates.ts");
  transpileTsModule("packages/vue/src/composables/useEscapeLayer.ts");
  transpileTsModule("packages/vue/src/composables/usePlatform.ts");
  compileVueModule("packages/vue/src/components/Dropdown.vue");
  compileVueModule("packages/vue/src/components/Calendar.vue");
  compileVueModule("packages/vue/src/components/TimeColumn.vue");
  compileVueModule("packages/vue/src/components/DateTimePicker.vue");
  compileVueModule("packages/vue/src/components/DateChip.vue");
  compileVueModule("packages/vue/src/components/HotkeyCapture.vue");
  compileVueModule("packages/vue/src/components/ContextMenu.vue");

  const Calendar = (await import(pathToFileURL(join(dir, "Calendar.compiled.mjs")).href)).default;
  const DateTimePicker = (await import(pathToFileURL(join(dir, "DateTimePicker.compiled.mjs")).href)).default;
  const Dropdown = (await import(pathToFileURL(join(dir, "Dropdown.compiled.mjs")).href)).default;
  const TimeColumn = (await import(pathToFileURL(join(dir, "TimeColumn.compiled.mjs")).href)).default;
  const DateChip = (await import(pathToFileURL(join(dir, "DateChip.compiled.mjs")).href)).default;
  const HotkeyCapture = (await import(pathToFileURL(join(dir, "HotkeyCapture.compiled.mjs")).href)).default;
  const ContextMenu = (await import(pathToFileURL(join(dir, "ContextMenu.compiled.mjs")).href)).default;

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

  // KOS-221 Calendar: год за пределами диапазона Date (6-значный "999999" >
  // ~275760) раньше принимался — localDate давал Invalid Date, и вся сетка
  // месяца превращалась в 42 ячейки "NaN" (viewMonth/viewYear → NaN). Теперь
  // такой год отвергается и в create-option, и в setYear, и в value-prop.
  {
    const { scope, bindings } = runSetup(Calendar, {
      value: "2024-06-15",
      today: "2024-06-15",
    });
    assert.equal(bindings.createYearOption("999999"), null, "out-of-range year option rejected");
    assert.equal(bindings.createYearOption("2025")?.value, 2025, "normal year option allowed");
    const before = bindings.viewMonth.value.getFullYear();
    bindings.setYear("999999");
    await nextTick();
    assert.equal(
      bindings.viewMonth.value.getFullYear(),
      before,
      "setYear ignores years outside the Date range",
    );
    assert.ok(
      bindings.cells.value.every((c) => Number.isFinite(c.day) && !c.iso.includes("NaN")),
      "grid never contains NaN cells",
    );
    scope.stop();
  }
  {
    // Тот же NaN-каскад через malformed value-prop: "999999-05-10" проходил
    // regex, но давал Invalid Date → сетка NaN. Теперь parseIso → null и
    // календарь открывается на `today`.
    const { scope, bindings } = runSetup(Calendar, {
      value: "999999-05-10",
      today: "2024-06-15",
    });
    assert.equal(bindings.viewMonth.value.getFullYear(), 2024);
    assert.equal(bindings.viewMonth.value.getMonth(), 5, "invalid value falls back to today");
    assert.ok(bindings.cells.value.every((c) => Number.isFinite(c.day)));
    scope.stop();
  }

  // KOS-248 Calendar: ‹ › стрелки у границы диапазона Date. setYear и
  // create-option отвергают out-of-range годы (KOS-221), но shiftMonth не
  // проверял результат — переход за последний валидный месяц клал в
  // viewMonth Invalid Date: 42 "NaN"-ячейки, и календарь зависал навсегда
  // (setYear отклонял любой год — isValidViewDate(year, NaN) === false).
  {
    const { scope, bindings } = runSetup(Calendar, {
      value: "2024-01-15",
      today: "2024-06-15",
    });
    // UI-путь: выбрать последний год, пока видимый месяц валиден для него,
    // затем идти ">" через границу (последний валидный месяц — сентябрь).
    bindings.setYear("275760");
    await nextTick();
    assert.equal(bindings.viewMonth.value.getFullYear(), 275760);
    for (let i = 0; i < 12; i++) bindings.shiftMonth(1);
    await nextTick();
    assert.ok(
      !Number.isNaN(bindings.viewMonth.value.getTime()),
      "month nav at the Date-range edge never produces Invalid Date",
    );
    // Стоим на последнем месяце диапазона: часть дней недели не существует
    // (после 13 сентября 275760) — они пустые, а не "NaN".
    assert.ok(
      bindings.cells.value.every((c) => !c.iso.includes("NaN") && (c.day === null || Number.isFinite(c.day))),
      "boundary month renders blank cells instead of NaN",
    );
    assert.ok(
      bindings.cells.value.some((c) => c.iso.startsWith("275760-09")),
      "valid days of the boundary month are still rendered",
    );
    const pos = bindings.viewMonth.value.getFullYear() * 12 + bindings.viewMonth.value.getMonth();
    bindings.shiftMonth(1);
    const posAfter = bindings.viewMonth.value.getFullYear() * 12 + bindings.viewMonth.value.getMonth();
    assert.equal(posAfter, pos, "shiftMonth past the last valid month is a no-op");
    // И календарь остаётся живым: год можно переключить обратно.
    bindings.setYear("2025");
    await nextTick();
    assert.equal(bindings.viewMonth.value.getFullYear(), 2025, "year switch still works");
    scope.stop();
  }

  // KOS-248 Calendar: переполненные поля в value ("2024-13-10", "2024-02-31")
  // localDate молча сдвигал на другую реальную дату — календарь открывал
  // другой месяц и подсвечивал не то число, что записано в prop'е. Теперь
  // malformed-поля эквивалентны отсутствию value (как в DateChip).
  {
    const { scope, bindings } = runSetup(Calendar, {
      value: "2024-13-10",
      today: "2024-06-15",
    });
    assert.equal(bindings.viewMonth.value.getFullYear(), 2024);
    assert.equal(bindings.viewMonth.value.getMonth(), 5, "malformed month falls back to today");
    assert.ok(
      bindings.cells.value.every((c) => !c.isSelected),
      "malformed value selects no phantom day",
    );
    scope.stop();
  }
  {
    const { scope, bindings } = runSetup(Calendar, {
      value: "2024-02-31",
      today: "2024-06-15",
    });
    assert.ok(
      bindings.cells.value.every((c) => !c.isSelected),
      "Feb 31 is not silently re-selected as Mar 2",
    );
    scope.stop();
  }

  // KOS-248 Dropdown: Tab внутрь списка ставит фокус на кнопку опции; Enter
  // на ней — её нативный click. Перехват выбирал highlighted (другой) пункт
  // и глушил правильный click через preventDefault.
  {
    const options = [
      { value: "a", label: "Apple" },
      { value: "b", label: "Banana" },
      { value: "c", label: "Cherry" },
    ];
    const { scope, bindings, emitted } = runSetup(Dropdown, {
      modelValue: "a",
      options,
      searchable: false,
    });
    bindings.toggle();
    await nextTick();
    // Фокус на опции "c" внутри панели (highlight при этом остаётся на "a").
    const optionEl = new Element();
    optionEl.closest = (sel) => (sel === ".kosmos-dd__option" ? optionEl : null);
    bindings.panelRef.value = { contains: (t) => t === optionEl };
    let prevented = false;
    bindings.onKey({
      key: "Enter",
      defaultPrevented: false,
      target: optionEl,
      preventDefault() {
        prevented = true;
      },
    });
    assert.equal(
      emitted.filter(([name]) => name === "update:modelValue").length,
      0,
      "Enter on a focused option is left to its native click",
    );
    assert.equal(prevented, false, "the focused option's click is not suppressed");

    // Enter при фокусе вне опций (search input, триггер) — по-прежнему
    // выбирает highlighted пункт.
    bindings.searchQuery.value = "";
    await nextTick();
    bindings.onKey({
      key: "Enter",
      defaultPrevented: false,
      target: new Element(),
      preventDefault() {},
    });
    const picked = emitted.filter(([name]) => name === "update:modelValue").map(([, v]) => v);
    assert.deepEqual(picked, ["a"], "Enter without option focus still picks the highlight");
    scope.stop();
  }

  // KOS-248 ContextMenu: меню крупнее окна — Math.min давал отрицательные
  // top/left (innerW - w - 8 < 0) и меню рендерилось за экраном целиком.
  // Clamp прижимает позицию к краю с тем же 8px отступом.
  {
    window.innerWidth = 300;
    window.innerHeight = 200;
    const { scope, bindings } = runSetup(ContextMenu, { open: true, x: 250, y: 150 });
    // Только root нужен watch'у — он читает его после nextTick.
    bindings.root.value = { offsetWidth: 400, offsetHeight: 500, contains: () => false };
    await nextTick();
    await nextTick();
    assert.equal(bindings.finalX.value, 8, "menu wider than the window is clamped to the edge");
    assert.equal(bindings.finalY.value, 8, "menu taller than the window is clamped to the edge");
    scope.stop();

    window.innerWidth = 1024;
    window.innerHeight = 768;
    const { scope: s2, bindings: b2 } = runSetup(ContextMenu, { open: true, x: 900, y: 700 });
    b2.root.value = { offsetWidth: 200, offsetHeight: 300, contains: () => false };
    await nextTick();
    await nextTick();
    assert.equal(b2.finalX.value, 1024 - 200 - 8, "normal case still right-clamps");
    assert.equal(b2.finalY.value, 768 - 300 - 8, "normal case still bottom-clamps");
    s2.stop();
  }

  // KOS-221 DateChip: label для malformed ISO-значения. Месяц "13" раньше
  // индексировал RU_MONTHS_SHORT вне массива → чип показывал буквальное
  // "5 undefined". Невалидные части → fallback на исходное value.
  {
    const { scope, bindings } = runSetup(DateChip, {
      value: "2024-13-05",
      placeholder: "Дата",
    });
    assert.equal(bindings.label.value, "2024-13-05", "malformed month shows raw value, not 'undefined'");
    assert.ok(!bindings.label.value.includes("undefined"));
    scope.stop();
  }
  {
    const { scope, bindings } = runSetup(DateChip, { value: "2024-05-07" });
    assert.equal(bindings.label.value, "7 май");
    scope.stop();
  }

  // KOS-221 HotkeyCapture: setup() не должен требовать `navigator` (в Node/
  // SSR его нет — раньше `navigator.platform` в setup падал ReferenceError).
  // И accelerator с "+" как основной клавишей ("Ctrl++") не должен терять
  // её в отображении — раньше показывался только модификатор.
  {
    const { scope, bindings } = runSetup(HotkeyCapture, { modelValue: "Ctrl++" });
    assert.deepEqual(bindings.keyParts.value, ["Ctrl", "+"], '"+" main key is preserved');
    scope.stop();
    const { scope: s2, bindings: b2 } = runSetup(HotkeyCapture, { modelValue: "Ctrl+B" });
    assert.deepEqual(b2.keyParts.value, ["Ctrl", "B"]);
    s2.stop();
  }

  console.log("vue-behavior tests passed");
} finally {
  console.warn = realWarn;
  rmSync(dir, { recursive: true, force: true });
}
