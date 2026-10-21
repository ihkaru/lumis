export interface Token {
  type: "string" | "number" | "bool" | "null" | "ident" | "state_var" | "builtin" | "op" | "punct";
  value: any;
  pos: number;
  start?: number;
  end?: number;
}

export function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;

  while (i < input.length) {
    const ch = input[i];

    if (/\s/.test(ch)) {
      i++;
      continue;
    }

    const start = i;

    // Hex colors or comments
    if (ch === "#") {
      if (i + 1 < input.length && /[a-fA-F0-9]/.test(input[i + 1])) {
        let hex = "#";
        i++;
        while (i < input.length && /[a-fA-F0-9]/.test(input[i])) {
          hex += input[i];
          i++;
        }
        tokens.push({ type: "string", value: hex, pos: i, start, end: i });
        continue;
      } else {
        // Comment until newline
        i++;
        while (i < input.length && input[i] !== "\n") {
          i++;
        }
        continue;
      }
    }

    // Strings
    if (ch === '"' || ch === "'") {
      const quote = ch;
      let str = "";
      i++;
      while (i < input.length && input[i] !== quote) {
        if (input[i] === "\\" && i + 1 < input.length) {
          i++;
          if (input[i] === "n") str += "\n";
          else if (input[i] === "t") str += "\t";
          else str += input[i];
        } else {
          str += input[i];
        }
        i++;
      }
      if (i < input.length) i++; // consume closing quote
      tokens.push({ type: "string", value: str, pos: i, start, end: i });
      continue;
    }

    // Numbers & Percentages
    if (/[0-9]/.test(ch) || (ch === "-" && /[0-9]/.test(input[i + 1] || ""))) {
      let numStr = ch;
      i++;
      while (i < input.length && /[0-9.]/.test(input[i])) {
        numStr += input[i];
        i++;
      }
      if (i < input.length && input[i] === "%") {
        i++;
        tokens.push({ type: "string", value: numStr + "%", pos: i, start, end: i });
      } else {
        tokens.push({ type: "number", value: parseFloat(numStr), pos: i, start, end: i });
      }
      continue;
    }

    // Built-in @function or Action
    if (ch === "@") {
      let name = "@";
      i++;
      while (i < input.length && /[a-zA-Z0-9_]/.test(input[i])) {
        name += input[i];
        i++;
      }
      tokens.push({ type: "builtin", value: name, pos: i, start, end: i });
      continue;
    }

    // State variable $var
    if (ch === "$") {
      let varName = "";
      i++;
      while (i < input.length && /[a-zA-Z0-9_]/.test(input[i])) {
        varName += input[i];
        i++;
      }
      tokens.push({ type: "state_var", value: varName, pos: i, start, end: i });
      continue;
    }

    // Identifiers / Keywords
    if (/[a-zA-Z_]/.test(ch)) {
      let ident = "";
      while (i < input.length && /[a-zA-Z0-9_]/.test(input[i])) {
        ident += input[i];
        i++;
      }
      if (ident === "true") tokens.push({ type: "bool", value: true, pos: i, start, end: i });
      else if (ident === "false") tokens.push({ type: "bool", value: false, pos: i, start, end: i });
      else if (ident === "null") tokens.push({ type: "null", value: null, pos: i, start, end: i });
      else tokens.push({ type: "ident", value: ident, pos: i, start, end: i });
      continue;
    }

    // Multi-char operators
    const two = input.slice(i, i + 2);
    if (["==", "!=", "<=", ">=", "&&", "||"].includes(two)) {
      tokens.push({ type: "op", value: two, pos: i, start, end: i + 2 });
      i += 2;
      continue;
    }

    // Single-char operators and punctuation
    if (["+", "-", "*", "/", "%", "<", ">", "!", "|"].includes(ch)) {
      tokens.push({ type: "op", value: ch, pos: i, start, end: i + 1 });
      i++;
      continue;
    }

    if (["(", ")", "[", "]", "{", "}", ",", ":", "?", "."].includes(ch)) {
      tokens.push({ type: "punct", value: ch, pos: i, start, end: i + 1 });
      i++;
      continue;
    }

    // Unknown character, advance
    i++;
  }

  return tokens;
}
