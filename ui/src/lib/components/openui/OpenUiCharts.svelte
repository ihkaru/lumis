<script lang="ts">
  import type { UiNode } from "./types";

  interface Props {
    node: UiNode;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node }: Props = $props();

  const PALETTE = ["#3b82f6", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4", "#f97316"];

  function getPoints(data: any[], xKey: string, yKey: string, width = 330, height = 110, padLeft = 45, padTop = 20) {
    if (!Array.isArray(data) || data.length === 0) return { points: [], min: 0, max: 1, labels: [] };
    const vals = data.map((d) => (typeof d === "object" && d !== null ? Number(d[yKey]) || 0 : Number(d) || 0));
    const labels = data.map((d) => (typeof d === "object" && d !== null ? String(d[xKey] ?? "") : ""));
    const min = Math.min(...vals, 0);
    const max = Math.max(...vals, 1);
    const range = max - min || 1;
    const n = Math.max(1, vals.length - 1);

    const points = vals.map((v, i) => {
      const x = padLeft + (i / n) * width;
      const y = padTop + height - ((v - min) / range) * height;
      return { x, y, val: v, label: labels[i] };
    });

    return { points, min, max, labels };
  }

  function getPieSlices(data: any[], labelKey: string, valueKey: string, cx = 95, cy = 95, r = 75, innerR = 0) {
    if (!Array.isArray(data) || data.length === 0) return { slices: [], total: 0 };
    const total = data.reduce((acc, d) => acc + (typeof d === "object" && d !== null ? Number(d[valueKey]) || 0 : Number(d) || 0), 0) || 1;

    let currentAngle = -Math.PI / 2;
    const slices = data.map((d, i) => {
      const val = typeof d === "object" && d !== null ? Number(d[valueKey]) || 0 : Number(d) || 0;
      const label = typeof d === "object" && d !== null ? String(d[labelKey] ?? "") : `Item ${i + 1}`;
      const angle = (val / total) * 2 * Math.PI;
      const endAngle = currentAngle + angle;
      const color = PALETTE[i % PALETTE.length];

      const x1 = cx + r * Math.cos(currentAngle);
      const y1 = cy + r * Math.sin(currentAngle);
      const x2 = cx + r * Math.cos(endAngle);
      const y2 = cy + r * Math.sin(endAngle);
      const largeArc = angle > Math.PI ? 1 : 0;

      let path = "";
      if (innerR > 0) {
        const x3 = cx + innerR * Math.cos(endAngle);
        const y3 = cy + innerR * Math.sin(endAngle);
        const x4 = cx + innerR * Math.cos(currentAngle);
        const y4 = cy + innerR * Math.sin(currentAngle);
        path = `M ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} L ${x3} ${y3} A ${innerR} ${innerR} 0 ${largeArc} 0 ${x4} ${y4} Z`;
      } else {
        path = `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} Z`;
      }

      currentAngle = endAngle;
      const pct = Math.round((val / total) * 100);
      return { path, color, label, val, pct };
    });

    return { slices, total };
  }
</script>

<div class="rounded-xl border border-border/60 bg-card/75 p-3.5 backdrop-blur-xs shadow-xs space-y-2 w-full overflow-hidden">
  {#if node.props.title}
    <div class="text-xs font-semibold text-foreground tracking-tight">{node.props.title}</div>
  {/if}

  {#if node.type === "LineChart"}
    {@const { points, min, max } = getPoints(node.props.data, node.props.xKey, node.props.yKey)}
    {@const pathD = points.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(" ")}
    <svg viewBox="0 0 400 160" class="w-full h-auto text-muted-foreground select-none">
      <line x1="45" y1="20" x2="375" y2="20" stroke="currentColor" stroke-dasharray="2,2" class="opacity-20" />
      <line x1="45" y1="75" x2="375" y2="75" stroke="currentColor" stroke-dasharray="2,2" class="opacity-20" />
      <line x1="45" y1="130" x2="375" y2="130" stroke="currentColor" class="opacity-40" />
      <text x="40" y="24" text-anchor="end" class="text-[9px] fill-muted-foreground">{max}</text>
      <text x="40" y="133" text-anchor="end" class="text-[9px] fill-muted-foreground">{min}</text>
      <path d={pathD} fill="none" stroke="#3b82f6" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
      {#each points as pt}
        <circle cx={pt.x} cy={pt.y} r="3.5" class="fill-[#3b82f6] stroke-background stroke-2" />
        {#if pt.label}
          <text x={pt.x} y="148" text-anchor="middle" class="text-[9px] fill-muted-foreground">{pt.label}</text>
        {/if}
      {/each}
    </svg>
  {:else if node.type === "AreaChart"}
    {@const { points, min, max } = getPoints(node.props.data, node.props.xKey, node.props.yKey)}
    {@const lineD = points.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(" ")}
    {@const areaD = points.length > 0 ? `${lineD} L ${points[points.length - 1].x.toFixed(1)} 130 L ${points[0].x.toFixed(1)} 130 Z` : ""}
    <svg viewBox="0 0 400 160" class="w-full h-auto select-none">
      <defs>
        <linearGradient id="openui-area-grad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.45" />
          <stop offset="100%" stop-color="#3b82f6" stop-opacity="0.02" />
        </linearGradient>
      </defs>
      <line x1="45" y1="130" x2="375" y2="130" stroke="currentColor" class="opacity-30" />
      <text x="40" y="24" text-anchor="end" class="text-[9px] fill-muted-foreground">{max}</text>
      <text x="40" y="133" text-anchor="end" class="text-[9px] fill-muted-foreground">{min}</text>
      <path d={areaD} fill="url(#openui-area-grad)" />
      <path d={lineD} fill="none" stroke="#3b82f6" stroke-width="2" />
      {#each points as pt}
        <circle cx={pt.x} cy={pt.y} r="3" class="fill-[#3b82f6] stroke-background stroke-2" />
        {#if pt.label}
          <text x={pt.x} y="148" text-anchor="middle" class="text-[9px] fill-muted-foreground">{pt.label}</text>
        {/if}
      {/each}
    </svg>
  {:else if node.type === "BarChart"}
    {@const { points, max } = getPoints(node.props.data, node.props.xKey, node.props.yKey, 320, 100, 50, 20)}
    {@const barWidth = Math.max(10, Math.min(32, 280 / Math.max(1, points.length)))}
    <svg viewBox="0 0 400 160" class="w-full h-auto select-none">
      <line x1="45" y1="120" x2="385" y2="120" stroke="currentColor" class="opacity-30" />
      <text x="40" y="24" text-anchor="end" class="text-[9px] fill-muted-foreground">{max}</text>
      <text x="40" y="123" text-anchor="end" class="text-[9px] fill-muted-foreground">0</text>
      {#each points as pt, i}
        {@const h = Math.max(2, 120 - pt.y)}
        <rect
          x={pt.x - barWidth / 2}
          y={pt.y}
          width={barWidth}
          height={h}
          rx="3"
          fill={PALETTE[i % PALETTE.length]}
          class="transition-opacity hover:opacity-80"
        />
        <text x={pt.x} y={pt.y - 4} text-anchor="middle" class="text-[9px] font-medium fill-foreground">{pt.val}</text>
        {#if pt.label}
          <text x={pt.x} y="136" text-anchor="middle" class="text-[9px] fill-muted-foreground">{pt.label}</text>
        {/if}
      {/each}
    </svg>
  {:else if node.type === "PieChart" || node.type === "DonutChart"}
    {@const innerR = node.type === "DonutChart" ? (node.props.innerRadius ?? 42) : 0}
    {@const { slices, total } = getPieSlices(node.props.data, node.props.labelKey, node.props.valueKey, 95, 95, 75, innerR)}
    <div class="flex flex-col sm:flex-row items-center gap-4">
      <svg viewBox="0 0 190 190" class="h-36 w-36 select-none shrink-0">
        {#each slices as slice}
          <path d={slice.path} fill={slice.color} class="transition-opacity hover:opacity-85" />
        {/each}
        {#if node.type === "DonutChart"}
          <text x="95" y="93" text-anchor="middle" class="text-xs font-semibold fill-foreground">{total}</text>
          <text x="95" y="106" text-anchor="middle" class="text-[8px] fill-muted-foreground uppercase">Total</text>
        {/if}
      </svg>
      <div class="grid grid-cols-2 sm:grid-cols-1 gap-1.5 w-full text-xs">
        {#each slices as slice}
          <div class="flex items-center justify-between gap-2 pr-2">
            <div class="flex items-center gap-1.5 min-w-0">
              <span class="h-2.5 w-2.5 rounded-full shrink-0" style="background-color: {slice.color};"></span>
              <span class="truncate text-foreground/90 text-xs">{slice.label}</span>
            </div>
            <span class="font-mono text-muted-foreground shrink-0">{slice.pct}%</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
