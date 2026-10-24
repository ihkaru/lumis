import type { FrontendToolDefinition, LumisEventEnvelope, LumisWidgetConfig } from "./types";

export interface LumisWidgetInstance {
  open: () => void;
  close: () => void;
  toggle: () => void;
  sendPrompt: (text: string) => void;
  setContext: (context: Record<string, unknown>) => void;
  setTheme: (theme: "dark" | "light") => void;
  registerTool: (tool: FrontendToolDefinition) => void;
  destroy: () => void;
}

export function initLumisWidget(config: LumisWidgetConfig = {}): LumisWidgetInstance {
  if (typeof window === "undefined") {
    return {
      open: () => {},
      close: () => {},
      toggle: () => {},
      sendPrompt: () => {},
      setContext: () => {},
      setTheme: () => {},
      registerTool: () => {},
      destroy: () => {},
    };
  }

  const serverUrl = (config.serverUrl || window.location.origin).replace(/\/+$/, "");
  const position = config.position || "bottom-right";
  const brandTitle = config.title || "Lumis AI";
  const registeredTools = new Map<string, FrontendToolDefinition>();

  let isOpen = config.defaultOpen || false;

  // 1. Container Wrapper
  const wrapper = document.createElement("div");
  wrapper.id = "lumis-widget-root";
  wrapper.style.position = "fixed";
  wrapper.style.bottom = "20px";
  wrapper.style[position === "bottom-left" ? "left" : "right"] = "20px";
  wrapper.style.zIndex = "2147483647";
  wrapper.style.display = "flex";
  wrapper.style.flexDirection = "column";
  wrapper.style.alignItems = position === "bottom-left" ? "flex-start" : "flex-end";

  // 2. Iframe Box
  const iframeContainer = document.createElement("div");
  iframeContainer.style.width = "400px";
  iframeContainer.style.height = "600px";
  iframeContainer.style.maxWidth = "calc(100vw - 40px)";
  iframeContainer.style.maxHeight = "calc(100dvh - 100px)";
  iframeContainer.style.boxShadow = "0 20px 25px -5px rgba(0, 0, 0, 0.2), 0 8px 10px -6px rgba(0, 0, 0, 0.2)";
  iframeContainer.style.borderRadius = "16px";
  iframeContainer.style.overflow = "hidden";
  iframeContainer.style.marginBottom = "12px";
  iframeContainer.style.display = isOpen ? "block" : "none";
  iframeContainer.style.transition = "transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease";
  iframeContainer.style.transform = isOpen ? "scale(1) translate3d(0, 0, 0)" : "scale(0.95) translate3d(0, 10px, 0)";
  iframeContainer.style.opacity = isOpen ? "1" : "0";

  const iframe = document.createElement("iframe");
  const params = new URLSearchParams();
  if (config.embedToken) params.set("token", config.embedToken);
  params.set("title", brandTitle);
  params.set("origin", window.location.origin);
  if (config.theme && config.theme !== "auto") params.set("theme", config.theme);

  iframe.src = `${serverUrl}/embed/chat?${params.toString()}`;
  iframe.style.width = "100%";
  iframe.style.height = "100%";
  iframe.style.border = "none";
  iframe.setAttribute("allow", "clipboard-write");
  iframeContainer.appendChild(iframe);

  // 3. Launcher Button
  const button = document.createElement("button");
  button.type = "button";
  button.setAttribute("aria-label", "Toggle AI Chat");
  button.style.width = "56px";
  button.style.height = "56px";
  button.style.borderRadius = "50%";
  button.style.backgroundColor = "#1a73e8";
  button.style.color = "#ffffff";
  button.style.border = "none";
  button.style.boxShadow = "0 10px 15px -3px rgba(0, 0, 0, 0.2)";
  button.style.cursor = "pointer";
  button.style.display = "flex";
  button.style.alignItems = "center";
  button.style.justifyContent = "center";
  button.style.transition = "transform 0.15s ease, background-color 0.15s ease";
  button.innerHTML = `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/></svg>`;

  function updateOpenState(open: boolean) {
    isOpen = open;
    if (isOpen) {
      iframeContainer.style.display = "block";
      requestAnimationFrame(() => {
        iframeContainer.style.transform = "scale(1) translate3d(0, 0, 0)";
        iframeContainer.style.opacity = "1";
      });
      button.innerHTML = `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>`;
    } else {
      iframeContainer.style.transform = "scale(0.95) translate3d(0, 10px, 0)";
      iframeContainer.style.opacity = "0";
      setTimeout(() => {
        if (!isOpen) iframeContainer.style.display = "none";
      }, 200);
      button.innerHTML = `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/></svg>`;
    }
  }

  button.addEventListener("click", () => updateOpenState(!isOpen));

  wrapper.appendChild(iframeContainer);
  wrapper.appendChild(button);
  document.body.appendChild(wrapper);

  // 4. PostMessage Handshake (RFC 6454 safe)
  function handleMessage(event: MessageEvent) {
    if (new URL(serverUrl).origin !== event.origin) return;
    const msg = event.data as LumisEventEnvelope<any>;
    if (!msg || msg.source !== "lumis-widget") return;

    if (msg.event === "lumis:ready") {
      iframe.contentWindow?.postMessage(
        {
          source: "host-app",
          event: "lumis:init",
          payload: {
            token: config.embedToken,
            context: config.contextData,
          },
        },
        serverUrl
      );
    }
  }

  window.addEventListener("message", handleMessage);

  return {
    open: () => updateOpenState(true),
    close: () => updateOpenState(false),
    toggle: () => updateOpenState(!isOpen),
    sendPrompt: (text: string) => {
      iframe.contentWindow?.postMessage(
        {
          source: "host-app",
          event: "lumis:send_prompt",
          payload: { prompt: text },
        },
        serverUrl
      );
    },
    setContext: (context: Record<string, unknown>) => {
      iframe.contentWindow?.postMessage(
        {
          source: "host-app",
          event: "lumis:set_context",
          payload: context,
        },
        serverUrl
      );
    },
    setTheme: (theme: "dark" | "light") => {
      iframe.contentWindow?.postMessage(
        {
          source: "host-app",
          event: "lumis:set_theme",
          payload: { theme },
        },
        serverUrl
      );
    },
    registerTool: (tool: FrontendToolDefinition) => {
      registeredTools.set(tool.name, tool);
    },
    destroy: () => {
      window.removeEventListener("message", handleMessage);
      wrapper.remove();
    },
  };
}

if (typeof window !== "undefined") {
  (window as any).Lumis = {
    init: initLumisWidget,
  };
}
