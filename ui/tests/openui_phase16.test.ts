import { describe, it, expect } from "bun:test";
import { OpenUiParser } from "../src/lib/components/openui/parser";
import { OpenUiEvaluator } from "../src/lib/components/openui/evaluator";
import { openUiBuiltins } from "../src/lib/components/openui/builtins";
import type { OpenUiContext } from "../src/lib/components/openui/types";

describe("OpenUI Phase 16: Form, Content & Financial Primitives", () => {
  it("should evaluate financial @Payment builtin accurately", () => {
    // Zero-rate test: PV = 1200 over 12 months = -100/mo
    const zeroRatePmt = openUiBuiltins["@Payment"](0, 12, 1200);
    expect(zeroRatePmt).toBe(-100);

    // Standard mortgage: 6% annual (0.5% monthly = 0.005), 360 months (30 years), 100,000 principal
    const mortgagePmt = openUiBuiltins["@Payment"](0.005, 360, 100000);
    expect(Math.round(mortgagePmt * 100) / 100).toBe(-599.55);

    // Annuity due (type = 1)
    const annuityDue = openUiBuiltins["@Payment"](0.005, 360, 100000, 0, 1);
    expect(Math.round(annuityDue * 100) / 100).toBe(-596.57);
  });

  it("should evaluate date and duration builtins correctly", () => {
    const today = openUiBuiltins["@Today"]();
    expect(today).toMatch(/^\d{4}-\d{2}-\d{2}$/);

    const now = openUiBuiltins["@Now"]();
    expect(now).toContain("T");

    const addedDays = openUiBuiltins["@AddDays"]("2026-10-01", 10);
    expect(addedDays).toBe("2026-10-11");

    const addedMonths = openUiBuiltins["@AddMonths"]("2026-01-15", 3);
    expect(addedMonths).toBe("2026-04-15");

    const daysDiff = openUiBuiltins["@DaysBetween"]("2026-10-01", "2026-10-15");
    expect(daysDiff).toBe(14);

    const monthsDiff = openUiBuiltins["@MonthsBetween"]("2026-01-01", "2026-07-01");
    expect(monthsDiff).toBe(6);

    expect(openUiBuiltins["@Between"](50, 10, 100)).toBe(true);
    expect(openUiBuiltins["@Between"](5, 10, 100)).toBe(false);

    expect(openUiBuiltins["@Duration"](125)).toBe("2:05");
    expect(openUiBuiltins["@Duration"](3665)).toBe("1:01:05");
    expect(openUiBuiltins["@Duration"](3665, "long")).toBe("1 hr 1 min 5 secs");

    expect(openUiBuiltins["@StartOf"]("2026-10-21", "month")).toBe("2026-10-01");
    expect(openUiBuiltins["@StartOf"]("2026-10-21", "year")).toBe("2026-01-01");
  });

  it("should construct and evaluate DatePicker, Timer, Form, List, Label, Markdown, and Math components", () => {
    const code = `root = Card([form, timerBox])
$loanDate = "2026-10-01"
$elapsed = 120
form = Form([lbl, dp, mathBlock, mdBlock, itemList], @Set($elapsed, 0))
lbl = Label("Tanggal Pengajuan Pinjaman", true, "loan_date")
dp = DatePicker($loanDate, "Pilih Tanggal", "2026-01-01", "2026-12-31")
mathBlock = Math("PMT = (r * PV) / (1 - (1 + r)^(-n))")
mdBlock = Markdown("**Ketentuan**: Bunga flat *0.5%* per bulan.")
itemList = List([ListItem(Text("Syarat 1")), ListItem(Text("Syarat 2"))], true)
timerBox = Timer($elapsed, "Sisa Waktu")
`;

    const parser = new OpenUiParser();
    const program = parser.parseProgram(code);

    let submitted = false;
    const state: Record<string, any> = { loanDate: "2026-10-01", elapsed: 120 };
    const ctx: OpenUiContext = {
      state,
      initialState: { loanDate: "2026-10-01", elapsed: 120 },
      definitions: new Map(),
      parser,
      setState: (k, v) => {
        state[k] = v;
        if (k === "elapsed" && v === 0) submitted = true;
      },
    };

    const evaluator = new OpenUiEvaluator();
    const ui = evaluator.evaluateProgram(program, ctx);

    expect(ui).not.toBeNull();
    expect(ui?.type).toBe("Card");
    expect(ui?.children?.length).toBe(2);

    const formNode = ui?.children?.[0] as any;
    expect(formNode.type).toBe("Form");
    expect(formNode.children.length).toBe(5);

    // Label
    const labelNode = formNode.children[0];
    expect(labelNode.type).toBe("Label");
    expect(labelNode.props.text).toBe("Tanggal Pengajuan Pinjaman");
    expect(labelNode.props.required).toBe(true);

    // DatePicker
    const dpNode = formNode.children[1];
    expect(dpNode.type).toBe("DatePicker");
    expect(dpNode.props.bindingVar).toBe("loanDate");
    expect(dpNode.props.min).toBe("2026-01-01");
    expect(dpNode.props.max).toBe("2026-12-31");

    // Math
    const mathNode = formNode.children[2];
    expect(mathNode.type).toBe("Math");
    expect(mathNode.props.formula).toBe("PMT = (r * PV) / (1 - (1 + r)^(-n))");

    // Markdown
    const mdNode = formNode.children[3];
    expect(mdNode.type).toBe("Markdown");
    expect(mdNode.props.content).toContain("Ketentuan");

    // List
    const listNode = formNode.children[4];
    expect(listNode.type).toBe("List");
    expect(listNode.props.ordered).toBe(true);
    expect(listNode.props.items.length).toBe(2);

    // Timer
    const timerNode = ui?.children?.[1] as any;
    expect(timerNode.type).toBe("Timer");
    expect(timerNode.props.bindingVar).toBe("elapsed");

    // Form onSubmit action execution
    expect(typeof formNode.props.onSubmit).toBe("function");
    formNode.props.onSubmit();
    expect(submitted).toBe(true);
    expect(state.elapsed).toBe(0);
  });
});
