// Random, shuffling, and sampling built-in functions for OpenUI
export const randomBuiltins: Record<string, (...args: any[]) => any> = {
  "@Random": (min: any = 0, max: any = 1, integer: any = false) => {
    const mi = Number(min) || 0;
    const ma = Number(max) || 1;
    const val = mi + Math.random() * (ma - mi);
    return integer ? Math.floor(val) : val;
  },

  "@Shuffle": (arr: any) => {
    if (!Array.isArray(arr)) return [];
    const copy = [...arr];
    for (let i = copy.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [copy[i], copy[j]] = [copy[j], copy[i]];
    }
    return copy;
  },

  "@Sample": (arr: any, count: any = 1) => {
    if (!Array.isArray(arr) || arr.length === 0) return [];
    const shuffled = randomBuiltins["@Shuffle"](arr);
    return shuffled.slice(0, Math.max(1, Number(count) || 1));
  },
};
