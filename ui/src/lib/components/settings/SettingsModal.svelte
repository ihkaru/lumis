<script lang="ts">
  import UserIcon from "lucide-svelte/icons/user";
  import Palette from "lucide-svelte/icons/palette";
  import Server from "lucide-svelte/icons/server";
  import ShieldCheck from "lucide-svelte/icons/shield-check";
  import Brain from "lucide-svelte/icons/brain";
  import { Dialog, DialogOverlay, DialogContent, DialogTitle, DialogClose } from "$lib/components/ui/dialog";
  import { Tabs, TabsContent } from "$lib/components/ui/tabs";
  import AccountTab from "./AccountTab.svelte";
  import ThemesTab, { type ThemePalette } from "./ThemesTab.svelte";
  import ProviderKeysTab from "./ProviderKeysTab.svelte";
  import MemoriesTab from "./MemoriesTab.svelte";
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

  const tabList = [
    { id: "account", label: "Akun & Profil", icon: UserIcon },
    { id: "themes", label: "Tema & Warna", icon: Palette },
    { id: "memories", label: "Memori AI", icon: Brain },
    { id: "providers", label: "Provider AI", icon: Server },
    { id: "admin", label: "Administrasi", icon: ShieldCheck },
  ];
</script>

<Dialog open={isOpen}>
  <DialogOverlay open={isOpen} onclick={onClose} />
  <DialogContent
    open={isOpen}
    {onClose}
    class="w-[95vw] sm:w-[92vw] md:max-w-3xl lg:max-w-4xl h-[88vh] max-h-[660px] p-0 overflow-hidden flex flex-col md:flex-row rounded-2xl border border-border bg-card shadow-2xl"
  >
    <!-- Navigation Tabs: Horizontal bar on Mobile (< md), Vertical sidebar on Desktop (>= md) -->
    <aside class="w-full md:w-56 lg:w-60 border-b md:border-b-0 md:border-r border-border bg-muted/20 p-2 md:p-3 shrink-0 flex flex-row md:flex-col gap-1.5 overflow-x-auto md:overflow-x-visible no-scrollbar">
      <div class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground px-2 py-1.5 hidden md:block select-none">
        Pengaturan
      </div>

      {#each tabList as tab}
        {@const Icon = tab.icon}
        {@const isActive = activeTab === tab.id}
        <button
          type="button"
          onclick={() => (activeTab = tab.id)}
          class="flex items-center gap-2 px-3 py-2 rounded-xl text-xs font-medium cursor-pointer transition-colors shrink-0 md:shrink md:w-full min-h-[44px] md:min-h-9 touch-manipulation {isActive ? 'bg-accent text-accent-foreground font-semibold shadow-xs border border-border/80' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'}"
        >
          <Icon class="h-4 w-4 shrink-0" />
          <span class="whitespace-nowrap md:whitespace-normal md:break-words text-left [overflow-wrap:anywhere]">
            {tab.label}
          </span>
        </button>
      {/each}
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-w-0 overflow-hidden bg-card">
      <header class="flex items-center justify-between border-b border-border px-4 sm:px-6 py-3.5 shrink-0 bg-card">
        <DialogTitle class="text-sm font-semibold capitalize truncate pr-6">
          {tabList.find((t) => t.id === activeTab)?.label || "Pengaturan"}
        </DialogTitle>
        <DialogClose onclick={onClose} class="relative top-0 right-0" />
      </header>

      <div class="flex-1 overflow-y-auto p-4 sm:p-6 smooth-momentum-scroll min-w-0">
        <Tabs bind:value={activeTab}>
          <TabsContent value="account">
            <AccountTab {user} {onLogout} />
          </TabsContent>
          <TabsContent value="themes">
            <ThemesTab {isDark} {palette} {onToggleDark} {onSelectPalette} />
          </TabsContent>
          <TabsContent value="memories">
            <MemoriesTab />
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
