<script lang="ts">
  import X from "lucide-svelte/icons/x";
  import UserIcon from "lucide-svelte/icons/user";
  import Palette from "lucide-svelte/icons/palette";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import AccountTab from "./AccountTab.svelte";
  import ThemesTab, { type ThemePalette } from "./ThemesTab.svelte";
  import AdminTab from "./AdminTab.svelte";
  import type { User } from "$lib/types";

  interface Props {
    isOpen: boolean;
    user: User | null;
    isDark: boolean;
    palette: ThemePalette;
    onClose: () => void;
    onLogout: () => void;
    onToggleDark: (dark: boolean) => void;
    onSelectPalette: (palette: ThemePalette) => void;
  }

  let {
    isOpen = false,
    user,
    isDark = true,
    palette = "llumen",
    onClose,
    onLogout,
    onToggleDark,
    onSelectPalette,
  }: Props = $props();

  let activeTab = $state<"account" | "themes" | "admin">("account");
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4 font-sans select-none">
    <div class="flex h-[85vh] max-h-[600px] w-full max-w-3xl flex-col md:flex-row overflow-hidden rounded-2xl border border-border bg-card shadow-2xl">
      <!-- Settings Sidebar Tabs -->
      <aside class="w-full md:w-56 border-b md:border-b-0 md:border-r border-border bg-muted/20 p-3 space-y-1 shrink-0 flex flex-row md:flex-col justify-between md:justify-start">
        <div class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground px-2 py-1 hidden md:block">
          Pengaturan
        </div>

        <button
          type="button"
          onclick={() => (activeTab = "account")}
          class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors {activeTab === 'account' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
        >
          <UserIcon class="h-4 w-4" />
          <span>Akun & Profil</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "themes")}
          class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors {activeTab === 'themes' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
        >
          <Palette class="h-4 w-4" />
          <span>Tema & Tampilan</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "admin")}
          class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors {activeTab === 'admin' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
        >
          <ShieldCheck class="h-4 w-4" />
          <span>Administrasi</span>
        </button>
      </aside>

      <!-- Settings Content Area -->
      <main class="flex-1 flex flex-col min-w-0 overflow-hidden">
        <header class="flex items-center justify-between border-b border-border px-6 py-3.5 shrink-0 bg-card">
          <h2 class="font-bold text-sm text-foreground capitalize">
            {activeTab === "account" ? "Pengaturan Akun" : activeTab === "themes" ? "Tema & Skema Warna" : "Panel Administrasi"}
          </h2>
          <button
            type="button"
            onclick={onClose}
            class="rounded-lg p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer"
            aria-label="Tutup Pengaturan"
          >
            <X class="h-4 w-4" />
          </button>
        </header>

        <div class="flex-1 overflow-y-auto p-6 smooth-momentum-scroll">
          {#if activeTab === "account"}
            <AccountTab {user} {onLogout} />
          {:else if activeTab === "themes"}
            <ThemesTab {isDark} {palette} {onToggleDark} {onSelectPalette} />
          {:else if activeTab === "admin"}
            <AdminTab />
          {/if}
        </div>
      </main>
    </div>
  </div>
{/if}
