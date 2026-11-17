// The highlight of a segmented control (`.seg`): a `.seg-indicator` child that springs
// to the chosen option, the one with aria-selected, aria-pressed or aria-checked set to
// true. It
// follows the option when labels appear or the window resizes.
import { useLayoutEffect, type RefObject } from "react";

export function useIndicator(container: RefObject<HTMLElement>, chosen: unknown): void {
  useLayoutEffect(() => {
    const box = container.current;
    if (!box) return;
    const place = () => {
      const marker = box.querySelector<HTMLElement>(":scope > .seg-indicator");
      if (!marker) return;
      const option = box.querySelector<HTMLElement>(
        ':scope > [aria-selected="true"], :scope > [aria-pressed="true"], :scope > [aria-checked="true"]',
      );
      if (!option) {
        marker.style.opacity = "0";
        return;
      }
      marker.style.opacity = "1";
      marker.style.transform = `translate(${option.offsetLeft}px, ${option.offsetTop}px)`;
      marker.style.width = `${option.offsetWidth}px`;
      marker.style.height = `${option.offsetHeight}px`;
    };
    place();
    // Moves animate only once the first position is set, so it does not fly in.
    const frame = requestAnimationFrame(() => box.setAttribute("data-ready", ""));
    const resized = new ResizeObserver(place);
    resized.observe(box);
    for (const child of Array.from(box.children)) resized.observe(child);
    return () => {
      cancelAnimationFrame(frame);
      resized.disconnect();
    };
  }, [container, chosen]);
}
