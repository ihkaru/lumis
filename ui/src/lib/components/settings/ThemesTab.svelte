<script lang="ts">
  import Sun from "lucide-svelte/icons/sun";
  import Moon from "lucide-svelte/icons/moon";
  import Palette from "lucide-svelte/icons/palette";
  import Check from "lucide-svelte/icons/check";

  export type ThemePalette = "llumen" | "dracula" | "vitesse" | "monochrome";

  interface Props {
    isDark: boolean;
    palette: ThemePalette;
    onToggleDark: (dark: boolean) => void;
    onSelectPalette: (palette: ThemePalette) => void;
  }

  let { isDark = true, palette = "llumen", onToggleDark, onSelectPalette }: Props = $props();

  const palettes = [
    { id: "llumen" as ThemePalette, name: "Llumen Standard", primary: "#06b6d4" },
    { id: "dracula" as ThemePalette, name: "Dracula Enterprise", primary: "#bd93f9" },
    { id: "vitesse" as ThemePalette, name: "Vitesse Minimal", primary: "#4ade80" },
    { id: "monochrome" as ThemePalette, name: "Monochrome (Hitam Putih)", primary: "#71717a" },
  ];
</script>

<div class="space-y-6 font-sans text-xs">
  <!-- Dark / Light Mode Toggle -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between p-3.5 rounded-xl border border-border bg-card gap-3">
    <div class="space-y-0.5 min-w-0">
      <div class="font-semibold text-sm text-foreground flex items-center gap-2">
        {#if isDark}
          <Moon class="h-4 w-4 text-primary shrink-0" />
        {:else}
          <Sun class="h-4 w-4 text-amber-400 shrink-0" />
        {/if}
        <span class="[overflow-wrap:anywhere] break-words">Mode Tampilan</span>
      </div>
      <p class="text-muted-foreground text-[11px] [overflow-wrap:anywhere] break-words">
        Pilih skema warna gelap atau terang untuk kenyamanan mata.
      </p>
    </div>
    <div class="flex rounded-lg border border-border p-1 bg-muted/40 shrink-0 self-start sm:self-auto">
      <button
        type="button"
        onclick={() => onToggleDark(false)}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium cursor-pointer transition-colors min-h-[44px] sm:min-h-0 touch-manipulation {!isDark ? 'bg-background text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground'}"
      >
        <Sun class="h-3.5 w-3.5 shrink-0" />
        <span>Terang</span>
      </button>
      <button
        type="button"
        onclick={() => onToggleDark(true)}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium cursor-pointer transition-colors min-h-[44px] sm:min-h-0 touch-manipulation {isDark ? 'bg-background text-foreground font-semibold shadow-xs' : 'text-muted-foreground hover:text-foreground'}"
      >
        <Moon class="h-3.5 w-3.5 shrink-0" />
        <span>Gelap</span>
      </button>
    </div>
  </div>

  <!-- Color Palette Switcher -->
  <div class="space-y-2">
    <div class="font-semibold text-sm text-foreground flex items-center gap-2">
      <Palette class="h-4 w-4 text-primary shrink-0" />
      <span class="[overflow-wrap:anywhere] break-words">Palet Aksen Warna</span>
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5">
      {#each palettes as p}
        <button
          type="button"
          onclick={() => onSelectPalette(p.id)}
          class="flex items-center justify-between p-3 rounded-xl border text-left cursor-pointer transition-all min-h-[44px] touch-manipulation {palette === p.id ? 'border-primary bg-primary/10 shadow-xs' : 'border-border bg-card hover:bg-muted/40'}"
        >
          <div class="flex items-center gap-2.5 min-w-0">
            <span class="h-3.5 w-3.5 rounded-full border border-white/20 shrink-0" style="background-color: {p.primary}"></span>
            <span class="font-medium text-xs text-foreground truncate [overflow-wrap:anywhere]">{p.name}</span>
          </div>
          {#if palette === p.id}
            <Check class="h-4 w-4 text-primary shrink-0 ml-2" />
          {/if}
        </button>
      {/each}
    </div>
  </div>
</div>
