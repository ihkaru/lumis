export interface Token {
  type: "string" | "number" | "bool" | "null" | "ident" | "state_var" | "builtin" | "op" | "punct";
  value: any;
  pos: number;
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
      tokens.push({ type: "string", value: str, pos: i });
      continue;
    }

    // Numbers
    if (/[0-9]/.test(ch) || (ch === "-" && /[0-9]/.test(input[i + 1] || ""))) {
      let numStr = ch;
      i++;
      while (i < input.length && /[0-9.]/.test(input[i])) {
        numStr += input[i];
        i++;
      }
      tokens.push({ type: "number", value: parseFloat(numStr), pos: i });
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
      tokens.push({ type: "builtin", value: name, pos: i });
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
      tokens.push({ type: "state_var", value: varName, pos: i });
      continue;
    }

    // Identifiers / Keywords
    if (/[a-zA-Z_]/.test(ch)) {
      let ident = "";
      while (i < input.length && /[a-zA-Z0-9_]/.test(input[i])) {
        ident += input[i];
        i++;
      }
      if (ident === "true") tokens.push({ type: "bool", value: true, pos: i });
      else if (ident === "false") tokens.push({ type: "bool", value: false, pos: i });
      else if (ident === "null") tokens.push({ type: "null", value: null, pos: i });
      else tokens.push({ type: "ident", value: ident, pos: i });
      continue;
    }

    // Multi-char operators
    const two = input.slice(i, i + 2);
    if (["==", "!=", "<=", ">=", "&&", "||"].includes(two)) {
      tokens.push({ type: "op", value: two, pos: i });
      i += 2;
      continue;
    }

    // Single-char operators and punctuation
    if (["+", "-", "*", "/", "%", "<", ">", "!"].includes(ch)) {
      tokens.push({ type: "op", value: ch, pos: i });
      i++;
      continue;
    }

    if (["(", ")", "[", "]", "{", "}", ",", ":", "?", "."].includes(ch)) {
      tokens.push({ type: "punct", value: ch, pos: i });
      i++;
      continue;
    }

    // Unknown character, advance
    i++;
  }

  return tokens;
}
