import { getContext, setContext } from "svelte";
import type { LumisChatEngine } from "./chatEngine.svelte";

const LUMIS_CHAT_CONTEXT_KEY = Symbol("lumis-chat-engine");

export function setLumisChatContext(engine: LumisChatEngine): LumisChatEngine {
  return setContext(Lumis_CHAT_CONTEXT_KEY, engine);
}

export function getLumisChatContext(): LumisChatEngine | undefined {
  return getContext<LumisChatEngine | undefined>(Lumis_CHAT_CONTEXT_KEY);
}

const Lumis_CHAT_CONTEXT_KEY = LUMIS_CHAT_CONTEXT_KEY;
