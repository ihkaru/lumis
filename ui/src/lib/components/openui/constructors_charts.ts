import type { UiNode } from "./types";

export function constructChartNode(name: string, args: any[]): UiNode | null {
  const props: Record<string, any> = {};

  switch (name) {
    case "LineChart":
      props.data = Array.isArray(args[0]) ? args[0] : [];
      props.xKey = args[1] ?? "x";
      props.yKey = args[2] ?? "y";
      props.color = args[3] ?? "primary";
      props.showAxes = args[4] ?? true;
      props.title = args[5];
      return { type: name, props, children: [] };

    case "BarChart":
      props.data = Array.isArray(args[0]) ? args[0] : [];
      props.xKey = args[1] ?? "x";
      props.yKey = args[2] ?? "y";
      props.horizontal = args[3] ?? false;
      props.color = args[4] ?? "primary";
      props.title = args[5];
      return { type: name, props, children: [] };

    case "PieChart":
      props.data = Array.isArray(args[0]) ? args[0] : [];
      props.labelKey = args[1] ?? "label";
      props.valueKey = args[2] ?? "value";
      props.title = args[3];
      return { type: name, props, children: [] };

    case "DonutChart":
      props.data = Array.isArray(args[0]) ? args[0] : [];
      props.labelKey = args[1] ?? "label";
      props.valueKey = args[2] ?? "value";
      props.innerRadius = typeof args[3] === "number" ? args[3] : 50;
      props.title = args[4];
      return { type: name, props, children: [] };

    case "AreaChart":
      props.data = Array.isArray(args[0]) ? args[0] : [];
      props.xKey = args[1] ?? "x";
      props.yKey = args[2] ?? "y";
      props.color = args[3] ?? "primary";
      props.fillOpacity = typeof args[4] === "number" ? args[4] : 0.25;
      props.title = args[5];
      return { type: name, props, children: [] };

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
