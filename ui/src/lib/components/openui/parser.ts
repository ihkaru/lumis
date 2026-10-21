import type { AstExpr, AstProgram, AstStatement } from "./types";
import { tokenize, type Token } from "./tokenizer";

export class OpenUiParser {
  private tokens: Token[] = [];
  private idx = 0;

  public parseProgram(source: string): AstProgram {
    const rawStmts = this.splitStatements(source);
    const statements: AstStatement[] = [];

    for (const rawStmt of rawStmts) {
      const stmt = rawStmt.trim();
      if (!stmt) continue;

      const assign = this.findAssignment(stmt);
      if (assign) {
        const isState = assign.left.startsWith("$");
        const target = isState ? assign.left.slice(1) : assign.left;
        try {
          const expr = this.parseExpression(assign.right);
          statements.push({ target, isState, expr });
        } catch {
          // Tolerant to streaming incomplete lines
        }
      } else {
        try {
          const expr = this.parseExpression(stmt);
          statements.push({ target: "root", isState: false, expr });
        } catch {
          // Tolerant to streaming incomplete lines
        }
      }
    }

    return { statements, rootTarget: "root" };
  }

  private splitStatements(source: string): string[] {
    const statements: string[] = [];
    let current = "", depth = 0, inString = false, stringChar = "", inComment = false;

    for (let i = 0; i < source.length; i++) {
      const char = source[i], nextChar = source[i + 1];
      if (inComment) {
        if (char === "\n") inComment = false;
        continue;
      }
      if (!inString && (char === "#" || (char === "/" && nextChar === "/"))) {
        inComment = true;
        continue;
      }
      if (inString) {
        current += char;
        if (char === "\\" && i + 1 < source.length) current += source[++i];
        else if (char === stringChar) inString = false;
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

  private findAssignment(stmt: string): { left: string; right: string } | null {
    let depth = 0, inString = false, stringChar = "";

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
      if (char === "(" || char === "[" || char === "{") { depth++; continue; }
      if (char === ")" || char === "]" || char === "}") { if (depth > 0) depth--; continue; }

      if (depth === 0 && char === "=") {
        const prev = i > 0 ? stmt[i - 1] : "";
        const next = i + 1 < stmt.length ? stmt[i + 1] : "";
        if (next === "=" || next === ">" || prev === "=" || prev === "!" || prev === "<" || prev === ">") continue;

        const left = stmt.slice(0, i).trim();
        const right = stmt.slice(i + 1).trim();
        if (/^\$?[a-zA-Z_][a-zA-Z0-9_]*$/.test(left)) return { left, right };
      }
    }
    return null;
  }

  public parseExpression(input: string): AstExpr {
    this.tokens = tokenize(input);
    this.idx = 0;
    return this.parseTernary();
  }

  private parseTernary(): AstExpr {
    let expr = this.parseLogicalOr();
    if (this.matchPunct("?")) {
      const trueExpr = this.parseTernary();
      this.consumePunct(":");
      const falseExpr = this.parseTernary();
      return { type: "ternary", condition: expr, trueExpr, falseExpr };
    }
    return expr;
  }

  private parseLogicalOr(): AstExpr {
    let left = this.parseLogicalAnd();
    while (this.matchOp("||")) {
      const right = this.parseLogicalAnd();
      left = { type: "binary", operator: "||", left, right };
    }
    return left;
  }

  private parseLogicalAnd(): AstExpr {
    let left = this.parseEquality();
    while (this.matchOp("&&")) {
      const right = this.parseEquality();
      left = { type: "binary", operator: "&&", left, right };
    }
    return left;
  }

  private parseEquality(): AstExpr {
    let left = this.parseComparison();
    while (this.peek()?.type === "op" && ["==", "!="].includes(this.peek()?.value)) {
      const op = this.next().value;
      const right = this.parseComparison();
      left = { type: "binary", operator: op, left, right };
    }
    return left;
  }

  private parseComparison(): AstExpr {
    let left = this.parseAdditive();
    while (this.peek()?.type === "op" && ["<", "<=", ">", ">="].includes(this.peek()?.value)) {
      const op = this.next().value;
      const right = this.parseAdditive();
      left = { type: "binary", operator: op, left, right };
    }
    return left;
  }

  private parseAdditive(): AstExpr {
    let left = this.parseMultiplicative();
    while (this.peek()?.type === "op" && ["+", "-"].includes(this.peek()?.value)) {
      const op = this.next().value;
      const right = this.parseMultiplicative();
      left = { type: "binary", operator: op, left, right };
    }
    return left;
  }

  private parseMultiplicative(): AstExpr {
    let left = this.parseUnary();
    while (this.peek()?.type === "op" && ["*", "/", "%"].includes(this.peek()?.value)) {
      const op = this.next().value;
      const right = this.parseUnary();
      left = { type: "binary", operator: op, left, right };
    }
    return left;
  }

  private parseUnary(): AstExpr {
    if (this.peek()?.type === "op" && ["!", "-"].includes(this.peek()?.value)) {
      const op = this.next().value;
      const argument = this.parseUnary();
      return { type: "unary", operator: op, argument };
    }
    return this.parsePostfix();
  }

  private parsePostfix(): AstExpr {
    let expr = this.parsePrimary();

    while (this.idx < this.tokens.length) {
      if (this.matchPunct(".")) {
        const propToken = this.next();
        const property = propToken?.type === "ident" ? propToken.value : String(propToken?.value || "");
        expr = { type: "member", object: expr, property };
      } else if (this.matchPunct("[")) {
        const index = this.parseTernary();
        this.consumePunct("]");
        expr = { type: "index", object: expr, index };
      } else if (this.matchPunct("(")) {
        const args: AstExpr[] = [];
        if (!this.checkPunct(")")) {
          do {
            args.push(this.parseTernary());
          } while (this.matchPunct(","));
        }
        this.consumePunct(")");
        const name = expr.type === "identifier" ? expr.name : expr.type === "call" ? expr.name : "Call";
        expr = { type: "call", name, args };
      } else {
        break;
      }
    }

    return expr;
  }

  private parsePrimary(): AstExpr {
    const t = this.next();
    if (!t) return { type: "literal", value: null };

    if (t.type === "string" || t.type === "number" || t.type === "bool" || t.type === "null") {
      return { type: "literal", value: t.value };
    }

    if (t.type === "state_var") {
      return { type: "state_var", name: t.value };
    }

    if (t.type === "builtin" || t.type === "ident") {
      return { type: "identifier", name: t.value };
    }

    if (t.type === "punct" && t.value === "[") {
      const elements: AstExpr[] = [];
      if (!this.checkPunct("]")) {
        do {
          elements.push(this.parseTernary());
        } while (this.matchPunct(","));
      }
      this.consumePunct("]");
      return { type: "array", elements };
    }

    if (t.type === "punct" && t.value === "{") {
      const properties: Record<string, AstExpr> = {};
      if (!this.checkPunct("}")) {
        do {
          const keyToken = this.next();
          const key = keyToken ? String(keyToken.value) : "";
          this.consumePunct(":");
          properties[key] = this.parseTernary();
        } while (this.matchPunct(","));
      }
      this.consumePunct("}");
      return { type: "object", properties };
    }

    if (t.type === "punct" && t.value === "(") {
      const inner = this.parseTernary();
      this.consumePunct(")");
      return inner;
    }

    return { type: "literal", value: null };
  }

  private peek(): Token | undefined {
    return this.tokens[this.idx];
  }

  private next(): Token {
    return this.tokens[this.idx++];
  }

  private checkPunct(ch: string): boolean {
    const t = this.peek();
    return !!t && t.type === "punct" && t.value === ch;
  }

  private matchPunct(ch: string): boolean {
    if (this.checkPunct(ch)) {
      this.idx++;
      return true;
    }
    return false;
  }

  private consumePunct(ch: string): void {
    if (this.checkPunct(ch)) {
      this.idx++;
    }
  }

  private matchOp(op: string): boolean {
    const t = this.peek();
    if (t && t.type === "op" && t.value === op) {
      this.idx++;
      return true;
    }
    return false;
  }
}
