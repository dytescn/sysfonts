import { callFfi } from "./ffi.ts";

// ============================================================
// export_font
// 入参: { postscript_name: string, dest_dir: string }
// 把系统里已安装的某个字体复制到指定目录。
// ============================================================

const postscriptName = Deno.args[0];
const destDir = Deno.args[1];

if (!postscriptName || !destDir) {
  console.error("用法: deno run --allow-net export.ts <postscript_name> <dest_dir>");
  Deno.exit(1);
}

try {
  const result = await callFfi<{ path: string }>("export_font", {
    postscript_name: postscriptName,
    dest_dir: destDir,
  });
  console.log("export_font -> ok");
  console.log("导出到:", result.path);
} catch (err) {
  console.error("export_font 失败:", (err as Error).message);
  Deno.exit(1);
}
