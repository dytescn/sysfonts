const BASE_URL = Deno.env.get("SYSFONTS_URL") ??
  "http://127.0.0.1:44944/libsysfonts";

/**
 * 调用 FFI 符号。
 *
 * 注意：symbol 传原始函数名（如 "install_font"），
 * 桥接层会自动拼接 "post_" 前缀后再 dlsym。
 * 不要自己加 "post_"，否则会变成 "post_post_install_font"。
 */
export async function callFfi<T>(
  symbol: string,
  payload: Record<string, unknown>,
): Promise<T> {
  const response = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/libary",
      "FFI-Symbol": symbol,
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    throw new Error(`HTTP 错误: ${response.status} - ${response.statusText}`);
  }

  const data = await response.json();
  if (data.code === 200) return data.data as T;
  throw new Error(`err info: ${data.msg} (code=${data.code})`);
}

/**
 * 无参 FFI 调用（对应 list_system_fonts）。
 */
export async function callFfiNoArg<T>(symbol: string): Promise<T> {
  const response = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/libary",
      "FFI-Symbol": symbol,
    },
    body: "",
  });

  if (!response.ok) {
    throw new Error(`HTTP 错误: ${response.status} - ${response.statusText}`);
  }

  const data = await response.json();
  if (data.code === 200) return data.data as T;
  throw new Error(`err info: ${data.msg} (code=${data.code})`);
}
