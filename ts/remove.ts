import { callFfi } from "./ffi.ts";

// ============================================================
// uninstall_font
// 入参: { postscript_name: string, delete_file?: boolean }
// ============================================================

const postscriptName = Deno.args[0];
const deleteFile = Deno.args[1] === "true";

if (!postscriptName) {
  console.error("用法: deno run --allow-net remove.ts <postscript_name> [true|false]");
  console.error("  delete_file 默认 false，只取消注册；传 true 会同时删除文件");
  Deno.exit(1);
}

try {
  const result = await callFfi<Record<string, never>>("uninstall_font", {
    postscript_name: postscriptName,
    delete_file: deleteFile,
  });
  console.log("uninstall_font -> ok");
  console.log(result);
} catch (err) {
  console.error("uninstall_font 失败:", (err as Error).message);
  Deno.exit(1);
}
