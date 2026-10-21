import { mathBuiltins } from "./builtins_math";
import { dateBuiltins } from "./builtins_date";
import { collectionBuiltins } from "./builtins_collections";

// Aggregated pure browser built-in functions for OpenUI
export const openUiBuiltins: Record<string, (...args: any[]) => any> = {
  ...mathBuiltins,
  ...dateBuiltins,
  ...collectionBuiltins,
};
