import type { UiNode } from "./types";

function normalizeSeriesData(arg0: any, arg1: any): { data: any[]; isZipped: boolean } {
  if (Array.isArray(arg0) && Array.isArray(arg1)) {
    const data = arg0.map((label, i) => ({
      x: String(label),
      y: Number(arg1[i]) || 0,
      label: String(label),
      value: Number(arg1[i]) || 0,
    }));
    return { data, isZipped: true };
  }
  return { data: Array.isArray(arg0) ? arg0 : [], isZipped: false };
}

export function constructChartNode(name: string, args: any[]): UiNode | null {
  const props: Record<string, any> = {};

  switch (name) {
    case "LineChart": {
      const { data, isZipped } = normalizeSeriesData(args[0], args[1]);
      props.data = data;
      props.xKey = isZipped ? "x" : args[1] ?? "x";
      props.yKey = isZipped ? "y" : args[2] ?? "y";
      props.color = args[3] ?? "primary";
      props.showAxes = args[4] ?? true;
      props.title = isZipped ? (typeof args[2] === "string" ? args[2] : args[5]) : args[5];
      return { type: name, props, children: [] };
    }

    case "BarChart": {
      const { data, isZipped } = normalizeSeriesData(args[0], args[1]);
      props.data = data;
      props.xKey = isZipped ? "x" : args[1] ?? "x";
      props.yKey = isZipped ? "y" : args[2] ?? "y";
      props.horizontal = args[3] ?? false;
      props.color = args[4] ?? "primary";
      props.title = isZipped ? (typeof args[2] === "string" ? args[2] : args[5]) : args[5];
      return { type: name, props, children: [] };
    }

    case "PieChart": {
      const { data, isZipped } = normalizeSeriesData(args[0], args[1]);
      props.data = data;
      props.labelKey = isZipped ? "label" : args[1] ?? "label";
      props.valueKey = isZipped ? "value" : args[2] ?? "value";
      props.title = isZipped ? (typeof args[2] === "string" ? args[2] : args[3]) : args[3];
      return { type: name, props, children: [] };
    }

    case "DonutChart": {
      const { data, isZipped } = normalizeSeriesData(args[0], args[1]);
      props.data = data;
      props.labelKey = isZipped ? "label" : args[1] ?? "label";
      props.valueKey = isZipped ? "value" : args[2] ?? "value";
      props.innerRadius = typeof args[3] === "number" ? args[3] : 50;
      props.title = isZipped ? (typeof args[2] === "string" ? args[2] : args[4]) : args[4];
      return { type: name, props, children: [] };
    }

    case "AreaChart": {
      const { data, isZipped } = normalizeSeriesData(args[0], args[1]);
      props.data = data;
      props.xKey = isZipped ? "x" : args[1] ?? "x";
      props.yKey = isZipped ? "y" : args[2] ?? "y";
      props.color = args[3] ?? "primary";
      props.fillOpacity = typeof args[4] === "number" ? args[4] : 0.25;
      props.title = isZipped ? (typeof args[2] === "string" ? args[2] : args[5]) : args[5];
      return { type: name, props, children: [] };
    }

    case "Carousel": {
      const children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.autoplay = args[1] ?? false;
      props.loop = args[2] ?? true;
      return { type: name, props, children };
    }

    case "Popover":
      props.trigger = args[0];
      props.content = args[1];
      props.position = args[2] ?? "bottom";
      return { type: name, props, children: [] };

    case "Video":
      props.src = args[0] ?? "";
      props.poster = args[1] ?? "";
      props.controls = args[2] ?? true;
      props.autoplay = args[3] ?? false;
      return { type: name, props, children: [] };

    case "Map":
      props.latitude = typeof args[0] === "number" ? args[0] : -6.1754;
      props.longitude = typeof args[1] === "number" ? args[1] : 106.8272;
      props.zoom = typeof args[2] === "number" ? args[2] : 13;
      props.markers = Array.isArray(args[3]) ? args[3] : [];
      props.title = args[4] ?? "";
      return { type: name, props, children: [] };

    default:
      return null;
  }
}
