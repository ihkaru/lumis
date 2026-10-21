<script lang="ts">
  import type { UiNode } from "./types";
  import OpenUiNode from "./OpenUiNode.svelte";

  interface Props {
    node: any;
    state: Record<string, any>;
    onStateChange: (key: string, value: any) => void;
  }

  let { node, state: uiState, onStateChange }: Props = $props();

  let isFlipped = $state(false);
</script>

{#if node.type === "FlashCard"}
  <div class="relative w-full max-w-md mx-auto perspective-1000 py-2">
    <div
      role="button"
      tabindex="0"
      onclick={() => (isFlipped = !isFlipped)}
      onkeydown={(e) => e.key === "Enter" && (isFlipped = !isFlipped)}
      class="relative w-full min-h-48 rounded-xl border border-border/70 bg-card p-5 shadow-sm transition-all duration-500 cursor-pointer flex flex-col justify-between"
      style="transform-style: preserve-3d; transform: {isFlipped ? 'rotateY(180deg)' : 'rotateY(0deg)'};"
    >
      {#if !isFlipped}
        <div class="space-y-3">
          <div class="flex items-center justify-between text-xs text-muted-foreground font-medium">
            <span>Pertanyaan / Depan</span>
            <span class="rounded bg-primary/10 px-2 py-0.5 text-primary text-[10px]">Klik untuk balik ⟲</span>
          </div>
          <div class="py-2 text-foreground">
            <OpenUiNode node={node.props.front} state={uiState} {onStateChange} />
          </div>
        </div>
      {:else}
        <div class="space-y-3" style="transform: rotateY(180deg);">
          <div class="flex items-center justify-between text-xs text-emerald-400 font-medium">
            <span>Jawaban / Belakang</span>
            <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-emerald-400 text-[10px]">Terbalik ✓</span>
          </div>
          <div class="py-2 text-foreground">
            <OpenUiNode node={node.props.back} state={uiState} {onStateChange} />
          </div>
        </div>
      {/if}
      <div class="pt-2 text-center text-[10px] text-muted-foreground">
        {isFlipped ? "Klik lagi untuk kembali ke pertanyaan" : "Ketuk kartu untuk melihat jawaban"}
      </div>
    </div>
  </div>
{:else if node.type === "Confetti"}
  <div class="rounded-xl border border-emerald-500/30 bg-emerald-500/10 p-3.5 text-center space-y-1.5 animate-pulse">
    <div class="text-xl select-none">🎉 ✨ 🎊</div>
    <div class="text-xs font-semibold text-emerald-400">Selamat! Pencapaian Terbuka!</div>
    <div class="text-[11px] text-muted-foreground">Target dan kriteria berhasil dipenuhi dengan sempurna.</div>
  </div>
{:else if node.type === "Audio"}
  <div class="rounded-xl border border-border/60 bg-card/60 p-2.5">
    <!-- svelte-ignore a11y_media_has_caption -->
    <audio
      src={node.props.src}
      controls
      autoplay={node.props.autoplay ?? false}
      loop={node.props.loop ?? false}
      class="w-full h-9"
    ></audio>
  </div>
{/if}

<style>
  .perspective-1000 {
    perspective: 1000px;
  }
</style>
