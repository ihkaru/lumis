// Math and financial built-in functions for OpenUI
export const mathBuiltins: Record<string, (...args: any[]) => any> = {
  "@Sum": (arr: any) => {
    if (!Array.isArray(arr)) return 0;
    return arr.reduce((acc, v) => acc + (typeof v === "number" ? v : Number(v) || 0), 0);
  },

  "@Avg": (arr: any) => {
    if (!Array.isArray(arr) || arr.length === 0) return 0;
    const sum = mathBuiltins["@Sum"](arr);
    return sum / arr.length;
  },

  "@Min": (arr: any) => (Array.isArray(arr) && arr.length > 0 ? Math.min(...arr.map(Number)) : 0),
  "@Max": (arr: any) => (Array.isArray(arr) && arr.length > 0 ? Math.max(...arr.map(Number)) : 0),

  "@Round": (num: any, decimals: any = 0) => {
    const n = Number(num) || 0;
    const d = Number(decimals) || 0;
    const f = Math.pow(10, d);
    return Math.round(n * f) / f;
  },

  "@Floor": (num: any) => Math.floor(Number(num) || 0),
  "@Ceil": (num: any) => Math.ceil(Number(num) || 0),
  "@Abs": (num: any) => Math.abs(Number(num) || 0),

  "@Clamp": (num: any, min: any, max: any) => {
    const n = Number(num) || 0;
    const mi = Number(min);
    const ma = Number(max);
    return Math.min(Math.max(n, mi), ma);
  },

  "@Percent": (part: any, total: any) => {
    const t = Number(total) || 0;
    if (t === 0) return 0;
    return ((Number(part) || 0) / t) * 100;
  },

  "@Payment": (rate: any, nper: any, pv: any, fv: any = 0, type: any = 0) => {
    const r = Number(rate) || 0;
    const n = Number(nper) || 0;
    const p = Number(pv) || 0;
    const f = Number(fv) || 0;
    const t = Number(type) || 0;
    if (n === 0) return 0;
    if (r === 0) return -(p + f) / n;
    const pvif = Math.pow(1 + r, n);
    let pmt = (r / (pvif - 1)) * -(p * pvif + f);
    if (t === 1) pmt /= 1 + r;
    return pmt;
  },
};
