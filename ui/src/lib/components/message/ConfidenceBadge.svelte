<script lang="ts">
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import ShieldAlert from "lucide-svelte/icons/shield-alert";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import Info from "lucide-svelte/icons/info";

  interface Props {
    score?: number; // 0.0 - 1.0
    groundingRatio?: number; // 0.0 - 1.0
    class?: string;
  }

  let { score = 0.95, groundingRatio = 0.9, class: className = "" }: Props = $props();

  let isHigh = $derived(score >= 0.85);
  let isMedium = $derived(score >= 0.65 && score < 0.85);
  let isLow = $derived(score < 0.65);

  let percentage = $derived(Math.round(score * 100));
  let groundingPct = $derived(Math.round(groundingRatio * 100));
</script>

<div
  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-[10px] font-mono border select-none transition-colors {isHigh ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400' : isMedium ? 'bg-amber-500/10 border-amber-500/30 text-amber-400' : 'bg-rose-500/10 border-rose-500/30 text-rose-400'} {className}"
  title="Skor Validitas: {percentage}% | Sumber Terverifikasi: {groundingPct}%"
>
  {#if isHigh}
    <ShieldCheck class="w-3 h-3 text-emerald-400 shrink-0" />
    <span class="font-medium">{percentage}% Terverifikasi</span>
  {:else if isMedium}
    <ShieldAlert class="w-3 h-3 text-amber-400 shrink-0" />
    <span class="font-medium">{percentage}% Moderat</span>
  {:else}
    <AlertTriangle class="w-3 h-3 text-rose-400 shrink-0" />
    <span class="font-medium">Perlu Validasi ({percentage}%)</span>
  {/if}
</div>
