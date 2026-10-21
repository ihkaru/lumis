import type { UiNode } from "./types";

export function constructCanvasNode(name: string, args: any[]): UiNode | null {
  const props: Record<string, any> = {};

  switch (name) {
    case "Canvas": {
      props.width = typeof args[0] === "number" ? args[0] : 400;
      props.height = typeof args[1] === "number" ? args[1] : 250;
      const children = Array.isArray(args[2]) ? args[2] : args[2] ? [args[2]] : [];
      return { type: name, props, children };
    }

    case "Rect":
      props.x = typeof args[0] === "number" ? args[0] : 0;
      props.y = typeof args[1] === "number" ? args[1] : 0;
      props.width = typeof args[2] === "number" ? args[2] : 60;
      props.height = typeof args[3] === "number" ? args[3] : 40;
      props.fill = args[4] ?? "#3b82f6";
      props.stroke = args[5];
      props.rx = typeof args[6] === "number" ? args[6] : 4;
      return { type: name, props, children: [] };

    case "Circle":
      props.cx = typeof args[0] === "number" ? args[0] : 50;
      props.cy = typeof args[1] === "number" ? args[1] : 50;
      props.r = typeof args[2] === "number" ? args[2] : 25;
      props.fill = args[3] ?? "#10b981";
      props.stroke = args[4];
      return { type: name, props, children: [] };

    case "Line":
      props.x1 = typeof args[0] === "number" ? args[0] : 0;
      props.y1 = typeof args[1] === "number" ? args[1] : 0;
      props.x2 = typeof args[2] === "number" ? args[2] : 100;
      props.y2 = typeof args[3] === "number" ? args[3] : 100;
      props.stroke = args[4] ?? "#8b5cf6";
      props.strokeWidth = typeof args[5] === "number" ? args[5] : 2;
      return { type: name, props, children: [] };

    case "Path":
      props.d = args[0] ?? "";
      props.fill = args[1] ?? "none";
      props.stroke = args[2] ?? "currentColor";
      props.strokeWidth = typeof args[3] === "number" ? args[3] : 2;
      return { type: name, props, children: [] };

    case "SvgText":
      props.x = typeof args[0] === "number" ? args[0] : 0;
      props.y = typeof args[1] === "number" ? args[1] : 0;
      props.text = args[2] ?? "";
      props.fontSize = typeof args[3] === "number" ? args[3] : 12;
      props.fill = args[4] ?? "currentColor";
      return { type: name, props, children: [] };

    case "Group": {
      const children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.transform = args[1] ?? "";
      return { type: name, props, children };
    }

    case "FlashCard":
      props.front = args[0];
      props.back = args[1];
      props.isFlipped = args[2] ?? false;
      return { type: name, props, children: [] };

    case "Confetti":
      props.active = args[0] ?? true;
      props.duration = typeof args[1] === "number" ? args[1] : 3000;
      return { type: name, props, children: [] };

    case "Audio":
      props.src = args[0] ?? "";
      props.autoplay = args[1] ?? false;
      props.loop = args[2] ?? false;
      return { type: name, props, children: [] };

    default:
      return null;
  }
}
