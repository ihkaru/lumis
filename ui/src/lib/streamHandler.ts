import type { Message, MessagePart, Session, SessionTask } from "./types";

export interface StreamState {
  activeSessionId: string | null;
  messages: Message[];
  isGenerating: boolean;
  sessions: Session[];
  errorMessage: string | null;
  tasks?: SessionTask[];
}

export function processStreamEvent(raw: string, state: StreamState): {
  messages: Message[];
  isGenerating: boolean;
  sessions: Session[];
  errorMessage: string | null;
  tasks: SessionTask[];
  shouldScroll: boolean;
} {
  let messages = [...state.messages];
  let isGenerating = state.isGenerating;
  let sessions = [...state.sessions];
  let errorMessage = state.errorMessage;
  let tasks = [...(state.tasks || [])];
  let shouldScroll = false;

  try {
    const data = JSON.parse(raw);
    if (data.event === "chunk" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        const parts = [...((messages[lastIdx].metadata?.parts as MessagePart[]) || [])];
        if (data.delta) {
          const lastPart = parts[parts.length - 1];
          if (lastPart && lastPart.type === "text") {
            parts[parts.length - 1] = { ...lastPart, text: lastPart.text + data.delta };
          } else {
            parts.push({ type: "text", text: data.delta });
          }
        }
        const meta = { ...(messages[lastIdx].metadata || {}), parts };
        if (data.delta) {
          delete (meta as any).status_label;
        }
        messages[lastIdx] = {
          ...messages[lastIdx],
          content: messages[lastIdx].content + data.delta,
          model_id: messages[lastIdx].model_id || data.model_id,
          metadata: meta,
        };
      } else {
        const parts: MessagePart[] = data.delta ? [{ type: "text", text: data.delta }] : [];
        messages.push({
          id: "stream_" + Date.now(),
          session_id: state.activeSessionId || "",
          role: "assistant",
          content: data.delta,
          model_id: data.model_id,
          metadata: { parts },
          created_at: new Date().toISOString(),
        });
      }
      shouldScroll = true;
      if (data.is_done) isGenerating = false;
    } else if (data.event === "status" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        messages[lastIdx] = {
          ...messages[lastIdx],
          model_id: data.model_id || messages[lastIdx].model_id,
          metadata: {
            ...(messages[lastIdx].metadata || {}),
            status_label: data.message,
          },
        };
      }
      shouldScroll = true;
    } else if (data.event === "model_failover" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        messages[lastIdx] = {
          ...messages[lastIdx],
          model_id: data.to_model,
          metadata: {
            ...(messages[lastIdx].metadata || {}),
            status_label: `Dialihkan ke ${data.to_model_name || data.to_model}...`,
            failover: {
              from: data.from_model,
              from_name: data.from_model_name,
              to: data.to_model,
              to_name: data.to_model_name,
              reason: data.reason,
            },
          },
        };
      }
      shouldScroll = true;
    } else if (data.event === "tool_call_start" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      const tc = {
        id: data.call_id,
        tool: data.tool,
        args: data.args,
        isRunning: true,
      };
      const toolPart: MessagePart = {
        type: "tool_call",
        call_id: data.call_id,
        tool: data.tool,
        args: data.args,
        isRunning: true,
      };
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        const prevCalls = (messages[lastIdx].metadata?.tool_calls as any[]) || [];
        const prevParts = [...((messages[lastIdx].metadata?.parts as MessagePart[]) || [])];
        prevParts.push(toolPart);
        messages[lastIdx] = {
          ...messages[lastIdx],
          metadata: {
            ...(messages[lastIdx].metadata || {}),
            tool_calls: [...prevCalls, tc],
            parts: prevParts,
          },
        };
      } else {
        messages.push({
          id: "stream_" + Date.now(),
          session_id: state.activeSessionId || "",
          role: "assistant",
          content: "",
          metadata: { tool_calls: [tc], parts: [toolPart] },
          created_at: new Date().toISOString(),
        });
      }
      shouldScroll = true;
    } else if (data.event === "tool_call_result" && data.session_id === state.activeSessionId) {
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        const prevCalls = (messages[lastIdx].metadata?.tool_calls as any[]) || [];
        const matchCall = (id?: string, tool?: string) => (id && data.call_id ? id === data.call_id : tool === data.tool);
        const updatedCalls = prevCalls.map((t: any) =>
          matchCall(t.id, t.tool)
            ? { ...t, result: data.result, error: data.is_error ? data.result : undefined, isRunning: false }
            : t
        );
        const prevParts = [...((messages[lastIdx].metadata?.parts as MessagePart[]) || [])];
        const updatedParts = prevParts.map((p) => {
          if (p.type === "tool_call" && matchCall(p.call_id, p.tool)) {
            return {
              ...p,
              result: data.result,
              error: data.is_error ? data.result : undefined,
              isRunning: false,
            };
          }
          return p;
        });
        messages[lastIdx] = {
          ...messages[lastIdx],
          metadata: {
            ...(messages[lastIdx].metadata || {}),
            tool_calls: updatedCalls,
            parts: updatedParts,
          },
        };
      }
      shouldScroll = true;
    } else if (data.event === "task_created" && data.session_id === state.activeSessionId && data.task) {
      const existingIdx = tasks.findIndex((t) => t.id === data.task.id);
      if (existingIdx >= 0) {
        tasks[existingIdx] = data.task;
      } else {
        tasks.push(data.task);
      }
      tasks.sort((a, b) => a.order_index - b.order_index);
      shouldScroll = true;
    } else if (data.event === "task_updated" && data.session_id === state.activeSessionId && data.task) {
      tasks = tasks.map((t) => (t.id === data.task.id ? data.task : t));
      tasks.sort((a, b) => a.order_index - b.order_index);
      shouldScroll = true;
    } else if (data.event === "task_deleted" && data.session_id === state.activeSessionId && data.task_id) {
      tasks = tasks.filter((t) => t.id !== data.task_id);
    } else if (data.event === "error" && data.session_id === state.activeSessionId) {
      errorMessage = data.error;
      isGenerating = false;
      const lastIdx = messages.length - 1;
      if (lastIdx >= 0 && messages[lastIdx].role === "assistant") {
        messages[lastIdx] = {
          ...messages[lastIdx],
          metadata: {
            ...(messages[lastIdx].metadata || {}),
            error_message: data.error,
            can_retry: true,
            suggested_model: data.suggested_model,
          },
        };
      }
    } else if ((data.event === "title_generated" || data.event === "title") && data.session_id) {
      sessions = sessions.map((s) => (s.id === data.session_id ? { ...s, title: data.title } : s));
    }
  } catch {}

  return { messages, isGenerating, sessions, errorMessage, tasks, shouldScroll };
}
