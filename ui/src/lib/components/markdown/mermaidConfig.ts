import type { MermaidConfig } from "mermaid";

export function getMermaidConfig(isDark: boolean): MermaidConfig {
  return {
    startOnLoad: false,
    theme: isDark ? ("dark" as const) : ("default" as const),
    securityLevel: "loose" as const,
    suppressErrorRendering: true,
    fontFamily: "Google Sans Text, system-ui, sans-serif",
    themeVariables: isDark
      ? {
          darkMode: true,
          background: "transparent",
          primaryColor: "#1e293b",
          primaryTextColor: "#f8fafc",
          primaryBorderColor: "#38bdf8",
          lineColor: "#94a3b8",
          textColor: "#f8fafc",
          mainBkg: "#1e293b",
          nodeBorder: "#38bdf8",
          nodeTextColor: "#f8fafc",
          secondaryColor: "#0f172a",
          tertiaryColor: "#1e293b",
          clusterBkg: "#090d16",
          clusterBorder: "#475569",
          titleColor: "#38bdf8",
          edgeLabelBackground: "#090d16",
          attributeBackgroundColorOdd: "#1e293b",
          attributeBackgroundColorEven: "#0f172a",
          rowOdd: "#1e293b",
          rowEven: "#0f172a",
          entityBorder: "#38bdf8",
          entityBkg: "#0f172a",
          entityTextColor: "#f8fafc",
          actorBkg: "#1e293b",
          actorBorder: "#38bdf8",
          actorTextColor: "#f8fafc",
          actorLineColor: "#64748b",
          signalColor: "#94a3b8",
          signalTextColor: "#f8fafc",
          labelBoxBkgColor: "#1e293b",
          labelBoxBorderColor: "#38bdf8",
          labelTextColor: "#f8fafc",
          loopTextColor: "#f8fafc",
          noteBkgColor: "#1e293b",
          noteBorderColor: "#f59e0b",
          noteTextColor: "#f8fafc",
          activationBkgColor: "#334155",
          activationBorderColor: "#38bdf8",
          classText: "#f8fafc",
          stateLabelColor: "#f8fafc",
        }
      : {
          darkMode: false,
          background: "transparent",
          primaryColor: "#f1f5f9",
          primaryTextColor: "#0f172a",
          primaryBorderColor: "#0284c7",
          lineColor: "#475569",
          textColor: "#0f172a",
          mainBkg: "#ffffff",
          nodeBorder: "#0284c7",
          nodeTextColor: "#0f172a",
          secondaryColor: "#e2e8f0",
          tertiaryColor: "#f8fafc",
          clusterBkg: "#f8fafc",
          clusterBorder: "#cbd5e1",
          titleColor: "#0284c7",
          edgeLabelBackground: "#ffffff",
          attributeBackgroundColorOdd: "#f8fafc",
          attributeBackgroundColorEven: "#f1f5f9",
          rowOdd: "#ffffff",
          rowEven: "#f1f5f9",
          entityBorder: "#0284c7",
          entityBkg: "#ffffff",
          entityTextColor: "#0f172a",
          actorBkg: "#f8fafc",
          actorBorder: "#0284c7",
          actorTextColor: "#0f172a",
          actorLineColor: "#94a3b8",
          signalColor: "#475569",
          signalTextColor: "#0f172a",
          labelBoxBkgColor: "#f8fafc",
          labelBoxBorderColor: "#0284c7",
          labelTextColor: "#0f172a",
          loopTextColor: "#0f172a",
          noteBkgColor: "#fef3c7",
          noteBorderColor: "#d97706",
          noteTextColor: "#78350f",
          activationBkgColor: "#e2e8f0",
          activationBorderColor: "#0284c7",
          classText: "#0f172a",
          stateLabelColor: "#0f172a",
        },
  };
}

