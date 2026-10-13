<script lang="ts">
  import KeyRound from "lucide-svelte/icons/key-round";
  import Loader2 from "lucide-svelte/icons/loader-2";
  import X from "lucide-svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { mcpApi } from "$lib/mcpApi";
  import type { McpUserPreferenceItem } from "$lib/types";

  interface Props {
    server: McpUserPreferenceItem | null;
    onClose: () => void;
    onConnected: () => void;
  }

  let { server, onClose, onConnected }: Props = $props();

  let tokenInput = $state("");
  let isConnecting = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleSubmit() {
    if (!server || !tokenInput.trim()) return;
    isConnecting = true;
    errorMessage = null;
    try {
      await mcpApi.callbackOAuth(server.id, {
        access_token: tokenInput.trim(),
        expires_in: 86400,
      });
      tokenInput = "";
      onConnected();
      onClose();
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : "Gagal menyimpan token OAuth";
    } finally {
      isConnecting = false;
    }
  }
</script>

{#if server}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-xs font-sans text-xs">
    <div class="bg-card border border-border rounded-2xl w-full max-w-md p-5 space-y-4 shadow-2xl">
      <div class="flex items-center justify-between">
        <div class="font-semibold text-sm text-foreground flex items-center gap-2">
          <KeyRound class="h-4 w-4 text-sky-400" />
          <span>Koneksi OAuth: {server.display_title}</span>
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

      <div class="text-[11px] text-muted-foreground">
        Jendela otorisasi telah dibuka. Masukkan token akses atau authorization code dari IdP jika tidak terisi otomatis:
      </div>
      <div>
        <input type="password" bind:value={tokenInput} placeholder="Tempel Access Token di sini..." class="w-full h-9 px-3 rounded-lg border border-border bg-background text-xs font-mono" />
      </div>
      <div class="flex justify-end gap-2 pt-2 border-t border-border">
        <Button variant="ghost" size="sm" onclick={onClose}>Batal</Button>
        <Button size="sm" onclick={handleSubmit} disabled={isConnecting || !tokenInput.trim()} class="bg-sky-600 hover:bg-sky-500 text-white">
          {#if isConnecting}<Loader2 class="h-3.5 w-3.5 animate-spin mr-1.5" />{/if}
          <span>Simpan Token & Hubungkan</span>
        </Button>
      </div>
    </div>
  </div>
{/if}
