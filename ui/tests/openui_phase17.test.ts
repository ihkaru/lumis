import { describe, it, expect } from "bun:test";
import { OpenUiParser } from "../src/lib/components/openui/parser";
import { OpenUiEvaluator } from "../src/lib/components/openui/evaluator";
import { openUiBuiltins } from "../src/lib/components/openui/builtins";
import type { OpenUiContext } from "../src/lib/components/openui/types";

describe("OpenUI Phase 17: Declarative Data Visualizations (Charts & Media)", () => {
  it("should evaluate collection and transformation builtins accurately", () => {
    const rawData = [
      { id: 1, category: "Tech", score: 85 },
      { id: 2, category: "Health", score: 92 },
      { id: 3, category: "Finance", score: 78 },
      { id: 4, category: "Tech", score: 85 },
    ];

    // @Map property extraction
    const categories = openUiBuiltins["@Map"](rawData, "category");
    expect(categories).toEqual(["Tech", "Health", "Finance", "Tech"]);

    // @SortBy numeric ascending & descending
    const sortedAsc = openUiBuiltins["@SortBy"](rawData, "score", "asc");
    expect(sortedAsc[0].score).toBe(78);
    expect(sortedAsc[sortedAsc.length - 1].score).toBe(92);

    const sortedDesc = openUiBuiltins["@SortBy"](rawData, "score", "desc");
    expect(sortedDesc[0].score).toBe(92);

    // @Reverse array & string
    const reversed = openUiBuiltins["@Reverse"]([1, 2, 3, 4]);
    expect(reversed).toEqual([4, 3, 2, 1]);
    expect(openUiBuiltins["@Reverse"]("LUMIS")).toBe("SIMUL");

    // @Flatten nested array
    const nested = [[1, 2], [3, [4, 5]]];
    expect(openUiBuiltins["@Flatten"](nested)).toEqual([1, 2, 3, 4, 5]);

    // @Distinct primitives & objects
    expect(openUiBuiltins["@Distinct"](["a", "b", "a", "c", "b"])).toEqual(["a", "b", "c"]);
    const distinctObjs = openUiBuiltins["@Distinct"](rawData);
    expect(distinctObjs.length).toBe(4);
  });

  it("should construct and evaluate LineChart, BarChart, PieChart, DonutChart, and AreaChart", () => {
    const code = `root = Col([lc, bc, pc, dc, ac])
salesData = [{month: "Jan", val: 100}, {month: "Feb", val: 150}, {month: "Mar", val: 200}]
marketShare = [{label: "A", value: 40}, {label: "B", value: 35}, {label: "C", value: 25}]

lc = LineChart(salesData, "month", "val", "emerald", true, "Tren Penjualan Bulanan")
bc = BarChart(salesData, "month", "val", false, "blue", "Komparasi Omzet")
pc = PieChart(marketShare, "label", "value", "Pangsa Pasar")
dc = DonutChart(marketShare, "label", "value", 45, "Distribusi Penjualan")
ac = AreaChart(salesData, "month", "val", "violet", 0.3, "Volume Kumulatif")
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
    expect(ui?.type).toBe("Col");
    expect(ui?.children?.length).toBe(5);

    // LineChart
    const lineNode = ui?.children?.[0] as any;
    expect(lineNode.type).toBe("LineChart");
    expect(lineNode.props.xKey).toBe("month");
    expect(lineNode.props.yKey).toBe("val");
    expect(lineNode.props.title).toBe("Tren Penjualan Bulanan");
    expect(lineNode.props.data.length).toBe(3);

    // BarChart
    const barNode = ui?.children?.[1] as any;
    expect(barNode.type).toBe("BarChart");
    expect(barNode.props.horizontal).toBe(false);
    expect(barNode.props.title).toBe("Komparasi Omzet");

    // PieChart
    const pieNode = ui?.children?.[2] as any;
    expect(pieNode.type).toBe("PieChart");
    expect(pieNode.props.labelKey).toBe("label");
    expect(pieNode.props.valueKey).toBe("value");

    // DonutChart
    const donutNode = ui?.children?.[3] as any;
    expect(donutNode.type).toBe("DonutChart");
    expect(donutNode.props.innerRadius).toBe(45);

    // AreaChart
    const areaNode = ui?.children?.[4] as any;
    expect(areaNode.type).toBe("AreaChart");
    expect(areaNode.props.fillOpacity).toBe(0.3);
  });

  it("should construct and evaluate Carousel, Popover, Video, and Map components", () => {
    const code = `root = Card([car, pop, vid, geoMap])
car = Carousel([Text("Slide 1"), Text("Slide 2"), Text("Slide 3")], true, true)
pop = Popover(Button("Buka Info"), Text("Rincian metadata dokumen"), "bottom")
vid = Video("https://example.com/demo.mp4", "https://example.com/poster.jpg", true, false)
geoMap = Map(-6.1754, 106.8272, 14, [], "Monumen Nasional")
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
    expect(ui?.children?.length).toBe(4);

    // Carousel
    const carNode = ui?.children?.[0] as any;
    expect(carNode.type).toBe("Carousel");
    expect(carNode.children.length).toBe(3);
    expect(carNode.props.autoplay).toBe(true);

    // Popover
    const popNode = ui?.children?.[1] as any;
    expect(popNode.type).toBe("Popover");
    expect(popNode.props.trigger.type).toBe("Button");
    expect(popNode.props.content.type).toBe("Text");

    // Video
    const vidNode = ui?.children?.[2] as any;
    expect(vidNode.type).toBe("Video");
    expect(vidNode.props.src).toBe("https://example.com/demo.mp4");
    expect(vidNode.props.controls).toBe(true);

    // Map
    const mapNode = ui?.children?.[3] as any;
    expect(mapNode.type).toBe("Map");
    expect(mapNode.props.latitude).toBe(-6.1754);
    expect(mapNode.props.longitude).toBe(106.8272);
    expect(mapNode.props.title).toBe("Monumen Nasional");
  });
});
