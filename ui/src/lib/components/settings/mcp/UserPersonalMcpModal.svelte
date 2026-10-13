<script lang="ts">
  import Server from "lucide-svelte/icons/server";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { mcpApi } from "$lib/mcpApi";

  interface Props {
    open: boolean;
    onClose: () => void;
    onCreated: () => void;
  }

  let { open, onClose, onCreated }: Props = $props();

  let name = $state("");
  let title = $state("");
  let url = $state("");
  let transport = $state<"sse" | "streamable_http">("sse");
  let isSaving = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleSave() {
    if (!name.trim() || !title.trim() || !url.trim()) {
      errorMessage = "Semua field server wajib diisi";
      return;
    }
    isSaving = true;
    errorMessage = null;
    try {
      await mcpApi.createUserServer({
        name: name.trim(),
        display_title: title.trim(),
        endpoint_url: url.trim(),
        transport_type: transport,
        auth_type: "none",
      });
      name = "";
      title = "";
      url = "";
      onCreated();
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal mendaftarkan server pribadi";
    } finally {
      isSaving = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-xs font-sans text-xs">
    <div class="bg-card border border-border rounded-2xl w-full max-w-md p-5 space-y-4 shadow-2xl">
      <div class="flex items-center justify-between">
        <div class="font-semibold text-sm text-foreground flex items-center gap-2">
          <Server class="h-4 w-4 text-primary" />
          <span>Tambah Server MCP Pribadi</span>
        </div>
        <button type="button" onclick={onClose} class="text-muted-foreground hover:text-foreground p-1 rounded-md" aria-label="Tutup modal">
          <X class="h-4 w-4" />
        </button>
      </div>

      {#if errorMessage}
        <div class="p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-[11px]">
          {errorMessage}
        </div>
      {/if}

      <div class="space-y-3">
        <div>
          <label for="usr-srv-name" class="block text-[11px] font-medium text-foreground mb-1">Slug Identifier</label>
          <input id="usr-srv-name" type="text" bind:value={name} placeholder="misal: my_local_server" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
        </div>
        <div>
          <label for="usr-srv-title" class="block text-[11px] font-medium text-foreground mb-1">Judul Tampilan</label>
          <input id="usr-srv-title" type="text" bind:value={title} placeholder="misal: Server MCP Pribadi" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs" />
        </div>
        <div>
          <label for="usr-srv-url" class="block text-[11px] font-medium text-foreground mb-1">Endpoint URL (Anti-SSRF Protected)</label>
          <input id="usr-srv-url" type="url" bind:value={url} placeholder="https://my-mcp.company.org/sse" class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs font-mono" />
        </div>
      </div>
      <div class="flex justify-end gap-2 pt-2 border-t border-border">
        <Button variant="ghost" size="sm" onclick={onClose}>Batal</Button>
        <Button size="sm" onclick={handleSave} disabled={isSaving}>
          {#if isSaving}<Loader2 class="h-3.5 w-3.5 animate-spin mr-1.5" />{/if}
          <span>Simpan</span>
        </Button>
      </div>
    </div>
  </div>
{/if}
