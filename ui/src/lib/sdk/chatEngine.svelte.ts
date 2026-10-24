import type { Message, Session, SessionTask } from "../types";
import type { ChatMode } from "../components/input/ModeSelector.svelte";
import type { AttachedFile } from "../components/input/FileDropzone.svelte";
import { processStreamEvent } from "../streamHandler";
import type { ChatEngineOptions, FrontendToolDefinition, ILumisTransport } from "./types";
import { defaultTransport } from "./transport";

export function createLumisChat(
  transport: ILumisTransport = defaultTransport,
  options: ChatEngineOptions = {}
) {
  let messages = $state<Message[]>([]);
  let isGenerating = $state(false);
  let activeSessionId = $state<string | null>(options.initialSessionId || null);
  let sessions = $state<Session[]>([]);
  let sessionTasks = $state<SessionTask[]>([]);
  let selectedModelId = $state<string>(options.defaultModelId || "my9model-free");
  let chatMode = $state<ChatMode>("normal");
  let errorMessage = $state<string | null>(null);
  let contextData = $state<Record<string, unknown>>(options.contextData || {});
  const frontendTools = new Map<string, FrontendToolDefinition>();

  let activeAbortController: AbortController | null = null;
  let eventSource: EventSource | (() => void) | null = null;

  const activeSession = $derived(sessions.find((s) => s.id === activeSessionId) || null);

  function setContextData(data: Record<string, unknown>) {
    contextData = { ...contextData, ...data };
  }

  function registerFrontendTool(tool: FrontendToolDefinition) {
    frontendTools.set(tool.name, tool);
  }

  function handleStreamEvent(raw: string): { shouldScroll: boolean } {
    const updated = processStreamEvent(raw, {
      activeSessionId,
      messages,
      isGenerating,
      sessions,
      errorMessage,
      tasks: sessionTasks,
    });

    messages = updated.messages;
    isGenerating = updated.isGenerating;
    sessions = updated.sessions;
    errorMessage = updated.errorMessage;
    sessionTasks = updated.tasks;

    try {
      const data = JSON.parse(raw);
      if (data.event === "tool_call_start" && data.tool && frontendTools.has(data.tool)) {
        const tool = frontendTools.get(data.tool)!;
        Promise.resolve(tool.execute(data.args || {})).then((result) => {
          options.onToolCallRequested?.(tool.name, data.args || {});
        });
      }
    } catch {}

    return { shouldScroll: updated.shouldScroll };
  }

  function initStream() {
    if (eventSource) return;
    eventSource = transport.connectStream((raw) => {
      handleStreamEvent(raw);
    });
  }

  function disconnectStream() {
    if (eventSource) {
      if (typeof eventSource === "function") {
        eventSource();
      } else if ("close" in eventSource) {
        eventSource.close();
      }
      eventSource = null;
    }
  }

  async function loadSessions(): Promise<Session[]> {
    try {
      const page = await transport.listSessions();
      sessions = page.items;
      return sessions;
    } catch (e: unknown) {
      errorMessage = e instanceof Error ? e.message : "Gagal memuat sesi percakapan.";
      return [];
    }
  }

  async function selectSession(id: string): Promise<void> {
    if (isGenerating && activeSessionId === id) return;
    activeSessionId = id;
    errorMessage = null;

    const current = sessions.find((s) => s.id === id);
    if (current?.model_id) selectedModelId = current.model_id;

    try {
      const [page, tasks] = await Promise.all([
        transport.listMessages(id),
        transport.listSessionTasks(id).catch(() => []),
      ]);
      messages = page.items.reverse();
      sessionTasks = tasks;
    } catch {
      messages = [];
      sessionTasks = [];
    }
  }

  async function createSession(customTitle?: string, modelId?: string): Promise<Session | null> {
    try {
      const title = customTitle || "Percakapan Baru";
      const targetModel = modelId || selectedModelId;
      const newSess = await transport.createSession(title, targetModel);
      activeSessionId = newSess.id;
      sessions = [newSess, ...sessions];
      messages = [];
      sessionTasks = [];
      errorMessage = null;
      return newSess;
    } catch (e: unknown) {
      errorMessage = e instanceof Error ? e.message : "Gagal membuat sesi baru.";
      return null;
    }
  }

  async function sendMessage(
    text: string,
    files: AttachedFile[] = [],
    mode: ChatMode = chatMode,
    modelId: string = selectedModelId
  ): Promise<void> {
    if (isGenerating) return;
    if (!text.trim() && files.length === 0) return;

    isGenerating = true;
    errorMessage = null;
    activeAbortController = new AbortController();

    let targetSessionId = activeSessionId;
    if (!targetSessionId || targetSessionId === "new") {
      const newSess = await createSession(text.slice(0, 30), modelId);
      if (!newSess) {
        isGenerating = false;
        activeAbortController = null;
        return;
      }
      targetSessionId = newSess.id;
    }

    const metadata: Record<string, unknown> = {
      ...(Object.keys(contextData).length > 0 ? { host_context: contextData } : {}),
      ...(mode !== "normal" ? { chat_mode: mode } : {}),
    };

    const localUserMsg: Message = {
      id: "temp_" + Date.now(),
      session_id: targetSessionId,
      role: "user",
      content: text,
      created_at: new Date().toISOString(),
      metadata,
    };

    const localAssistantMsg: Message = {
      id: "stream_" + Date.now(),
      session_id: targetSessionId,
      role: "assistant",
      content: "",
      model_id: modelId,
      created_at: new Date().toISOString(),
    };

    messages = [...messages, localUserMsg, localAssistantMsg];

    try {
      await transport.sendMessage(targetSessionId, text, modelId, metadata);
    } catch (e: unknown) {
      const err = e as { name?: string; message?: string };
      if (err?.name !== "AbortError") {
        errorMessage = err?.message || "Gagal mengirimkan pesan.";
      }
      isGenerating = false;
    } finally {
      activeAbortController = null;
    }
  }

  function stop(): void {
    activeAbortController?.abort();
    activeAbortController = null;
    isGenerating = false;
  }

  function retryLastMessage(modelOverride?: string): void {
    const lastUser = [...messages].reverse().find((m) => m.role === "user");
    if (lastUser) {
      sendMessage(lastUser.content, [], chatMode, modelOverride || selectedModelId);
    }
  }

  function renameSession(id: string, newTitle: string): void {
    sessions = sessions.map((s) => (s.id === id ? { ...s, title: newTitle } : s));
  }

  function deleteSession(id: string): void {
    sessions = sessions.filter((s) => s.id !== id);
    if (activeSessionId === id) {
      activeSessionId = null;
      messages = [];
      sessionTasks = [];
    }
  }

  function togglePinSession(target: Session): void {
    sessions = sessions.map((s) => (s.id === target.id ? { ...s, is_pinned: !s.is_pinned } : s));
  }

  return {
    get messages() { return messages; },
    set messages(val) { messages = val; },
    get isGenerating() { return isGenerating; },
    get activeSessionId() { return activeSessionId; },
    set activeSessionId(val) { activeSessionId = val; },
    get activeSession() { return activeSession; },
    get sessions() { return sessions; },
    set sessions(val) { sessions = val; },
    get sessionTasks() { return sessionTasks; },
    get selectedModelId() { return selectedModelId; },
    set selectedModelId(val) { selectedModelId = val; },
    get chatMode() { return chatMode; },
    set chatMode(val) { chatMode = val; },
    get errorMessage() { return errorMessage; },
    set errorMessage(val) { errorMessage = val; },
    get contextData() { return contextData; },
    loadSessions,
    selectSession,
    createSession,
    sendMessage,
    stop,
    retryLastMessage,
    renameSession,
    deleteSession,
    togglePinSession,
    registerFrontendTool,
    setContextData,
    handleStreamEvent,
    initStream,
    disconnectStream,
  };
}

export type LumisChatEngine = ReturnType<typeof createLumisChat>;
