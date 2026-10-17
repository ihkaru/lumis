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
          primaryColor: "#0284c7",
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

export function cleanMermaidSvg(rawSvg: string, isLightCanvas: boolean): string {
  let cleaned = rawSvg;
  if (!isLightCanvas) {
    cleaned = cleaned
      .replace(/fill="#333333"/gi, 'fill="currentColor"')
      .replace(/fill="#333"/gi, 'fill="currentColor"')
      .replace(/fill="#000000"/gi, 'fill="currentColor"')
      .replace(/fill="#fff5ad"/gi, 'fill="#1e293b"');
  } else {
    cleaned = cleaned
      .replace(/fill="#ffffff"/gi, 'fill="currentColor"')
      .replace(/fill="#fff"/gi, 'fill="currentColor"')
      .replace(/fill="#f8fafc"/gi, 'fill="currentColor"')
      .replace(/fill="#f1f5f9"/gi, 'fill="currentColor"');
  }
  return cleaned;
}
