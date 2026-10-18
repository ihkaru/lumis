import type { AstExpr, AstProgram, AstStatement } from "./types";
import { tokenize, type Token } from "./tokenizer";

export class OpenUiParser {
  private tokens: Token[] = [];
  private idx = 0;

  public parseProgram(source: string): AstProgram {
    const lines = source.split("\n");
    const statements: AstStatement[] = [];

    for (const rawLine of lines) {
      const line = rawLine.trim();
      if (!line || line.startsWith("#") || line.startsWith("//")) continue;

      const eqIdx = line.indexOf("=");
      if (eqIdx === -1) continue;

      const left = line.slice(0, eqIdx).trim();
      const right = line.slice(eqIdx + 1).trim();
      if (!left || !right) continue;

      const isState = left.startsWith("$");
      const target = isState ? left.slice(1) : left;

      try {
        const expr = this.parseExpression(right);
        statements.push({ target, isState, expr });
      } catch {
        // Tolerant to streaming incomplete lines
      }
    }

    return { statements, rootTarget: "root" };
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
