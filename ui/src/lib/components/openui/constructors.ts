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
  if (["Slider", "Input", "SegmentedControl", "Checkbox", "RadioGroup", "DatePicker", "Timer"].includes(name)) {
    if (rawArgs[0]?.type === "state_var") {
      bindingVar = rawArgs[0].name;
    }
  }

  const props: Record<string, any> = { bindingVar };
  let children: any[] = [];

  const extractChildren = (rawArgs: any[]): { children: any[]; rest: any[] } => {
    if (Array.isArray(rawArgs[0])) {
      return { children: rawArgs[0], rest: rawArgs.slice(1) };
    }
    const nodes: any[] = [];
    const rest: any[] = [];
    for (const a of rawArgs) {
      if (a && typeof a === "object" && "type" in a) {
        nodes.push(a);
      } else if (Array.isArray(a)) {
        nodes.push(...a);
      } else if (nodes.length === 0 && (typeof a === "string" || typeof a === "number" || typeof a === "boolean")) {
        nodes.push(a);
      } else {
        rest.push(a);
      }
    }
    return { children: nodes.length > 0 ? nodes : (rawArgs[0] ? [rawArgs[0]] : []), rest };
  };

  switch (name) {
    case "Col": {
      const { children: c, rest } = extractChildren(args);
      children = c;
      props.gap = typeof rest[0] === "number" ? rest[0] : 3;
      props.align = typeof rest[1] === "string" ? rest[1] : "stretch";
      break;
    }

    case "Row": {
      const { children: c, rest } = extractChildren(args);
      children = c;
      props.gap = typeof rest[0] === "number" ? rest[0] : 2;
      props.align = typeof rest[1] === "string" ? rest[1] : "center";
      props.justify = typeof rest[2] === "string" ? rest[2] : "start";
      props.wrap = typeof rest[3] === "boolean" ? rest[3] : false;
      break;
    }

    case "Card": {
      const { children: c, rest } = extractChildren(args);
      children = c;
      props.variant = typeof rest[0] === "string" ? rest[0] : "outline";
      props.padding = typeof rest[1] === "number" ? rest[1] : 4;
      props.gap = typeof rest[2] === "number" ? rest[2] : 3;
      break;
    }

    case "Grid": {
      const { children: c, rest } = extractChildren(args);
      children = c;
      props.columns = typeof rest[0] === "number" ? rest[0] : 2;
      props.minChildWidth = typeof rest[1] === "number" ? rest[1] : 160;
      props.gap = typeof rest[2] === "number" ? rest[2] : 3;
      break;
    }

    case "Box": {
      const { children: c, rest } = extractChildren(args);
      children = c;
      props.direction = typeof rest[0] === "string" ? rest[0] : "col";
      props.gap = typeof rest[1] === "number" ? rest[1] : 2;
      props.padding = typeof rest[2] === "number" ? rest[2] : 0;
      props.background = rest[3];
      props.radius = rest[4] ?? "md";
      props.border = rest[5] ?? false;
      props.width = rest[6];
      props.height = rest[7];
      break;
    }

    case "Subtitle":
      props.text = args[0] ?? "";
      props.size = args[1] ?? "sm";
      props.color = "muted";
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

    case "TextArea":
      props.value = args[0] ?? "";
      props.placeholder = args[1] ?? "";
      props.label = args[2] ?? "";
      props.rows = typeof args[3] === "number" ? args[3] : 3;
      break;

    case "Select":
      props.value = args[0] ?? "";
      props.options = Array.isArray(args[1]) ? args[1] : [];
      props.label = args[2] ?? "";
      break;

    case "RadioGroup":
      props.value = args[0] ?? "";
      props.options = Array.isArray(args[1]) ? args[1] : [];
      props.label = args[2] ?? "";
      break;

    case "Link":
      props.text = args[0] ?? "";
      props.url = args[1] ?? "#";
      break;

    case "Code":
      props.text = args[0] ?? "";
      props.language = args[1] ?? "";
      break;

    case "Quote":
      props.text = args[0] ?? "";
      props.author = args[1] ?? "";
      break;

    case "Image":
      props.src = args[0] ?? "";
      props.alt = args[1] ?? "";
      props.width = args[2];
      props.height = args[3];
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

    case "DatePicker":
      props.value = args[0] ?? "";
      props.label = args[1] ?? "";
      props.min = args[2];
      props.max = args[3];
      break;

    case "Timer":
      props.seconds = typeof args[0] === "number" ? args[0] : 0;
      props.label = args[1] ?? "";
      break;

    case "List":
      props.items = Array.isArray(args[0]) ? args[0] : [];
      props.ordered = args[1] ?? false;
      break;

    case "ListItem":
      props.content = args[0] ?? "";
      break;

    case "Label":
      props.text = args[0] ?? "";
      props.required = args[1] ?? false;
      props.forId = args[2];
      break;

    case "Markdown":
      props.content = args[0] ?? "";
      break;

    case "Math":
      props.formula = args[0] ?? "";
      break;

    case "Form": {
      const { children: c, rest } = extractChildren(args);
      children = c;
      props.onSubmit = rest[0];
      break;
    }

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
