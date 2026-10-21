import { describe, it, expect } from "bun:test";
import { OpenUiParser } from "../src/lib/components/openui/parser";
import { OpenUiEvaluator } from "../src/lib/components/openui/evaluator";

describe("OpenUI Real World LLM Scenarios", () => {
  it("parses and evaluates Floorplan Canvas SVG with pipe lists and unquoted phrases", () => {
    const dsl = `root = Col("Denah Kantor Statistik", "lg")
$lantai = Lantai 1
$highlight = Semua
Col(
  Title("Denah Kantor Statistik - PST Sederhana", "lg"),
  Text("Skala 1 - 200. Ukuran gedung 16x11 meter. Pilar struktur beton diameter 40 cm.", "sm"),
  SegmentedControl($lantai, Lantai 1 | Lantai 2),
  SegmentedControl($highlight, Semua | Pelayanan | Operasional),
  Canvas(800, 460,
    Rect(10, 10, 780, 440, #f8fafc),
    Rect(30, 30, 740, 120, #e2e8f0),
    Rect(30, 170, 220, 140, #dbeafe),
    Rect(270, 170, 220, 140, #fef3c7),
    Rect(510, 170, 260, 140, #dcfce7),
    Rect(30, 330, 370, 100, #f1f5f9),
    Rect(420, 330, 350, 100, #ffe4e6),
    Rect(90, 70, 160, 45, #1e40af),
    Circle(270, 170, 14, #64748b),
    Circle(510, 170, 14, #64748b),
    Circle(270, 330, 14, #64748b),
    Circle(510, 330, 14, #64748b),
    SvgText(40, 50, LOBBY UTAMA, md),
    SvgText(95, 98, MEJA RESEPSIONIS, sm),
    SvgText(45, 220, RUANG PST, md),
    SvgText(45, 242, Pelayanan Statistik Terpadu, sm),
    SvgText(285, 220, RUANG KEPALA, md),
    SvgText(285, 242, Kepala Kantor + Tata Usaha, sm),
    SvgText(525, 220, RUANG OLAH DATA, md),
    SvgText(525, 242, Pengolahan + Server, sm),
    SvgText(45, 370, AULA / BRIEFING PETUGAS, md),
    SvgText(435, 370, MUSHOLLA + TOILET + ARSIP, md),
    SvgText(360, 445, PINTU UTAMA - 3m, sm)
  ),
  Row(
    Badge($lantai, info),
    Badge(RSE Tertib, success),
    Button(Reset Denah, @Reset($highlight))
  )
)`;

    const parser = new OpenUiParser();
    const evaluator = new OpenUiEvaluator();
    const prog = parser.parseProgram(dsl);
    expect(prog.statements.length).toBe(4);

    const ctx = {
      state: {},
      initialState: {},
      definitions: new Map(),
      handlers: {},
      setState: () => {},
    };

    const root = evaluator.evaluateProgram(prog, ctx);
    expect(root).not.toBeNull();
    expect(root?.type).toBe("Col");
    expect(root?.children?.length).toBe(6);

    // SegmentedControl options parsed via pipe |
    const segLantai = root?.children?.[2];
    expect(segLantai?.type).toBe("SegmentedControl");
    expect(segLantai?.props.options).toEqual(["Lantai 1", "Lantai 2"]);

    const segHighlight = root?.children?.[3];
    expect(segHighlight?.type).toBe("SegmentedControl");
    expect(segHighlight?.props.options).toEqual(["Semua", "Pelayanan", "Operasional"]);

    // Canvas variadic shapes
    const canvas = root?.children?.[4];
    expect(canvas?.type).toBe("Canvas");
    expect(canvas?.props.width).toBe(800);
    expect(canvas?.props.height).toBe(460);
    expect(canvas?.children?.length).toBe(23);

    // Check SVG text children phrases
    const svgTexts = canvas?.children?.filter((c: any) => c.type === "SvgText");
    expect(svgTexts.length).toBe(11);
    expect(svgTexts[0].props.text).toBe("LOBBY UTAMA");
    expect(svgTexts[0].props.fontSize).toBe(14); // md -> 14
    expect(svgTexts[1].props.text).toBe("MEJA RESEPSIONIS");
    expect(svgTexts[1].props.fontSize).toBe(12); // sm -> 12
    expect(svgTexts[3].props.text).toBe("Pelayanan Statistik Terpadu");
    expect(svgTexts[5].props.text).toBe("Kepala Kantor + Tata Usaha");
    expect(svgTexts[8].props.text).toBe("AULA / BRIEFING PETUGAS");
    expect(svgTexts[10].props.text).toBe("PINTU UTAMA - 3m");

    // Row buttons and badges
    const row = root?.children?.[5];
    expect(row?.type).toBe("Row");
    expect(row?.children?.length).toBe(3);
    expect(row?.children?.[1].props.text).toBe("RSE Tertib");
    expect(row?.children?.[2].props.label).toBe("Reset Denah");
  });

  it("parses and evaluates Inflation Dashboard with KPI Metrics, parallel arrays and charts", () => {
    const dsl = `root = Col("Dashboard Inflasi 2026", "lg")
$bulan = September
$view = Tren
Col(
  Title("Dashboard Inflasi Indonesia 2026 - BPS", "lg"),
  Text("Sumber: Berita Resmi Statistik BPS September 2026. IHK 2018=100.", "sm"),
  Row(
    Metric(Inflasi y-on-y Sep, 3.28%, IHK 112.31),
    Metric(Inflasi m-to-m, 0.30%, Sep 2026),
    Metric(Inflasi Inti y-on-y, 2.84%, Komponen inti),
    Metric(Inflasi y-to-d, 2.17%, Jan-Sep 2026)
  ),
  SegmentedControl($view, Tren | Pangan | Pengeluaran),
  Select($bulan, Jan | Feb | Mar | Apr | Mei | Jun | Jul | Agu | September),
  LineChart(Jan | Feb | Mar | Apr | Mei | Jun | Jul | Agu | Sep, 1.95 | 2.10 | 2.35 | 2.48 | 2.61 | 2.75 | 2.90 | 3.05 | 3.28, Tren Inflasi y-on-y 2026 Persen),
  BarChart(Beras | Cabai Merah | Bawang Merah | Minyak Goreng | Ayam Ras | Telur Ayam, 4.2 | 8.7 | 6.1 | 3.4 | 5.5 | 4.9, Komparasi Inflasi Pangan y-on-y Sep 2026 Persen),
  DonutChart(Makanan | Perumahan | Transportasi | Pendidikan | Kesehatan | Lainnya, 38.5 | 22.1 | 12.8 | 6.4 | 5.2 | 15.0, Komposisi Pengeluaran Rumah Tangga SBH Persen),
  Row(
    Badge(Target BI 1.5-3.5 Persen, warning),
    Button(Lihat September, @Set($bulan, September)),
    Button(Salin Angka Inti, @CopyToClipboard(2.84 persen September 2026))
  )
)`;

    const parser = new OpenUiParser();
    const evaluator = new OpenUiEvaluator();
    const prog = parser.parseProgram(dsl);

    const ctx = {
      state: {},
      initialState: {},
      definitions: new Map(),
      handlers: {},
      setState: () => {},
    };

    const root = evaluator.evaluateProgram(prog, ctx);
    expect(root).not.toBeNull();
    expect(root?.children?.length).toBe(9);

    // Row with 4 Metrics
    const row = root?.children?.[2];
    expect(row?.type).toBe("Row");
    expect(row?.children?.length).toBe(4);
    expect(row?.children?.[0].type).toBe("Metric");
    expect(row?.children?.[0].props.label).toBe("Inflasi y-on-y Sep");
    expect(row?.children?.[0].props.value).toBe("3.28%");
    expect(row?.children?.[0].props.description).toBe("IHK 112.31");

    // LineChart with zipped parallel data
    const lineChart = root?.children?.[5];
    expect(lineChart?.type).toBe("LineChart");
    expect(lineChart?.props.title).toBe("Tren Inflasi y-on-y 2026 Persen");
    expect(lineChart?.props.data.length).toBe(9);
    expect(lineChart?.props.data[0]).toEqual({ x: "Jan", y: 1.95, label: "Jan", value: 1.95 });
    expect(lineChart?.props.data[8]).toEqual({ x: "Sep", y: 3.28, label: "Sep", value: 3.28 });

    // BarChart with multi-word categories
    const barChart = root?.children?.[6];
    expect(barChart?.type).toBe("BarChart");
    expect(barChart?.props.title).toBe("Komparasi Inflasi Pangan y-on-y Sep 2026 Persen");
    expect(barChart?.props.data.length).toBe(6);
    expect(barChart?.props.data[1]).toEqual({ x: "Cabai Merah", y: 8.7, label: "Cabai Merah", value: 8.7 });

    // DonutChart with percentages
    const donutChart = root?.children?.[7];
    expect(donutChart?.type).toBe("DonutChart");
    expect(donutChart?.props.title).toBe("Komposisi Pengeluaran Rumah Tangga SBH Persen");
    expect(donutChart?.props.data.length).toBe(6);
    expect(donutChart?.props.data[0]).toEqual({ x: "Makanan", y: 38.5, label: "Makanan", value: 38.5 });

    // Bottom row badge and buttons
    const bottomRow = root?.children?.[8];
    expect(bottomRow?.children?.[0].props.text).toBe("Target BI 1.5-3.5 Persen");
    expect(bottomRow?.children?.[1].props.label).toBe("Lihat September");
  });
});
