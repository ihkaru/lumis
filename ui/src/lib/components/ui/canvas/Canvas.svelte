<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { cn } from "$lib/utils";
  import { api } from "$lib/api";
  import type { ArtifactSpecification } from "$lib/types";
  import Button from "$lib/components/ui/button/Button.svelte";
  import Badge from "$lib/components/ui/badge/Badge.svelte";
  import Separator from "$lib/components/ui/separator/Separator.svelte";

  import Code from "lucide-svelte/icons/code";
  import Eye from "lucide-svelte/icons/eye";
  import Copy from "lucide-svelte/icons/copy";
  import Check from "lucide-svelte/icons/check";
  import Download from "lucide-svelte/icons/download";
  import X from "lucide-svelte/icons/x";
  import Zap from "lucide-svelte/icons/zap";

  interface Props {
    artifact: ArtifactSpecification | null;
    sessionId?: string;
    onClose?: () => void;
    class?: string;
  }

  let {
    artifact,
    sessionId,
    onClose,
    class: className,
  }: Props = $props();

  let activeTab: "preview" | "code" = $state("preview");
  let copied = $state(false);
  let iframeEl: HTMLIFrameElement | null = $state(null);
  let ipcStatus: "idle" | "connected" | "error" = $state("idle");

  async function handleCopy() {
    if (!artifact?.content) return;
    try {
      await navigator.clipboard.writeText(artifact.content);
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      copied = false;
    }
  }

  function handleDownload() {
    if (!artifact) return;
    const ext = artifact.artifact_type.includes("html")
      ? "html"
      : artifact.artifact_type.includes("python")
        ? "py"
        : artifact.artifact_type.includes("svg")
          ? "svg"
          : artifact.artifact_type.includes("json")
            ? "json"
            : "txt";
    const blob = new Blob([artifact.content], { type: artifact.artifact_type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${artifact.identifier}.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  async function onIpcMessage(event: MessageEvent) {
    if (!event.data || event.data.type !== "lumis:ipc" || !artifact) return;
    const { request } = event.data;
    if (!request || !request.request_id) return;

    try {
      const response = await api.sendArtifactIpc(artifact.identifier, request, sessionId);
      ipcStatus = response.success ? "connected" : "error";
      if (iframeEl?.contentWindow) {
        iframeEl.contentWindow.postMessage(
          { type: "lumis:ipc_response", response },
          "*"
        );
      }
    } catch (err: unknown) {
      ipcStatus = "error";
      if (iframeEl?.contentWindow) {
        const errorMsg = err instanceof Error ? err.message : "IPC request failed";
        iframeEl.contentWindow.postMessage(
          {
            type: "lumis:ipc_response",
            response: {
              request_id: request.request_id,
              success: false,
              error: errorMsg,
            },
          },
          "*"
        );
      }
    }
  }

  onMount(() => {
    window.addEventListener("message", onIpcMessage);
  });

  onDestroy(() => {
    window.removeEventListener("message", onIpcMessage);
  });
</script>

{#if artifact}
  <aside
    class={cn(
      "flex flex-col h-full w-full bg-card border-l border-border transition-all duration-150",
      className
    )}
    aria-label="Interactive Artifact Canvas"
  >
    <!-- Canvas Header Bar -->
    <header class="flex items-center justify-between px-4 py-3 border-b border-border bg-card/50">
      <div class="flex items-center gap-2 min-w-0">
        <Zap class="w-4 h-4 text-primary shrink-0" />
        <h3 class="font-sans font-medium text-sm text-foreground truncate max-w-[200px] sm:max-w-[320px]">
          {artifact.title}
        </h3>
        <Badge variant="outline" class="font-mono text-[10px] uppercase shrink-0">
          {artifact.artifact_type.split("/")[1] || artifact.artifact_type}
        </Badge>
        {#if ipcStatus === "connected"}
          <Badge variant="default" class="bg-primary/20 text-primary text-[10px] shrink-0">
            IPC Live
          </Badge>
        {/if}
      </div>

      <div class="flex items-center gap-1.5 shrink-0">
        <!-- Tab Switcher -->
        <div class="flex items-center bg-muted/60 p-0.5 rounded-lg border border-border">
          <Button
            variant={activeTab === "preview" ? "secondary" : "ghost"}
            size="sm"
            class="h-7 px-2.5 text-xs font-sans gap-1"
            onclick={() => (activeTab = "preview")}
          >
            <Eye class="w-3.5 h-3.5" />
            <span class="hidden sm:inline">Preview</span>
          </Button>
          <Button
            variant={activeTab === "code" ? "secondary" : "ghost"}
            size="sm"
            class="h-7 px-2.5 text-xs font-sans gap-1"
            onclick={() => (activeTab = "code")}
          >
            <Code class="w-3.5 h-3.5" />
            <span class="hidden sm:inline">Code</span>
          </Button>
        </div>

        <Separator orientation="vertical" class="h-4 mx-1" />

        <!-- Copy Action -->
        <Button
          variant="outline"
          size="sm"
          class="h-7 px-2 text-xs gap-1 font-sans"
          onclick={handleCopy}
          title="Copy artifact source code"
        >
          {#if copied}
            <Check class="w-3.5 h-3.5 text-green-500" />
            <span class="text-green-500 hidden sm:inline">Copied</span>
          {:else}
            <Copy class="w-3.5 h-3.5" />
            <span class="hidden sm:inline">Copy</span>
          {/if}
        </Button>

        <!-- Download Action -->
        <Button
          variant="outline"
          size="sm"
          class="h-7 px-2 text-xs font-sans"
          onclick={handleDownload}
          title="Download artifact file"
        >
          <Download class="w-3.5 h-3.5" />
        </Button>

        {#if onClose}
          <Button
            variant="ghost"
            size="sm"
            class="h-7 w-7 p-0"
            onclick={onClose}
            title="Close Canvas"
          >
            <X class="w-4 h-4" />
          </Button>
        {/if}
      </div>
    </header>

    <!-- Canvas Content Pane -->
    <main class="flex-1 min-h-0 relative bg-background overflow-hidden">
      {#if activeTab === "preview"}
        <!-- Sandboxed Zero-Bleed Iframe View -->
        <iframe
          bind:this={iframeEl}
          srcdoc={artifact.content}
          sandbox="allow-scripts"
          title={artifact.title}
          class="w-full h-full border-0 bg-background"
        ></iframe>
      {:else}
        <!-- Syntax/Code View -->
        <div class="h-full w-full overflow-auto p-4 bg-secondary/30">
          <pre class="font-mono text-xs leading-relaxed text-foreground whitespace-pre-wrap break-words [overflow-wrap:anywhere]"><code>{artifact.content}</code></pre>
        </div>
      {/if}
    </main>
  </aside>
{/if}
