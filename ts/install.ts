import { callFfi } from "./ffi.ts";

// ============================================================
// install_font
// 入参: { font_path: string, scope?: "user" | "system" }
// ============================================================

const fontPath = Deno.args[0];
const scope = (Deno.args[1] as "user" | "system") ?? "user";

if (!fontPath) {
  console.error("用法: deno run --allow-net install.ts <font_path> [user|system]");
  Deno.exit(1);
}

try {
  const result = await callFfi<Record<string, never>>("install_font", {
    font_path: fontPath,
    scope,
  });
  console.log("install_font -> ok");
  console.log(result);
} catch (err) {
  console.error("install_font 失败:", (err as Error).message);
  Deno.exit(1);
}


