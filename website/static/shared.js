export const createCancelGroup = () => {
  const cancels = new Set();

  const cancel = () => {
    for (const cancel of cancels) {
      cancel();
    }
    cancels.clear();
  };

  const add = (cancel) => {
    cancels.add(cancel);
  };

  return { cancel, add };
};

export const on = (
  element,
  event,
  callback,
) => {
  element.addEventListener(event, callback);
  return () => {
    element.removeEventListener(event, callback);
  };
};

export const $ = (selector, parent = document) => {
  return parent.querySelector(selector) ?? undefined;
};
