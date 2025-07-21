export type Cancel = () => void;

export const createCancelGroup = () => {
  const cancels = new Set<Cancel>();

  const cancel = () => {
    for (const cancel of cancels) {
      cancel();
    }
    cancels.clear();
  };

  const add = (cancel: Cancel) => {
    cancels.add(cancel);
  };

  return { cancel, add };
};
