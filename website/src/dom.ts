import type { Cancel } from "./cancel.ts";

export const on = (
  element: Element,
  event: string,
  callback: EventListenerOrEventListenerObject,
): Cancel => {
  element.addEventListener(event, callback);
  return () => {
    element.removeEventListener(event, callback);
  };
};

export const $ = (
  selector: string,
  parent: Element | Document = document,
): Element | undefined => {
  return parent.querySelector(selector) ?? undefined;
};
