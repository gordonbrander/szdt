export const _ = (selector, root = document) =>
  root.querySelector(`:scope ${selector}`);

export const on = (element, event, callback, options) => {
  element.addEventListener(event, callback, options);
  return () => {
    element.removeEventListener(event, callback, options);
  };
};

on(_("#menu"), "pointerdown", (event) => {
  event.preventDefault();
  _("#page").classList.toggle("sidebar-open");
});

on(_("#close"), "pointerdown", (event) => {
  event.preventDefault();
  _("#page").classList.remove("sidebar-open");
});