const NAMED_LIGHT_COLORS: Record<string, string> = {
  white: "#ffffff",
  lightgray: "#d3d3d3",
  lightgrey: "#d3d3d3",
  yellow: "#ffff00",
  lightyellow: "#ffffe0",
  lightblue: "#add8e6",
  lightcyan: "#e0ffff",
  lightgreen: "#90ee90",
  pink: "#ffc0cb",
  lightpink: "#ffb6c1",
  beige: "#f5f5dc",
  ivory: "#fffff0",
  floralwhite: "#fffaf0",
  wheat: "#f5deb3",
  papayawhip: "#ffefd5",
  blanchedalmond: "#ffebcd",
  cornsilk: "#fff8dc",
  lavender: "#e6e6fa",
  ghostwhite: "#f8f8ff",
  aliceblue: "#f0f8ff",
  honeydew: "#f0fff0",
  mintcream: "#f5fffa",
  azure: "#f0ffff",
  seashell: "#fff5ee",
  linen: "#faf0e6",
  oldlace: "#fdf5e6",
  snow: "#fffafa",
};

export function adaptColorForDarkMode(rawColor: string): string {
  let hex = rawColor.trim().toLowerCase();
  if (NAMED_LIGHT_COLORS[hex]) {
    hex = NAMED_LIGHT_COLORS[hex];
  }
  let clean = hex.replace("#", "").trim();
  if (clean.length === 3) {
    clean = clean.split("").map((c) => c + c).join("");
  }
  if (clean.length !== 6) return rawColor;
  const r = parseInt(clean.substring(0, 2), 16);
  const g = parseInt(clean.substring(2, 4), 16);
  const b = parseInt(clean.substring(4, 6), 16);
  if (isNaN(r) || isNaN(g) || isNaN(b)) return rawColor;

  const lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
  if (lum > 0.65) {
    // Blend 18% original hue with 82% slate-900 (#0f172a: 15, 23, 42)
    const nr = Math.round(r * 0.18 + 15 * 0.82);
    const ng = Math.round(g * 0.18 + 23 * 0.82);
    const nb = Math.round(b * 0.18 + 42 * 0.82);
    return `#${nr.toString(16).padStart(2, "0")}${ng.toString(16).padStart(2, "0")}${nb.toString(16).padStart(2, "0")}`;
  }
  return rawColor;
}

export function cleanMermaidSvg(rawSvg: string, isLightCanvas: boolean): string {
  let cleaned = rawSvg;
  if (!isLightCanvas) {
    cleaned = cleaned
      .replace(/fill="#333333"/gi, 'fill="currentColor"')
      .replace(/fill="#333"/gi, 'fill="currentColor"')
      .replace(/fill="#000000"/gi, 'fill="currentColor"')
      .replace(/fill="#fff5ad"/gi, 'fill="#1e293b"')
      .replace(/stroke="#000000"/gi, 'stroke="#64748b"')
      .replace(/stroke="#000"/gi, 'stroke="#64748b"')
      .replace(/stroke="#333333"/gi, 'stroke="#64748b"')
      .replace(/stroke="#333"/gi, 'stroke="#64748b"')
      .replace(/stroke:\s*#(?:000000|000|333333|333)\b/gi, 'stroke:#64748b');

    // Adapt light pastel fills in style blocks and inline attributes to dark pastel tones
    cleaned = cleaned.replace(/(fill\s*:\s*)(#[0-9a-fA-F]{3,8}|[a-zA-Z]+)(?=[\s;}"'])/g, (match, prefix, col) => {
      return `${prefix}${adaptColorForDarkMode(col)}`;
    });
    cleaned = cleaned.replace(/(fill\s*=\s*["'])(#[0-9a-fA-F]{3,8}|[a-zA-Z]+)(["'])/g, (match, p1, col, p3) => {
      if (col === "currentColor" || col === "none") return match;
      return `${p1}${adaptColorForDarkMode(col)}${p3}`;
    });
  } else {
    // In light canvas mode, do NOT replace white/light fills with currentColor (which is dark slate #0f172a)!
    // Light fills are correct and provide 18:1 contrast against dark text.
    cleaned = cleaned
      .replace(/fill="#000000"/gi, 'fill="#0f172a"')
      .replace(/fill="#000"/gi, 'fill="#0f172a"');
  }
  return cleaned;
}
