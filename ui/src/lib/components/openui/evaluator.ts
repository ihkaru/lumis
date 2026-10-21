import type { AstExpr, AstProgram, OpenUiContext, UiNode } from "./types";
import { openUiBuiltins } from "./builtins";
import { constructUiNode } from "./constructors";

export class OpenUiEvaluator {
  private visitedDefs = new Set<string>();

  public evaluateProgram(program: AstProgram, ctx: OpenUiContext): UiNode | null {
    ctx.definitions.clear();
    // 1. Initialize definitions and record initial states
    for (const stmt of program.statements) {
      if (stmt.isState) {
        if (!(stmt.target in ctx.state) && !(stmt.target in ctx.initialState)) {
          const val = this.evalExpr(stmt.expr, ctx, new Map());
          ctx.initialState[stmt.target] = JSON.parse(JSON.stringify(val ?? null));
          if (!ctx.setState) {
            ctx.state[stmt.target] = val;
          }
        }
      } else {
        ctx.definitions.set(stmt.target, stmt.expr);
      }
    }

    // 2. Resolve root expression
    let rootExpr = ctx.definitions.get(program.rootTarget);
    if (!rootExpr) {
      rootExpr = ctx.definitions.get("app")
        ?? ctx.definitions.get("main")
        ?? ctx.definitions.get("calculator")
        ?? ctx.definitions.get("ui")
        ?? Array.from(ctx.definitions.values()).pop();
    }
    if (!rootExpr) return null;

    this.visitedDefs.clear();
    const result = this.evalExpr(rootExpr, ctx, new Map());
    if (result && typeof result === "object" && "type" in result) {
      return result as UiNode;
    }

    return {
      type: "Col",
      props: {},
      children: Array.isArray(result) ? result : [result],
    };
  }

  public evalExpr(expr: AstExpr, ctx: OpenUiContext, scope: Map<string, any>): any {
    if (!expr) return null;

    switch (expr.type) {
      case "literal":
        if (typeof expr.value === "string" && (expr.value.includes("$") || expr.value.includes("@"))) {
          return this.interpolateString(expr.value, ctx, scope);
        }
        return expr.value;

      case "state_var":
        return expr.name in ctx.state ? ctx.state[expr.name] : ctx.initialState[expr.name];

      case "identifier": {
        const name = expr.name;
        if (scope.has(name)) return scope.get(name);
        if (name in openUiBuiltins) return openUiBuiltins[name];

        // Hoisted reference resolution
        if (ctx.definitions.has(name)) {
          if (this.visitedDefs.has(name)) return null;
          this.visitedDefs.add(name);
          const val = this.evalExpr(ctx.definitions.get(name)!, ctx, scope);
          this.visitedDefs.delete(name);
          return val;
        }

        return name;
      }

      case "array":
        return expr.elements.map((el) => this.evalExpr(el, ctx, scope));

      case "object": {
        const obj: Record<string, any> = {};
        for (const [k, v] of Object.entries(expr.properties)) {
          obj[k] = this.evalExpr(v, ctx, scope);
        }
        return obj;
      }

      case "member": {
        const obj = this.evalExpr(expr.object, ctx, scope);
        if (Array.isArray(obj)) {
          return obj.map((item) => (item ? item[expr.property] : undefined));
        }
        return obj ? obj[expr.property] : undefined;
      }

      case "index": {
        const obj = this.evalExpr(expr.object, ctx, scope);
        const idx = this.evalExpr(expr.index, ctx, scope);
        return obj ? obj[idx] : undefined;
      }

      case "unary": {
        const val = this.evalExpr(expr.argument, ctx, scope);
        if (expr.operator === "!") return !val;
        if (expr.operator === "-") return -(Number(val) || 0);
        return val;
      }

      case "binary": {
        const left = this.evalExpr(expr.left, ctx, scope);
        const right = this.evalExpr(expr.right, ctx, scope);
        const op = expr.operator;

        if (op === "+") {
          if (typeof left === "string" || typeof right === "string") {
            return String(left ?? "") + String(right ?? "");
          }
          return (Number(left) || 0) + (Number(right) || 0);
        }
        if (op === "-") return (Number(left) || 0) - (Number(right) || 0);
        if (op === "*") return (Number(left) || 0) * (Number(right) || 0);
        if (op === "/") {
          const denom = Number(right) || 0;
          return denom !== 0 ? (Number(left) || 0) / denom : 0;
        }
        if (op === "%") return (Number(left) || 0) % (Number(right) || 1);
        if (op === "==") return left == right;
        if (op === "!=") return left != right;
        if (op === "<") return left < right;
        if (op === "<=") return left <= right;
        if (op === ">") return left > right;
        if (op === ">=") return left >= right;
        if (op === "&&") return left && right;
        if (op === "||") return left || right;
        return null;
      }

      case "ternary": {
        const cond = this.evalExpr(expr.condition, ctx, scope);
        return cond ? this.evalExpr(expr.trueExpr, ctx, scope) : this.evalExpr(expr.falseExpr, ctx, scope);
      }

      case "call":
        return this.evalCall(expr, ctx, scope);

      default:
        return null;
    }
  }

