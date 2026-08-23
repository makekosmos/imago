// Глобальный scroll-fade listener — ставит `data-scrolling="1"` на любой
// элемент с классом `kosmos-scroll` пока он скроллится, и убирает атрибут
// через `idleMs` миллисекунд бездействия. Парный CSS — `.kosmos-scroll`
// в `packages/visuals/theme/css-variables.css`: thumb имеет alpha=0 по
// дефолту, alpha=0.18 при `[data-scrolling="1"]`, transition 450ms.
//
// Используется в shell main.ts и каждом extension main.ts через
// `installScrollFadeListener()`. Один listener на document с capture:true —
// он ловит scroll'ы любых вложенных контейнеров.

const scrollTimers = new WeakMap<HTMLElement, number>();
const scrollFadeOff = new WeakMap<Document | HTMLElement, () => void>();

export interface InstallScrollFadeOptions {
  /** Задержка перед снятием `data-scrolling`, мс. Default 600. */
  idleMs?: number;
  /** Опциональный root — по умолчанию `document`. */
  root?: Document | HTMLElement;
}

/**
 * Установить scroll-fade listener. Возвращает функцию-отписку.
 *
 * Idempotent — повторный вызов с тем же root возвращает прежнюю отписку
 * (новый listener не регистрируется), чтобы случайный двойной вызов
 * в Vue setup() не накапливал handler'ы.
 */
export function installScrollFadeListener(options: InstallScrollFadeOptions = {}): () => void {
  const idleMs = options.idleMs ?? 600;
  const root = options.root ?? document;

  const existingOff = scrollFadeOff.get(root);
  if (existingOff) {
    return existingOff;
  }

  const handler = (e: Event): void => {
    const target = e.target;
    if (!(target instanceof HTMLElement)) return;
    if (!target.classList.contains("kosmos-scroll")) return;
    target.setAttribute("data-scrolling", "1");
    const timer = scrollTimers.get(target);
    if (timer !== undefined) {
      window.clearTimeout(timer);
    }
    scrollTimers.set(
      target,
      window.setTimeout(
        () => target.removeAttribute("data-scrolling"),
        idleMs,
      ),
    );
  };

  root.addEventListener("scroll", handler, { capture: true, passive: true });

  const off = (): void => {
    root.removeEventListener("scroll", handler, true);
    scrollFadeOff.delete(root);
  };
  scrollFadeOff.set(root, off);
  return off;
}
