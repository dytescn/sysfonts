const BASE_URL = "http://127.0.0.1:44944/libdesigndb";

async function callFfi(symbol: string, payload: Record<string, unknown>) {
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
  if (data.code === 200) return data.data;
  throw new Error(`err info: ${data.msg} (code=${data.code})`);
}

const db_insert = (path: string, collection: string, doc: Record<string, unknown>) =>
  callFfi("db_insert", { path, collection, doc });

const db_query = (
  path: string,
  collection: string,
  options?: { filter?: Record<string, unknown> },
) => callFfi("db_query", { path, collection, ...options });

function assertEq<T>(actual: T, expected: T, msg: string) {
  if (actual !== expected) {
    throw new Error(
      `ASSERT FAILED: ${msg}\n  expected: ${JSON.stringify(expected)}\n  actual:   ${JSON.stringify(actual)}`,
    );
  }
}

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(`ASSERT FAILED: ${msg}`);
}

async function runCase(name: string, fn: () => Promise<void>) {
  try {
    await fn();
    console.log(`✅ ${name}`);
  } catch (e) {
    console.error(`❌ ${name}`);
    console.error(e);
    Deno.exitCode = 1;
  }
}

const path = `test_insert_${Date.now()}`;
const collection = "users";

await runCase("insert: 单条", async () => {
  await db_insert(path, collection, { name: "tom", age: 18 });
  const arr = (await db_query(path, collection, { filter: { name: "tom" } })) as any[];
  assertEq(arr.length, 1, "找到 1 条");
  assertEq(arr[0].age, 18, "age");
});

await runCase("insert: 多条", async () => {
  for (let i = 0; i < 5; i++) {
    await db_insert(path, collection, { name: `u${i}`, age: 20 + i });
  }
  const arr = (await db_query(path, collection)) as any[];
  assert(arr.length >= 6, `至少 6 条，实际 ${arr.length}`);
});

await runCase("insert: 嵌套文档 + 数组", async () => {
  await db_insert(path, collection, {
    name: "nested",
    address: { city: "beijing", zip: "100000" },
    tags: ["a", "b", "c"],
  });
  const arr = (await db_query(path, collection, { filter: { name: "nested" } })) as any[];
  assertEq(arr.length, 1, "找到 1 条");
  assertEq(arr[0].address.city, "beijing", "nested city");
  assertEq(arr[0].tags[2], "c", "array index 2");
});

await runCase("insert: null / bool / float", async () => {
  await db_insert(path, collection, {
    name: "types",
    a: null,
    b: true,
    c: false,
    pi: 3.14,
  });
  const arr = (await db_query(path, collection, { filter: { name: "types" } })) as any[];
  assertEq(arr.length, 1, "找到 1 条");
  assertEq(arr[0].a, null, "a 是 null");
  assertEq(arr[0].b, true, "b 是 true");
  assertEq(arr[0].c, false, "c 是 false");
  assertEq(arr[0].pi, 3.14, "pi");
});

await runCase("insert: 结果含 _id", async () => {
  const arr = (await db_query(path, collection, { filter: { name: "tom" } })) as any[];
  assert(arr[0]._id, "应该有 _id");
  assertEq(typeof arr[0]._id, "string", "_id 是字符串");
});

await runCase("insert: 缺 doc 报错", async () => {
  let caught = false;
  try {
    await callFfi("db_insert", { path, collection });
  } catch (e) {
    caught = true;
    if (!(e as Error).message.includes("400103")) {
      throw new Error(`期望 400103，实际: ${(e as Error).message}`);
    }
  }
  if (!caught) throw new Error("应该抛错但没有");
});
