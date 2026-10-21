export function splitStatements(source: string): string[] {
  const statements: string[] = [];
  let current = "";
  let depth = 0;
  let inString = false;
  let stringChar = "";
  let inComment = false;

  for (let i = 0; i < source.length; i++) {
    const char = source[i];
    const nextChar = source[i + 1];

    if (inComment) {
      if (char === "\n") inComment = false;
      continue;
    }

    const isHexColor = char === "#" && i + 1 < source.length && /[a-fA-F0-9]/.test(source[i + 1]);
    if (!inString && !isHexColor && (char === "#" || (char === "/" && nextChar === "/"))) {
      inComment = true;
      continue;
    }

    if (inString) {
      current += char;
      if (char === "\\" && i + 1 < source.length) {
        current += source[++i];
      } else if (char === stringChar) {
        inString = false;
      }
      continue;
    }

    if (char === '"' || char === "'") {
      inString = true;
      stringChar = char;
      current += char;
      continue;
    }

    if (char === "(" || char === "[" || char === "{") {
      depth++;
      current += char;
    } else if (char === ")" || char === "]" || char === "}") {
      if (depth > 0) depth--;
      current += char;
    } else if ((char === "\n" || char === ";") && depth === 0) {
      if (current.trim()) statements.push(current.trim());
      current = "";
    } else {
      current += char;
    }
  }

  if (current.trim()) statements.push(current.trim());
  return statements;
}

export function findAssignment(stmt: string): { left: string; right: string } | null {
  let depth = 0;
  let inString = false;
  let stringChar = "";

  for (let i = 0; i < stmt.length; i++) {
    const char = stmt[i];

    if (inString) {
      if (char === "\\" && i + 1 < stmt.length) i++;
      else if (char === stringChar) inString = false;
      continue;
    }

    if (char === '"' || char === "'") {
      inString = true;
      stringChar = char;
      continue;
    }

    if (char === "(" || char === "[" || char === "{") {
      depth++;
      continue;
    }
    if (char === ")" || char === "]" || char === "}") {
      if (depth > 0) depth--;
      continue;
    }

    if (depth === 0 && char === "=") {
      const prev = i > 0 ? stmt[i - 1] : "";
      const next = i + 1 < stmt.length ? stmt[i + 1] : "";
      if (next === "=" || next === ">" || prev === "=" || prev === "!" || prev === "<" || prev === ">") {
        continue;
      }

      const left = stmt.slice(0, i).trim();
      const right = stmt.slice(i + 1).trim();
      if (/^\$?[a-zA-Z_][a-zA-Z0-9_]*$/.test(left)) {
        return { left, right };
      }
    }
  }

  return null;
}