  private evalCall(call: { name: string; args: AstExpr[] }, ctx: OpenUiContext, scope: Map<string, any>): any {
    const name = call.name;

    // 1. @Each loop: @Each(array, "varName", templateExpr)
    if (name === "@Each") {
      const arr = this.evalExpr(call.args[0], ctx, scope);
      const varNameExpr = call.args[1];
      const varName = varNameExpr && "value" in varNameExpr ? String(varNameExpr.value) : "item";
      const template = call.args[2];

      if (!Array.isArray(arr) || !template) return [];

      return arr.map((item) => {
        const itemScope = new Map(scope);
        itemScope.set(varName, item);
        return this.evalExpr(template, ctx, itemScope);
      });
    }

    // 2. Actions: @Set, @Reset, @CopyToClipboard, @ToAssistant, @OpenUrl
    if (name === "@Set") {
      const targetArg = call.args[0];
      const targetVar = targetArg?.type === "state_var" ? targetArg.name : null;
      return () => {
        if (targetVar) {
          const val = this.evalExpr(call.args[1], ctx, scope);
          if (ctx.setState) {
            ctx.setState(targetVar, val);
          } else {
            ctx.state[targetVar] = val;
          }
        }
      };
    }

    if (name === "@Reset") {
      const varArgs = call.args.filter((a) => a.type === "state_var") as { type: "state_var"; name: string }[];
      return () => {
        for (const va of varArgs) {
          if (va.name in ctx.initialState) {
            const val = JSON.parse(JSON.stringify(ctx.initialState[va.name]));
            if (ctx.setState) {
              ctx.setState(va.name, val);
            } else {
              ctx.state[va.name] = val;
            }
          }
        }
      };
    }

    if (name === "@CopyToClipboard") {
      return () => {
        const text = String(this.evalExpr(call.args[0], ctx, scope) ?? "");
        if (ctx.onCopy) ctx.onCopy(text);
        else if (typeof navigator !== "undefined" && navigator.clipboard) {
          navigator.clipboard.writeText(text);
        }
      };
    }

    if (name === "@ToAssistant") {
      return () => {
        const prompt = String(this.evalExpr(call.args[0], ctx, scope) ?? "");
        const contextData = call.args[1] ? this.evalExpr(call.args[1], ctx, scope) : ctx.state;
        if (ctx.onToAssistant) ctx.onToAssistant(prompt, contextData);
      };
    }

    if (name === "@OpenUrl") {
      return () => {
        const url = String(this.evalExpr(call.args[0], ctx, scope) ?? "");
        if (ctx.onOpenUrl) ctx.onOpenUrl(url);
        else if (typeof window !== "undefined") window.open(url, "_blank", "noopener,noreferrer");
      };
    }

    if (name === "Action") {
      const actionSteps = call.args.map((a) => this.evalExpr(a, ctx, scope));
      return () => {
        const flat = actionSteps.flat();
        for (const step of flat) {
          if (typeof step === "function") step();
        }
      };
    }

    // 3. Regular Builtin @function
    if (name.startsWith("@") && name in openUiBuiltins) {
      const evaluatedArgs = call.args.map((a) => this.evalExpr(a, ctx, scope));
      return openUiBuiltins[name](...evaluatedArgs);
    }

    // 4. Component constructor -> returns UiNode
    return constructUiNode(name, call.args, (e, c, s) => this.evalExpr(e, c, s), ctx, scope);
  }

  private interpolateString(str: string, ctx: OpenUiContext, scope: Map<string, any>): string {
    let res = str;
    const atRegex = /@([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g;
    let match: RegExpExecArray | null;

    while ((match = atRegex.exec(res)) !== null) {
      const startIdx = match.index;
      let depth = 1, endIdx = -1;
      for (let i = startIdx + match[0].length; i < res.length; i++) {
        if (res[i] === "(") depth++;
        else if (res[i] === ")") {
          depth--;
          if (depth === 0) { endIdx = i + 1; break; }
        }
      }

      if (endIdx !== -1 && ctx.parser) {
        const exprCode = res.slice(startIdx, endIdx);
        try {
          const subAst = ctx.parser.parseExpression(exprCode);
          const evalStr = String(this.evalExpr(subAst, ctx, scope) ?? "");
          res = res.slice(0, startIdx) + evalStr + res.slice(endIdx);
          atRegex.lastIndex = startIdx + evalStr.length;
        } catch {}
      }
    }

    return res.replace(/\$([a-zA-Z_][a-zA-Z0-9_]*)/g, (match, varName) => {
      if (varName in ctx.state) return String(ctx.state[varName]);
      if (varName in ctx.initialState) return String(ctx.initialState[varName]);
      return match;
    });
  }
}
