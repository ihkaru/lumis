// Array, string, and collection manipulation built-in functions for OpenUI
export const collectionBuiltins: Record<string, (...args: any[]) => any> = {
  "@Count": (arr: any) => (Array.isArray(arr) ? arr.length : typeof arr === "string" ? arr.length : 0),
  "@First": (arr: any) => (Array.isArray(arr) && arr.length > 0 ? arr[0] : null),
  "@Last": (arr: any) => (Array.isArray(arr) && arr.length > 0 ? arr[arr.length - 1] : null),

  "@Plural": (n: any, word: any, plural?: any) => {
    const num = Number(n) || 0;
    const w = String(word || "");
    const pl = plural ? String(plural) : `${w}s`;
    return `${num} ${num === 1 ? w : pl}`;
  },

  "@Format": (val: any, kind: any = "number", decimals?: any, currency: any = "USD") => {
    const n = Number(val) || 0;
    const d = typeof decimals === "number" ? decimals : kind === "currency" ? 2 : 0;
    if (kind === "currency") {
      try {
        return new Intl.NumberFormat("en-US", { style: "currency", currency, minimumFractionDigits: d, maximumFractionDigits: d }).format(n);
      } catch {
        return `$${n.toFixed(d)}`;
      }
    }
    if (kind === "percent") return `${n.toFixed(d)}%`;
    if (kind === "compact") {
      return new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: d }).format(n);
    }
    return new Intl.NumberFormat("en-US", { minimumFractionDigits: d, maximumFractionDigits: d }).format(n);
  },

  "@Join": (list: any, sep: any = ", ") => {
    if (!Array.isArray(list)) return "";
    return list.filter((x) => x !== null && x !== undefined).join(String(sep));
  },

  "@Includes": (listOrStr: any, item: any) => {
    if (Array.isArray(listOrStr)) return listOrStr.includes(item);
    if (typeof listOrStr === "string") return listOrStr.includes(String(item));
    return false;
  },

  "@Toggle": (list: any, item: any) => {
    if (!Array.isArray(list)) return [item];
    return list.includes(item) ? list.filter((x) => x !== item) : [...list, item];
  },

  "@Append": (list: any, item: any) => (Array.isArray(list) ? [...list, item] : [item]),
  "@Prepend": (list: any, item: any) => (Array.isArray(list) ? [item, ...list] : [item]),

  "@Remove": (list: any, field: any, value: any) => {
    if (!Array.isArray(list)) return [];
    if (!field || field === "") return list.filter((x) => x !== value);
    return list.filter((x) => x && x[field] !== value);
  },

  "@RemoveAt": (list: any, index: any) => {
    if (!Array.isArray(list)) return [];
    const idx = Number(index);
    const pos = idx < 0 ? list.length + idx : idx;
    return list.filter((_, i) => i !== pos);
  },

  "@Update": (list: any, field: any, value: any, setField: any, setValue: any) => {
    if (!Array.isArray(list)) return [];
    return list.map((item) => {
      if (item && item[field] === value) {
        return { ...item, [setField]: setValue };
      }
      return item;
    });
  },

  "@Range": (start: any, end: any, step: any = 1) => {
    const s = Number(start) || 0;
    const e = Number(end) || 0;
    const st = Number(step) || 1;
    const res: number[] = [];
    if (st > 0) {
      for (let i = s; i < e; i += st) res.push(i);
    } else if (st < 0) {
      for (let i = s; i > e; i += st) res.push(i);
    }
    return res;
  },

  "@Allocate": (total: any, weights: any, decimals: any = 2) => {
    const t = Number(total) || 0;
    if (!Array.isArray(weights) || weights.length === 0) return [];
    const sumW = weights.reduce((acc, w) => acc + (Number(w) || 0), 0);
    if (sumW === 0) return weights.map(() => 0);
    const d = Number(decimals) || 2;
    const factor = Math.pow(10, d);
    let allocatedTotal = 0;
    const raw = weights.map((w) => {
      const share = Math.floor(((Number(w) || 0) / sumW) * t * factor) / factor;
      allocatedTotal += share;
      return share;
    });
    const remainder = Math.round((t - allocatedTotal) * factor) / factor;
    if (raw.length > 0) raw[0] = Math.round((raw[0] + remainder) * factor) / factor;
    return raw;
  },

  "@Default": (val: any, fallback: any) => (val === null || val === undefined || val === "" || Number.isNaN(val) ? fallback : val),

  "@Slice": (target: any, start: any, end?: any) => {
    if (Array.isArray(target) || typeof target === "string") {
      return end !== undefined ? target.slice(Number(start), Number(end)) : target.slice(Number(start));
    }
    return target;
  },

  "@Split": (text: any, sep: any = ",") => {
    if (typeof text !== "string") return [];
    return text.split(String(sep)).map((s) => s.trim()).filter(Boolean);
  },

  "@Upper": (text: any) => String(text || "").toUpperCase(),
  "@Lower": (text: any) => String(text || "").toLowerCase(),
  "@Trim": (text: any) => String(text || "").trim(),
  "@Capitalize": (text: any) =>
    String(text || "")
      .split(" ")
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
      .join(" "),

  "@Filter": (arr: any, field: any, op: any, val: any) => {
    if (!Array.isArray(arr)) return [];
    return arr.filter((item) => {
      if (!item) return false;
      const v = item[field];
      if (op === "==") return v == val;
      if (op === "!=") return v != val;
      if (op === ">") return v > val;
      if (op === "<") return v < val;
      if (op === ">=") return v >= val;
      if (op === "<=") return v <= val;
      if (op === "contains") return String(v).includes(String(val));
      return false;
    });
  },

  "@Sort": (arr: any, field: any, direction: any = "asc") => {
    if (!Array.isArray(arr)) return [];
    const copy = [...arr];
    copy.sort((a, b) => {
      const va = a ? a[field] : undefined;
      const vb = b ? b[field] : undefined;
      if (va === vb) return 0;
      if (va > vb) return direction === "desc" ? -1 : 1;
      return direction === "desc" ? 1 : -1;
    });
    return copy;
  },

  "@SortBy": (arr: any, field: any, direction: any = "asc") => {
    return collectionBuiltins["@Sort"](arr, field, direction);
  },

  "@Top": (rows: any, field: any, n: any, direction: any = "desc") => {
    const sorted = collectionBuiltins["@Sort"](rows, field, direction);
    return sorted.slice(0, Number(n) || 1);
  },

  "@GroupBy": (rows: any, key: any, field?: any, agg: any = "sum") => {
    if (!Array.isArray(rows)) return [];
    const groups = new Map<string, number[]>();
    for (const r of rows) {
      if (!r) continue;
      const k = String(r[key] ?? "");
      const list = groups.get(k) || [];
      list.push(field ? Number(r[field]) || 0 : 1);
      groups.set(k, list);
    }
    const result: { key: string; value: number }[] = [];
    for (const [k, vals] of groups.entries()) {
      let val = 0;
      if (agg === "count") val = vals.length;
      else if (agg === "avg") val = vals.length ? vals.reduce((a, b) => a + b, 0) / vals.length : 0;
      else if (agg === "min") val = vals.length ? Math.min(...vals) : 0;
      else if (agg === "max") val = vals.length ? Math.max(...vals) : 0;
      else val = vals.reduce((a, b) => a + b, 0);
      result.push({ key: k, value: val });
    }
    return result;
  },

  "@Map": (arr: any, keyOrFn: any) => {
    if (!Array.isArray(arr)) return [];
    if (typeof keyOrFn === "string") {
      return arr.map((item) => (item && typeof item === "object" ? item[keyOrFn] : item));
    }
    if (typeof keyOrFn === "function") {
      return arr.map(keyOrFn);
    }
    return arr;
  },

  "@Reverse": (arr: any) => {
    if (Array.isArray(arr)) return [...arr].reverse();
    if (typeof arr === "string") return arr.split("").reverse().join("");
    return arr;
  },

  "@Flatten": (arr: any) => {
    if (!Array.isArray(arr)) return [];
    return arr.flat(2);
  },

  "@Distinct": (arr: any) => {
    if (!Array.isArray(arr)) return [];
    if (arr.length > 0 && typeof arr[0] === "object" && arr[0] !== null) {
      const seen = new Set<string>();
      return arr.filter((item) => {
        const key = JSON.stringify(item);
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });
    }
    return Array.from(new Set(arr));
  },
};
