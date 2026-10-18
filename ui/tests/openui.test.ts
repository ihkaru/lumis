import { describe, it, expect } from "bun:test";
import { OpenUiParser } from "../src/lib/components/openui/parser";
import { OpenUiEvaluator } from "../src/lib/components/openui/evaluator";
import type { OpenUiContext } from "../src/lib/components/openui/types";

describe("OpenUI Parser & Evaluator", () => {
  it("should parse and evaluate a shopping list with stepper", () => {
    const code = `root = Card([title, stepper, list, copy])
$guests = 4
$got = []
title = Title("Pasta night for " + @Plural($guests, "guest"), "lg")
stepper = Row([IconButton("minus", @Set($guests, @Clamp($guests - 1, 1, 20)), "Fewer"), Text($guests + "", "lg", null, "semibold"), IconButton("plus", @Set($guests, @Clamp($guests + 1, 1, 20)), "More")], 3)
lines = @Each(base, "i", {name: i.name, qty: @Round(i.qty * $guests / 4) + i.unit})
list = Col(@Each(lines, "l", Text(l.qty + " " + l.name)))
copy = Button("Copy list", @CopyToClipboard(@Join(@Each(lines, "l", l.qty + " " + l.name), "\\n")), "outline")
base = [{name: "spaghetti", qty: 400, unit: " g"}, {name: "tomatoes", qty: 6, unit: ""}, {name: "parmesan", qty: 80, unit: " g"}]`;

    const parser = new OpenUiParser();
    const program = parser.parseProgram(code);

    expect(program.statements.length).toBeGreaterThan(5);

    const ctx: OpenUiContext = {
      state: {},
      initialState: {},
      definitions: new Map(),
    };

    const evaluator = new OpenUiEvaluator();
    const ui = evaluator.evaluateProgram(program, ctx);

    expect(ui).not.toBeNull();
    expect(ui?.type).toBe("Card");
    expect(ctx.state["guests"]).toBe(4);
    expect(Array.isArray(ctx.state["got"])).toBe(true);

    // Test children
    const children = ui?.children || [];
    expect(children.length).toBe(4);

    // Title should be: "Pasta night for 4 guests"
    const titleNode = children[0] as any;
    expect(titleNode.type).toBe("Title");
    expect(titleNode.props.text).toBe("Pasta night for 4 guests");

    // Test action execution: trigger plus button action
    const stepperNode = children[1] as any;
    expect(stepperNode.type).toBe("Row");
    const plusBtn = stepperNode.children[2] as any;
    expect(typeof plusBtn.props.action).toBe("function");

    // Execute plus button
    plusBtn.props.action();
    expect(ctx.state["guests"]).toBe(5);

    // Re-evaluate with state=5
    const reEvaluated = evaluator.evaluateProgram(program, ctx);
    const updatedTitle = (reEvaluated?.children?.[0] as any)?.props?.text;
    expect(updatedTitle).toBe("Pasta night for 5 guests");
  });

  it("should evaluate built-in mathematical and array functions correctly", () => {
    const code = `root = Card([t1, t2, t3, t4])
nums = [10, 20, 30]
t1 = Text("Sum: " + @Sum(nums))
t2 = Text("Avg: " + @Avg(nums))
t3 = Text("Formatted: " + @Format(1250.5, "currency", 2, "USD"))
t4 = Text("Filtered Count: " + @Count(@Filter(nums, "", ">", 15)))`;

    const parser = new OpenUiParser();
    const program = parser.parseProgram(code);

    const ctx: OpenUiContext = {
      state: {},
      initialState: {},
      definitions: new Map(),
    };

    const evaluator = new OpenUiEvaluator();
    const ui = evaluator.evaluateProgram(program, ctx);

    const children = ui?.children as any[];
    expect(children[0].props.text).toBe("Sum: 60");
    expect(children[1].props.text).toBe("Avg: 20");
    expect(children[2].props.text).toContain("$1,250.50");
  });
});
