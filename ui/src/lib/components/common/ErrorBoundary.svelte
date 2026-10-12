<script lang="ts">
  import type { Snippet } from "svelte";
  import AlertTriangle from "lucide-svelte/icons/triangle-alert";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
  let hasError = $state(false);
  let errorMessage = $state("");

  export function triggerError(err: Error | string) {
    hasError = true;
    errorMessage = typeof err === "string" ? err : err.message;
  }

  function handleReset() {
    hasError = false;
    errorMessage = "";
    window.location.reload();
  }
</script>

{#if hasError}
  <div class="flex h-screen w-screen flex-col items-center justify-center bg-background p-6 text-foreground font-sans">
    <div class="max-w-md w-full rounded-2xl border border-destructive/40 bg-card p-6 shadow-2xl space-y-4">
      <div class="flex items-center gap-3 text-destructive">
        <AlertTriangle class="h-6 w-6" />
        <h2 class="text-lg font-bold">Terjadi Kesalahan Aplikasi</h2>
      </div>
      <p class="text-sm text-muted-foreground break-words [overflow-wrap:anywhere]">
        {errorMessage || "Terjadi galat tak terduga pada antarmuka. Silakan muat ulang komponen."}
      </p>
      <div class="pt-2 flex justify-end gap-2">
        <Button variant="outline" onclick={handleReset} class="gap-2">
          <RefreshCw class="h-4 w-4" />
          <span>Muat Ulang</span>
        </Button>
      </div>
    </div>
  </div>
{:else}
  {@render children?.()}
{/if}
