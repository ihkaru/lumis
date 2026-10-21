import { describe, it, expect } from "bun:test";
import { OpenUiParser } from "../src/lib/components/openui/parser";
import { OpenUiEvaluator } from "../src/lib/components/openui/evaluator";
import { openUiBuiltins } from "../src/lib/components/openui/builtins";
import type { OpenUiContext } from "../src/lib/components/openui/types";

describe("OpenUI Phase 18: Vector Canvas Primitives & Interactive Play", () => {
  it("should evaluate random, shuffle, and sample builtins correctly", () => {
    // @Random
    const rInt = openUiBuiltins["@Random"](10, 20, true);
    expect(rInt).toBeGreaterThanOrEqual(10);
    expect(rInt).toBeLessThanOrEqual(20);
    expect(Number.isInteger(rInt)).toBe(true);

    const rFloat = openUiBuiltins["@Random"](0, 5, false);
    expect(rFloat).toBeGreaterThanOrEqual(0);
    expect(rFloat).toBeLessThanOrEqual(5);

    // @Shuffle
    const items = [1, 2, 3, 4, 5, 6, 7, 8];
    const shuffled = openUiBuiltins["@Shuffle"](items);
    expect(shuffled.length).toBe(items.length);
    expect(shuffled.sort()).toEqual(items.sort());

    // @Sample
    const samples = openUiBuiltins["@Sample"](items, 3);
    expect(samples.length).toBe(3);
    for (const s of samples) {
      expect(items).toContain(s);
    }
  });

  it("should construct and evaluate Vector Canvas primitives (Canvas, Rect, Circle, Line, Path, SvgText, Group)", () => {
    const code = `root = Canvas(500, 300, [r1, c1, l1, p1, txt, grp])
r1 = Rect(10, 10, 80, 50, "#3b82f6", "#1d4ed8", 6)
c1 = Circle(150, 40, 30, "#10b981", "#047857")
l1 = Line(200, 20, 280, 80, "#8b5cf6", 3)
p1 = Path("M 10 100 Q 50 150 90 100", "none", "#f59e0b", 2)
txt = SvgText(20, 140, "Arsitektur Vektor Bebas", 14, "#0f172a")
grp = Group([Rect(0, 0, 40, 40, "#ec4899")], "translate(300, 20)")
`;

    const parser = new OpenUiParser();
    const program = parser.parseProgram(code);

    const ctx: OpenUiContext = {
      state: {},
      initialState: {},
      definitions: new Map(),
      parser,
    };

    const evaluator = new OpenUiEvaluator();
    const ui = evaluator.evaluateProgram(program, ctx);

    expect(ui).not.toBeNull();
    expect(ui?.type).toBe("Canvas");
    expect(ui?.props.width).toBe(500);
    expect(ui?.props.height).toBe(300);
    expect(ui?.children?.length).toBe(6);

    // Rect
    const rectNode = ui?.children?.[0] as any;
    expect(rectNode.type).toBe("Rect");
    expect(rectNode.props.x).toBe(10);
    expect(rectNode.props.width).toBe(80);
    expect(rectNode.props.rx).toBe(6);

    // Circle
    const circleNode = ui?.children?.[1] as any;
    expect(circleNode.type).toBe("Circle");
    expect(circleNode.props.cx).toBe(150);
    expect(circleNode.props.r).toBe(30);

    // Line
    const lineNode = ui?.children?.[2] as any;
    expect(lineNode.type).toBe("Line");
    expect(lineNode.props.x1).toBe(200);
    expect(lineNode.props.strokeWidth).toBe(3);

    // Path
    const pathNode = ui?.children?.[3] as any;
    expect(pathNode.type).toBe("Path");
    expect(pathNode.props.d).toBe("M 10 100 Q 50 150 90 100");

    // SvgText
    const textNode = ui?.children?.[4] as any;
    expect(textNode.type).toBe("SvgText");
    expect(textNode.props.text).toBe("Arsitektur Vektor Bebas");

    // Group
    const grpNode = ui?.children?.[5] as any;
    expect(grpNode.type).toBe("Group");
    expect(grpNode.props.transform).toBe("translate(300, 20)");
    expect(grpNode.children.length).toBe(1);
  });

  it("should construct and evaluate FlashCard, Confetti, and Audio components", () => {
    const code = `root = Card([card, cnf, aud])
card = FlashCard(Text("Apa kepanjangan BPS?"), Text("Badan Pusat Statistik"), false)
cnf = Confetti(true, 5000)
aud = Audio("https://example.com/sound.mp3", false, true)
`;

    const parser = new OpenUiParser();
    const program = parser.parseProgram(code);

    const ctx: OpenUiContext = {
      state: {},
      initialState: {},
      definitions: new Map(),
      parser,
    };

    const evaluator = new OpenUiEvaluator();
    const ui = evaluator.evaluateProgram(program, ctx);

    expect(ui).not.toBeNull();
    expect(ui?.children?.length).toBe(3);

    // FlashCard
    const flashNode = ui?.children?.[0] as any;
    expect(flashNode.type).toBe("FlashCard");
    expect(flashNode.props.front.type).toBe("Text");
    expect(flashNode.props.back.type).toBe("Text");

    // Confetti
    const cnfNode = ui?.children?.[1] as any;
    expect(cnfNode.type).toBe("Confetti");
    expect(cnfNode.props.active).toBe(true);
    expect(cnfNode.props.duration).toBe(5000);

    // Audio
    const audNode = ui?.children?.[2] as any;
    expect(audNode.type).toBe("Audio");
    expect(audNode.props.src).toBe("https://example.com/sound.mp3");
    expect(audNode.props.loop).toBe(true);
  });
});
