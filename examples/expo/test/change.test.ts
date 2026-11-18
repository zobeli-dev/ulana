// The loop in changeView, with OpenRouter and `ulana serve` stood in by a fake fetch.

import { afterEach, expect, test } from "bun:test";

import { changeView } from "../src/ai";

const host = { spec: "SPEC", blocks: ["calendar"], actions: ["send"], images: ["https://picsum.photos/"], edit: true };
const real = globalThis.fetch;
afterEach(() => {
  globalThis.fetch = real;
});

type Call = { url: string; body: any };

/** Answers the model's turns in order; the server fails any script containing "BROKEN". */
function fake(answers: string[]) {
  const calls: Call[] = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    const url = String(input);
    const body = init?.body ? JSON.parse(String(init.body)) : null;
    calls.push({ url, body });
    const json = (status: number, value: unknown) => new Response(JSON.stringify(value), { status });
    if (url.endsWith("/source") && !init?.method) {
      return json(200, { name: "counter", source: "return ui.text('old')\n", data: { n: 1 } });
    }
    if (url.endsWith("/source") && init?.method === "PUT") {
      const broken = body.source.includes("BROKEN");
      return json(200, {
        name: "counter",
        revision: calls.length,
        frame: { version: 1, root: { type: "text", text: "x" } },
        error: broken ? "at root: unknown field `BROKEN`" : null,
        logs: [],
      });
    }
    if (url.endsWith("/chat/completions")) {
      const content = answers.shift() ?? "";
      return json(200, { choices: [{ message: { content } }], usage: { cost: 0.001 } });
    }
    return json(404, { error: "not found" });
  }) as typeof fetch;
  return calls;
}

const args = (request: string) => ({
  key: "sk-test",
  host,
  name: "counter",
  request,
  history: [{ role: "user" as const, content: "earlier" }],
  error: null,
  tones: { accent: "#2563EB" },
  onStatus: () => {},
});

test("a failing script goes back to the model with its error, and the fix is kept", async () => {
  const calls = fake([
    "Made it bold.\n```luau\nreturn ui.text{ 'new', BROKEN = 1 }\n```",
    "Removed the field.\n```luau\nreturn ui.text('new')\n```",
  ]);
  const change = await changeView(args("make it bold"));
  expect(change).toMatchObject({ reply: "Made it bold.", changed: true, error: null, tries: 2, previous: "return ui.text('old')\n" });
  expect(change.cost).toBeCloseTo(0.002);

  const model = calls.filter((c) => c.url.endsWith("/chat/completions"));
  expect(model).toHaveLength(2);
  const first = model[0]!.body;
  expect(first.model).toBe("z-ai/glm-5.3-flash");
  expect(first.messages[0].content).toContain("SPEC");
  expect(first.messages[0].content).toContain('ui.custom("calendar", props)');
  expect(first.messages[0].content).toContain("accent #2563EB");
  expect(first.messages[0].content).toContain('addresses starting with "https://picsum.photos/"');
  expect(first.messages[1]).toEqual({ role: "user", content: "earlier" });
  expect(first.messages[2].content).toContain("return ui.text('old')");
  expect(first.messages[2].content).toContain('"n": 1');
  expect(first.messages[2].content).toEndWith("My request: make it bold");
  const retry = model[1]!.body.messages;
  expect(retry.at(-1).content).toContain("unknown field `BROKEN`");
  expect(retry.at(-2).role).toBe("assistant");

  const puts = calls.filter((c) => c.url.endsWith("/source") && c.body);
  expect(puts.map((p) => p.body.source)).toEqual(["return ui.text{ 'new', BROKEN = 1 }\n", "return ui.text('new')\n"]);
});

test("it stops after two repairs and reports the error", async () => {
  fake(["a\n```luau\nreturn BROKEN\n```", "b\n```luau\nreturn BROKEN\n```", "c\n```luau\nreturn BROKEN\n```", "never asked"]);
  const change = await changeView(args("x"));
  expect(change.tries).toBe(3);
  expect(change.error).toContain("BROKEN");
});

test("a question gets an answer and nothing is written", async () => {
  const calls = fake(["It counts presses with ui.state."]);
  const change = await changeView(args("how does it count?"));
  expect(change).toMatchObject({ changed: false, reply: "It counts presses with ui.state.", tries: 1 });
  expect(calls.some((c) => c.body?.source)).toBe(false);
});

test("a refused key says so", async () => {
  globalThis.fetch = (async (input: string | URL | Request) =>
    String(input).endsWith("/source")
      ? new Response(JSON.stringify({ name: "counter", source: "", data: null }))
      : new Response("{}", { status: 401 })) as typeof fetch;
  await expect(changeView(args("x"))).rejects.toThrow("OpenRouter refused the key");
});
