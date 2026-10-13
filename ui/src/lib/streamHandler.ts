import type { Message, Session } from "./types";

export interface StreamState {
  activeSessionId: string | null;
  messages: Message[];
  isGenerating: boolean;
  sessions: Session[];
  errorMessage: string | null;
}

export function processStreamEvent(raw: string, state: StreamState): {
  messages: Message[];
  isGenerating: boolean;
  sessions: Session[];
  errorMessage: string | null;
  shouldScroll: boolean;
} {
  let messages = [...state.messages];
  let isGenerating = state.isGenerating;
  let sessions = [...state.sessions];
  let errorMessage = state.errorMessage;
  let shouldScroll = false;

  try {
    const data = JSON.parse(raw);
    if (data.event === "chunk" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        messages[lastIdx] = {
          ...messages[lastIdx],
          content: messages[lastIdx].content + data.delta,
          model_id: messages[lastIdx].model_id || data.model_id,
        };
      } else {
        messages.push({
          id: "stream_" + Date.now(),
          session_id: state.activeSessionId || "",
          role: "assistant",
          content: data.delta,
          model_id: data.model_id,
          created_at: new Date().toISOString(),
        });
      }
      shouldScroll = true;
      if (data.is_done) isGenerating = false;
    } else if (data.event === "tool_call_start" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      const tc = {
        id: data.call_id,
        tool: data.tool,
        args: data.args,
        isRunning: true,
      };
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        const prevCalls = (messages[lastIdx].metadata?.tool_calls as any[]) || [];
        messages[lastIdx] = {
          ...messages[lastIdx],
          metadata: { ...(messages[lastIdx].metadata || {}), tool_calls: [...prevCalls, tc] },
        };
      } else {
        messages.push({
          id: "stream_" + Date.now(),
          session_id: state.activeSessionId || "",
          role: "assistant",
          content: "",
          metadata: { tool_calls: [tc] },
          created_at: new Date().toISOString(),
        });
      }
      shouldScroll = true;
    } else if (data.event === "tool_call_result" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        const prevCalls = (messages[lastIdx].metadata?.tool_calls as any[]) || [];
        const updated = prevCalls.map((t: any) =>
          t.id === data.call_id || t.tool === data.tool
            ? { ...t, result: data.result, error: data.is_error ? data.result : undefined, isRunning: false }
            : t
        );
        messages[lastIdx] = {
          ...messages[lastIdx],
          metadata: { ...(messages[lastIdx].metadata || {}), tool_calls: updated },
        };
      }
      shouldScroll = true;
    } else if (data.event === "error" && data.session_id === state.activeSessionId) {
      errorMessage = data.error;
      isGenerating = false;
    } else if ((data.event === "title_generated" || data.event === "title") && data.session_id) {
      sessions = sessions.map((s) => (s.id === data.session_id ? { ...s, title: data.title } : s));
    }
  } catch {}

  return { messages, isGenerating, sessions, errorMessage, shouldScroll };
}
