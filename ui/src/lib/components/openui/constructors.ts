import type { AstExpr, OpenUiContext, UiNode } from "./types";

export function constructUiNode(
  name: string,
  rawArgs: AstExpr[],
  evalExpr: (expr: AstExpr, ctx: OpenUiContext, scope: Map<string, any>) => any,
  ctx: OpenUiContext,
  scope: Map<string, any>
): UiNode {
  const args = rawArgs.map((a) => evalExpr(a, ctx, scope));

  let bindingVar: string | undefined;
  if (["Slider", "Input", "SegmentedControl", "Checkbox", "RadioGroup"].includes(name)) {
    if (rawArgs[0]?.type === "state_var") {
      bindingVar = rawArgs[0].name;
    }
  }

  const props: Record<string, any> = { bindingVar };
  let children: any[] = [];

  switch (name) {
    case "Col":
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.gap = args[1] ?? 3;
      props.align = args[2] ?? "stretch";
      break;

    case "Row":
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.gap = args[1] ?? 2;
      props.align = args[2] ?? "center";
      props.justify = args[3] ?? "start";
      props.wrap = args[4] ?? false;
      break;

    case "Card":
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.variant = args[1] ?? "outline";
      props.padding = args[2] ?? 4;
      props.gap = args[3] ?? 3;
      break;

    case "Grid":
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.columns = args[1] ?? 2;
      props.minChildWidth = args[2] ?? 160;
      props.gap = args[3] ?? 3;
      break;

    case "Box":
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.direction = args[1] ?? "col";
      props.gap = args[2] ?? 2;
      props.padding = args[3] ?? 0;
      props.background = args[4];
      props.radius = args[5] ?? "md";
      props.border = args[6] ?? false;
      props.width = args[7];
      props.height = args[8];
      break;

    case "Title":
      props.text = args[0] ?? "";
      props.size = args[1] ?? "md";
      props.color = args[2];
      break;

    case "Text":
      props.text = args[0] ?? "";
      props.size = args[1] ?? "sm";
      props.color = args[2];
      props.weight = args[3] ?? "normal";
      props.italic = args[4] ?? false;
      props.lineThrough = args[5] ?? false;
      props.tabularNums = args[6] ?? false;
      break;

    case "Caption":
      props.text = args[0] ?? "";
      props.color = args[1] ?? "muted";
      break;

    case "Badge":
      props.text = args[0] ?? "";
      props.color = args[1] ?? "neutral";
      props.icon = args[2];
      break;

    case "Button":
      props.label = args[0] ?? "";
      props.action = args[1];
      props.variant = args[2] ?? "solid";
      props.color = args[3] ?? "primary";
      props.size = args[4] ?? "md";
      props.icon = args[5];
      props.disabled = args[6] ?? false;
      break;

    case "IconButton":
      props.icon = args[0] ?? "circle";
      props.action = args[1];
      props.label = args[2] ?? "";
      props.variant = args[3] ?? "ghost";
      break;

    case "Slider":
      props.value = args[0] ?? 0;
      props.min = args[1] ?? 0;
      props.max = args[2] ?? 100;
      props.step = args[3] ?? 1;
      props.label = args[4] ?? "";
      props.unit = args[5] ?? "";
      break;

    case "Input":
      props.value = args[0] ?? "";
      props.label = args[1] ?? "";
      props.type = args[2] ?? "text";
      props.placeholder = args[3] ?? "";
      break;

    case "SegmentedControl":
      props.value = args[0] ?? "";
      props.options = Array.isArray(args[1]) ? args[1] : [];
      props.block = args[2] ?? false;
      break;

    case "Checkbox":
      props.checked = !!args[0];
      props.label = args[1] ?? "";
      props.lineThrough = args[2] ?? false;
      break;

    case "Table":
      props.columns = Array.isArray(args[0]) ? args[0] : [];
      props.rows = Array.isArray(args[1]) ? args[1] : [];
      break;

    case "Icon":
      props.name = args[0] ?? "circle";
      props.size = args[1] ?? "sm";
      props.color = args[2];
      break;

    case "Pressable":
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.action = args[1];
      props.selected = args[2] ?? false;
      props.padding = args[3] ?? 2;
      props.background = args[4];
      props.radius = args[5] ?? "md";
      break;

    case "Spacer":
    case "Divider":
      break;

    default:
      children = Array.isArray(args[0]) ? args[0] : args[0] ? [args[0]] : [];
      props.rawArgs = args;
      break;
  }

  return { type: name, props, children };
}
