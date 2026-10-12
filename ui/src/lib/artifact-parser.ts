import type { ArtifactSpecification } from "./types";

export interface ParsedMessageContent {
  cleanText: string;
  artifacts: ArtifactSpecification[];
}

/**
 * Deterministically parses XML/fenced lumis_artifact blocks from text.
 * Prevents premature code fence collapses (CommonMark safe).
 */
export function parseArtifactsFromContent(content: string): ParsedMessageContent {
  const artifacts: ArtifactSpecification[] = [];
  const tagOpen = "<lumis_artifact";
  const tagClose = "</lumis_artifact>";
  let cleanText = "";
  let searchFrom = 0;
  let ver = 1;

  while (true) {
    const startIdx = content.indexOf(tagOpen, searchFrom);
    if (startIdx === -1) {
      cleanText += content.slice(searchFrom);
      break;
    }
    cleanText += content.slice(searchFrom, startIdx);
    const afterOpen = startIdx + tagOpen.length;
    const headerEnd = content.indexOf(">", afterOpen);
    if (headerEnd === -1) {
      cleanText += content.slice(startIdx);
      break;
    }
    const headerStr = content.slice(afterOpen, headerEnd);
    const contentStart = headerEnd + 1;
    const closeIdx = content.indexOf(tagClose, contentStart);
    if (closeIdx === -1) {
      cleanText += content.slice(startIdx);
      break;
    }
    const rawContent = content.slice(contentStart, closeIdx).trim();
    const idMatch = headerStr.match(/identifier=["']([^"']+)["']/);
    const typeMatch = headerStr.match(/type=["']([^"']+)["']/);
    const titleMatch = headerStr.match(/title=["']([^"']+)["']/);

    const identifier = idMatch ? idMatch[1] : `artifact-${ver}`;
    const artifact_type = typeMatch ? typeMatch[1] : "text/html";
    const title = titleMatch ? titleMatch[1] : identifier;

    artifacts.push({
      identifier,
      artifact_type,
      title,
      content: rawContent,
      version: ver,
    });
    ver += 1;
    searchFrom = closeIdx + tagClose.length;
  }

  return { cleanText: cleanText.trim(), artifacts };
}
