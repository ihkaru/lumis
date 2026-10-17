<script lang="ts">
  import ZoomIn from "lucide-svelte/icons/zoom-in";
  import ZoomOut from "lucide-svelte/icons/zoom-out";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import ArrowUp from "lucide-svelte/icons/arrow-up";
  import ArrowDown from "lucide-svelte/icons/arrow-down";
  import ArrowLeft from "lucide-svelte/icons/arrow-left";
  import ArrowRight from "lucide-svelte/icons/arrow-right";

  interface Props {
    scale: number;
    step?: number;
    onPan: (dx: number, dy: number) => void;
    onZoom: (factor: number) => void;
    onReset: () => void;
  }

  let { scale, step = 60, onPan, onZoom, onReset }: Props = $props();
</script>

<div class="flex items-center gap-1 pointer-events-auto select-none">
  <div class="grid grid-cols-3 gap-0.5 p-1 rounded-md bg-slate-900/85 backdrop-blur-md border border-white/10 shadow-lg text-slate-300">
    <div></div>
    <button type="button" onclick={() => onPan(0, step)} class="p-1 rounded hover:bg-white/15" title="Pan Atas"><ArrowUp class="h-3 w-3" /></button>
    <div></div>
    <button type="button" onclick={() => onPan(step, 0)} class="p-1 rounded hover:bg-white/15" title="Pan Kiri"><ArrowLeft class="h-3 w-3" /></button>
    <button type="button" onclick={onReset} class="p-1 rounded hover:bg-white/15 font-mono text-[9px] text-center" title="Reset">{Math.round(scale * 100)}%</button>
    <button type="button" onclick={() => onPan(-step, 0)} class="p-1 rounded hover:bg-white/15" title="Pan Kanan"><ArrowRight class="h-3 w-3" /></button>
    <div></div>
    <button type="button" onclick={() => onPan(0, -step)} class="p-1 rounded hover:bg-white/15" title="Pan Bawah"><ArrowDown class="h-3 w-3" /></button>
    <div></div>
  </div>
  <div class="flex flex-col gap-0.5 p-1 rounded-md bg-slate-900/85 backdrop-blur-md border border-white/10 shadow-lg text-slate-300">
    <button type="button" onclick={() => onZoom(1.2)} class="p-1 rounded hover:bg-white/15" title="Zoom In"><ZoomIn class="h-3 w-3" /></button>
    <button type="button" onclick={() => onZoom(0.8)} class="p-1 rounded hover:bg-white/15" title="Zoom Out"><ZoomOut class="h-3 w-3" /></button>
    <button type="button" onclick={onReset} class="p-1 rounded hover:bg-white/15" title="Reset"><RotateCcw class="h-2.5 w-2.5" /></button>
  </div>
</div>
