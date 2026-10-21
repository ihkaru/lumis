// Date, time, and duration built-in functions for OpenUI
export const dateBuiltins: Record<string, (...args: any[]) => any> = {
  "@Today": () => new Date().toISOString().split("T")[0],
  "@Now": () => new Date().toISOString(),

  "@FormatDate": (date: any, style: any = "date") => {
    if (!date) return "";
    try {
      const d = new Date(date);
      if (isNaN(d.getTime())) return String(date);
      if (style === "short") return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
      if (style === "long") return d.toLocaleDateString("en-US", { weekday: "long", year: "numeric", month: "long", day: "numeric" });
      if (style === "time") return d.toLocaleTimeString("en-US", { hour: "numeric", minute: "numeric" });
      if (style === "iso") return d.toISOString().split("T")[0];
      return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
    } catch {
      return String(date);
    }
  },

  "@AddDays": (date: any, days: any) => {
    const d = new Date(date || Date.now());
    d.setDate(d.getDate() + (Number(days) || 0));
    return d.toISOString().split("T")[0];
  },

  "@AddMonths": (date: any, months: any) => {
    const d = new Date(date || Date.now());
    d.setMonth(d.getMonth() + (Number(months) || 0));
    return d.toISOString().split("T")[0];
  },

  "@DaysBetween": (d1: any, d2: any) => {
    const t1 = new Date(d1).getTime();
    const t2 = new Date(d2).getTime();
    return Math.round((t2 - t1) / (1000 * 60 * 60 * 24));
  },

  "@MonthsBetween": (d1: any, d2: any) => {
    const date1 = new Date(d1);
    const date2 = new Date(d2);
    return (date2.getFullYear() - date1.getFullYear()) * 12 + (date2.getMonth() - date1.getMonth());
  },

  "@Between": (val: any, min: any, max: any) => {
    const v = Number(val);
    return v >= Number(min) && v <= Number(max);
  },

  "@Duration": (seconds: any, format?: any) => {
    const s = Math.max(0, Math.floor(Number(seconds) || 0));
    const hrs = Math.floor(s / 3600);
    const mins = Math.floor((s % 3600) / 60);
    const secs = s % 60;
    if (format === "long") {
      const parts: string[] = [];
      if (hrs > 0) parts.push(`${hrs} hr${hrs > 1 ? "s" : ""}`);
      if (mins > 0) parts.push(`${mins} min${mins > 1 ? "s" : ""}`);
      if (secs > 0 || parts.length === 0) parts.push(`${secs} sec${secs > 1 ? "s" : ""}`);
      return parts.join(" ");
    }
    if (hrs > 0) {
      return `${hrs}:${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
    }
    return `${mins}:${String(secs).padStart(2, "0")}`;
  },

  "@StartOf": (date: any, unit: any = "month") => {
    const d = new Date(date || Date.now());
    if (unit === "year") return `${d.getFullYear()}-01-01`;
    if (unit === "month") {
      const m = String(d.getMonth() + 1).padStart(2, "0");
      return `${d.getFullYear()}-${m}-01`;
    }
    if (unit === "week") {
      const day = d.getDay();
      const diff = d.getDate() - day + (day === 0 ? -6 : 1);
      const mon = new Date(d.setDate(diff));
      return mon.toISOString().split("T")[0];
    }
    return d.toISOString().split("T")[0];
  },
};
