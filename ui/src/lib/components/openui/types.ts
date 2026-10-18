export type AstExpr =
  | { type: "literal"; value: any }
  | { type: "identifier"; name: string }
  | { type: "state_var"; name: string }
  | { type: "call"; name: string; args: AstExpr[] }
  | { type: "array"; elements: AstExpr[] }
  | { type: "object"; properties: Record<string, AstExpr> }
  | { type: "member"; object: AstExpr; property: string }
  | { type: "index"; object: AstExpr; index: AstExpr }
  | { type: "binary"; operator: string; left: AstExpr; right: AstExpr }
  | { type: "unary"; operator: string; argument: AstExpr }
  | { type: "ternary"; condition: AstExpr; trueExpr: AstExpr; falseExpr: AstExpr };

export interface AstStatement {
  target: string;
  isState: boolean;
  expr: AstExpr;
}

export interface AstProgram {
  statements: AstStatement[];
  rootTarget: string;
}

export interface UiNode {
  type: string;
  props: Record<string, any>;
  children?: (UiNode | string | number | boolean | null)[];
}

export interface OpenUiContext {
  state: Record<string, any>;
  initialState: Record<string, any>;
  definitions: Map<string, AstExpr>;
  onToAssistant?: (prompt: string, context?: any) => void;
  onOpenUrl?: (url: string) => void;
  onCopy?: (text: string) => void;
}
