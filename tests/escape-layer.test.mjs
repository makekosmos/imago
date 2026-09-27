import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

// useEscapeLayer.ts — чистый TS-модуль; транспилируем его тем же typescript,
// что гоняет typecheck, и импортируем как ESM. Файл кладём в tests/.tmp-*,
// чтобы голый specifier "vue" резолвился в корневые node_modules.
const dir = mkdtempSync(join(import.meta.dirname, ".tmp-escape-layer-"));

try {
  const transpiled = ts.transpileModule(
    readFileSync(join(import.meta.dirname, "../packages/vue/src/composables/useEscapeLayer.ts"), "utf8"),
    { compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 } },
  ).outputText;
  writeFileSync(join(dir, "useEscapeLayer.mjs"), transpiled);

  const { isTopEscapeLayer, useEscapeLayer } = await import(
    pathToFileURL(join(dir, "useEscapeLayer.mjs")).href
  );
  const { effectScope, nextTick, ref } = await import("vue");

  const tick = () => nextTick();

  // Верхний слой — последний открытый; после его закрытия Escape
  // возвращается предыдущему слою.
  {
    const aOpen = ref(false);
    const bOpen = ref(false);
    const scope = effectScope();
    let tokenA;
    let tokenB;
    scope.run(() => {
      tokenA = useEscapeLayer(aOpen);
      tokenB = useEscapeLayer(bOpen);
    });

    aOpen.value = true;
    await tick();
    assert.equal(isTopEscapeLayer(tokenA.value), true, "single layer is top");

    bOpen.value = true;
    await tick();
    assert.equal(isTopEscapeLayer(tokenB.value), true, "newer layer is top");
    assert.equal(isTopEscapeLayer(tokenA.value), false, "lower layer no longer top");

    bOpen.value = false;
    await tick();
    assert.equal(tokenB.value, null, "closed layer releases token");
    assert.equal(isTopEscapeLayer(tokenA.value), true, "previous layer regains top");

    scope.stop();
    await tick();
    assert.equal(tokenA.value, null, "scope stop releases still-open layer");
    assert.equal(isTopEscapeLayer(tokenA.value), false, "no layer is top after dispose");
  }

  // Закрытие среднего слоя не ломает стек: нижний остаётся на месте,
  // верхний по-прежнему top.
  {
    const low = ref(true);
    const mid = ref(true);
    const top = ref(true);
    const scope = effectScope();
    let tLow;
    let tMid;
    let tTop;
    scope.run(() => {
      tLow = useEscapeLayer(low);
      tMid = useEscapeLayer(mid);
      tTop = useEscapeLayer(top);
    });
    await tick();
    mid.value = false;
    await tick();
    assert.equal(isTopEscapeLayer(tTop.value), true, "top stays top after mid closes");
    assert.equal(isTopEscapeLayer(tMid.value), false, "closed mid not top");
    top.value = false;
    await tick();
    assert.equal(isTopEscapeLayer(tLow.value), true, "bottom layer regains top");
    scope.stop();
  }

  // null-токен и незанятый слой никогда не top; повторное открытие даёт
  // новый токен и не дублирует записи в стеке.
  {
    const open = ref(false);
    const scope = effectScope();
    const token = scope.run(() => useEscapeLayer(open));
    assert.equal(token.value, null, "inactive layer has no token");
    assert.equal(isTopEscapeLayer(null), false, "null token never top");
    open.value = true;
    await tick();
    const first = token.value;
    assert.equal(isTopEscapeLayer(first), true);
    open.value = false;
    await tick();
    open.value = true;
    await tick();
    assert.notEqual(token.value, first, "reopen pushes a fresh token");
    assert.equal(isTopEscapeLayer(token.value), true);
    scope.stop();
    await tick();
    assert.equal(isTopEscapeLayer(first), false, "stale token not top after dispose");
  }

  console.log("escape-layer tests passed");
} finally {
  rmSync(dir, { recursive: true, force: true });
}
