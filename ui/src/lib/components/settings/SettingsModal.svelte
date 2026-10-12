<script lang="ts">
  import UserIcon from "lucide-svelte/icons/user";
  import Palette from "lucide-svelte/icons/palette";
  import Server from "lucide-svelte/icons/server";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import { Dialog, DialogOverlay, DialogContent, DialogHeader, DialogTitle, DialogClose } from "$lib/components/ui/dialog";
  import { Tabs, TabsList, TabsTrigger, TabsContent } from "$lib/components/ui/tabs";
  import AccountTab from "./AccountTab.svelte";
  import ThemesTab, { type ThemePalette } from "./ThemesTab.svelte";
  import ProviderKeysTab from "./ProviderKeysTab.svelte";
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

  let activeTab = $state("account");
</script>

<Dialog open={isOpen}>
  <DialogOverlay open={isOpen} onclick={onClose} />
  <DialogContent open={isOpen} {onClose} class="max-w-3xl h-[85vh] max-h-[620px] p-0 overflow-hidden flex flex-col md:flex-row">
    <!-- Settings Sidebar Tabs -->
    <aside class="w-full md:w-56 border-b md:border-b-0 md:border-r border-border bg-muted/20 p-3 space-y-1 shrink-0 flex flex-row md:flex-col justify-between md:justify-start">
      <div class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground px-2 py-1 hidden md:block select-none">
        Pengaturan
      </div>

      <button
        type="button"
        onclick={() => (activeTab = "account")}
        class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors min-h-[44px] md:min-h-0 touch-manipulation {activeTab === 'account' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
      >
        <UserIcon class="h-4 w-4 shrink-0" />
        <span class="truncate">Akun & Profil</span>
      </button>

      <button
        type="button"
        onclick={() => (activeTab = "themes")}
        class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors min-h-[44px] md:min-h-0 touch-manipulation {activeTab === 'themes' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
      >
        <Palette class="h-4 w-4 shrink-0" />
        <span class="truncate">Tema & Warna</span>
      </button>

      <button
        type="button"
        onclick={() => (activeTab = "providers")}
        class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors min-h-[44px] md:min-h-0 touch-manipulation {activeTab === 'providers' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
      >
        <Server class="h-4 w-4 shrink-0" />
        <span class="truncate">Provider & API Key</span>
      </button>

      <button
        type="button"
        onclick={() => (activeTab = "admin")}
        class="flex items-center gap-2 w-full px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors min-h-[44px] md:min-h-0 touch-manipulation {activeTab === 'admin' ? 'bg-cyan-950/60 text-cyan-400 border border-cyan-800/40 shadow-xs' : 'text-muted-foreground hover:bg-muted/40 hover:text-foreground'}"
      >
        <ShieldCheck class="h-4 w-4 shrink-0" />
        <span class="truncate">Administrasi</span>
      </button>
    </aside>

    <!-- Settings Content Area -->
    <main class="flex-1 flex flex-col min-w-0 overflow-hidden bg-card">
      <header class="flex items-center justify-between border-b border-border px-6 py-3.5 shrink-0">
        <DialogTitle class="text-sm font-bold capitalize">
          {activeTab === "account" ? "Pengaturan Akun" : activeTab === "themes" ? "Tema & Skema Warna" : activeTab === "providers" ? "Konfigurasi Provider AI" : "Panel Administrasi"}
        </DialogTitle>
        <DialogClose onclick={onClose} class="relative top-0 right-0" />
      </header>

      <div class="flex-1 overflow-y-auto p-6 smooth-momentum-scroll">
        <Tabs bind:value={activeTab}>
          <TabsContent value="account">
            <AccountTab {user} {onLogout} />
          </TabsContent>
          <TabsContent value="themes">
            <ThemesTab {isDark} {palette} {onToggleDark} {onSelectPalette} />
          </TabsContent>
          <TabsContent value="providers">
            <ProviderKeysTab />
          </TabsContent>
          <TabsContent value="admin">
            <AdminTab />
          </TabsContent>
        </Tabs>
      </div>
    </main>
  </DialogContent>
</Dialog>
