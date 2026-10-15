<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import Menu from "lucide-svelte/icons/menu";
  import FileEdit from "lucide-svelte/icons/file-edit";

  interface Props {
    title: string;
    selectedModelId?: string;
    isMobileDrawerOpen: boolean;
    onToggleMobileDrawer: () => void;
    onOpenScratchpad: () => void;
  }

  let {
    title,
    selectedModelId,
    isMobileDrawerOpen,
    onToggleMobileDrawer,
    onOpenScratchpad
  }: Props = $props();
</script>

<header class="h-16 border-b border-border px-4 md:px-6 flex items-center justify-between shrink-0 bg-background/80 backdrop-blur-md select-none font-sans">
  <div class="flex items-center gap-3 min-w-0">
    <Button
      variant="ghost"
      size="icon"
      onclick={onToggleMobileDrawer}
      class="{isMobileDrawerOpen ? 'md:hidden' : ''} min-h-[44px] min-w-[44px] sm:min-h-9 sm:min-w-9 text-muted-foreground hover:text-foreground"
      aria-label="Buka Menu"
    >
      <Menu class="w-5 h-5" />
    </Button>
    <div class="font-medium text-foreground truncate max-w-xs md:max-w-md">
      {title}
    </div>
  </div>
  <div class="flex items-center gap-2">
    <Button
      variant="ghost"
      size="sm"
      onclick={onOpenScratchpad}
      class="gap-1.5 text-xs text-muted-foreground hover:text-foreground hidden sm:flex"
      title="Buka Markdown Scratchpad"
    >
      <FileEdit class="w-3.5 h-3.5 text-primary" />
      <span>Scratchpad</span>
    </Button>
  </div>
</header>
