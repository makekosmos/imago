// Стек открытых overlay-слоёв для координации Escape-dismissal.
//
// Каждый overlay (Modal, Dropdown, DateTimePicker, ContextMenu, DateChip,
// StatusDot, QuickEntryPanel, CommandPalette, expanded TodoRow,
// capturing HotkeyCapture) захватывает токен, пока он открыт. При Escape
// срабатывает только верхний слой — нижние проверяют `isTopEscapeLayer`.
//
// Overlay-обработчики вешаются на document с capture: true и вызывают
// e.preventDefault() — capture-фаза идёт раньше element-level keydown'ов
// (инпуты внутри панелей, expanded-редактор TodoRow и т.п.), а те пропускают
// события с `e.defaultPrevented`. Регистрация-порядок между overlay'ями не
// важен: срабатывает только top-of-stack.

import { onScopeDispose, shallowRef, watch, type Ref, type WatchSource } from "vue";

const escapeStack: symbol[] = [];

/** true, если `token` — верхний слой в стеке (ему принадлежит Escape). */
export function isTopEscapeLayer(token: symbol | null): boolean {
  return token !== null && escapeStack[escapeStack.length - 1] === token;
}

/**
 * Держит токен в стеке пока `active` truthy; снимает при close и при
 * остановке scope'а (unmount / dispose). Возвращает
 * Ref<symbol|null> — null, когда слой не в стеке. Компоненты, которым
 * нужно только «занять место» в стеке (QuickEntryPanel, CommandPalette),
 * могут игнорировать результат.
 */
export function useEscapeLayer(active: WatchSource<boolean>): Readonly<Ref<symbol | null>> {
  const token = shallowRef<symbol | null>(null);

  const pop = () => {
    if (token.value === null) return;
    const idx = escapeStack.lastIndexOf(token.value);
    if (idx >= 0) escapeStack.splice(idx, 1);
    token.value = null;
  };

  // Vue 3.6: watcher cleanups (onWatcherCleanup) выполняются при КАЖДОМ
  // запуске эффекта — даже когда cb отбрасывается дедупликацией (переход
  // open=false→true внутри одного flush'а). Старая схема «push + cleanup
  // снимает» снимала слой, а cb его уже не восстанавливал: переоткрытый
  // overlay оставался без токена и переставал отвечать на Escape/клавиши.
  // Идемпотентные push/pop через значение cb переживают дедупликацию.
  watch(
    active,
    (isActive) => {
      if (isActive && token.value === null) {
        token.value = Symbol("escape-layer");
        escapeStack.push(token.value);
      } else if (!isActive) {
        pop();
      }
    },
    { immediate: true },
  );
  onScopeDispose(pop);

  return token;
}
