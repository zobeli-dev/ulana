// Asking a model on OpenRouter to rewrite a view.
//
// The model gets the view API (spec/ulana.md, from the server), the script
// as it is, the data it reads, and the request. It answers with a sentence
// and the whole new script. The script goes to `ulana serve`, which loads it
// at once; when it fails, the error goes back to the model to fix.

import { getSource, putSource, type HostInfo, type Snapshot } from "./server";

export const MODEL = process.env.EXPO_PUBLIC_ULANA_MODEL || "z-ai/glm-5.3-flash";

const OPENROUTER = "https://openrouter.ai/api/v1";
/** Times the model may try again after its script fails. */
const REPAIRS = 2;
/** Earlier requests and answers sent along, without their scripts. */
const HISTORY = 8;

export type Turn = { role: "user" | "assistant"; content: string };

export type Change = {
  /** What the model said it did. */
  reply: string;
  /** False when it only answered a question. */
  changed: boolean;
  /** The view after the change. */
  snapshot: Snapshot | null;
  /** The error left when every try failed. */
  error: string | null;
  /** The script before, for undo. */
  previous: string;
  tries: number;
  seconds: number;
  cost: number | null;
};

type Message = { role: "system" | "user" | "assistant"; content: string };

/** Check a key with OpenRouter before keeping it. */
export async function checkKey(key: string): Promise<void> {
  const r = await fetch(`${OPENROUTER}/key`, { headers: { Authorization: `Bearer ${key}` } });
  if (r.status === 401 || r.status === 403) throw new Error("OpenRouter refused this key.");
  if (!r.ok) throw new Error(`OpenRouter answered ${r.status}. Try again.`);
}

export async function changeView(args: {
  key: string;
  host: HostInfo;
  name: string;
  request: string;
  history: Turn[];
  error: string | null;
  /** What each tone looks like in this app, such as `{ accent: "#2563EB" }`. */
  tones?: Record<string, string>;
  onStatus: (text: string) => void;
}): Promise<Change> {
  const started = Date.now();
  const { source, data } = await getSource(args.name);
  const messages: Message[] = [
    { role: "system", content: systemPrompt(args.host, args.tones ?? {}) },
    ...args.history.slice(-HISTORY),
    { role: "user", content: request(args.name, source, data, args.error, args.request) },
  ];

  let cost: number | null = null;
  let reply = "";
  let snapshot: Snapshot | null = null;
  let error: string | null = null;
  let tries = 0;
  for (; tries <= REPAIRS; tries++) {
    args.onStatus(tries === 0 ? "Writing the view…" : `Fixing an error (try ${tries + 1})…`);
    const answer = await complete(args.key, messages);
    cost = answer.cost === null ? cost : (cost ?? 0) + answer.cost;
    const { text, script } = split(answer.content);
    if (tries === 0) reply = text;
    if (script === null) {
      if (tries === 0) {
        return { reply: text || "I had nothing to change.", changed: false, snapshot: null, error: null, previous: source, tries: 1, seconds: secs(started), cost };
      }
      error = "The model did not send a script.";
      break;
    }
    args.onStatus("Loading it…");
    snapshot = await putSource(args.name, script);
    error = snapshot.error;
    if (!error) break;
    messages.push(
      { role: "assistant", content: answer.content },
      {
        role: "user",
        content: `That script does not work. Ulana says:\n${error}\n\nFix it and send the whole script again, with one sentence on what you fixed.`,
      },
    );
  }
  return { reply: reply || "Done.", changed: true, snapshot, error, previous: source, tries: Math.min(tries + 1, REPAIRS + 1), seconds: secs(started), cost };
}

function systemPrompt(host: HostInfo, tones: Record<string, string>): string {
  const blocks = host.blocks.length
    ? host.blocks.map((b) => `ui.custom("${b}", props)`).join(", ")
    : "none";
  const actions = host.actions.length ? host.actions.map((a) => `ui.host("${a}", value)`).join(", ") : "none";
  const palette = Object.entries(tones)
    .map(([tone, colour]) => `${tone} ${colour}`)
    .join(", ");
  return `You change the interface of an app by rewriting one Ulana view: a Luau script that builds the screen with the \`ui\` library. The app draws the result with its own native components.

Answer every request in this shape:
1. One or two short sentences in plain words saying what you changed. No headings, lists, or Markdown.
2. The complete new script in one \`\`\`luau code block. Always the whole script, never a part or a diff.

Rules:
- Change what the user asked for and keep everything else as it is.
- Keep the names given to ui.state, so what the user entered (counts, lists, switches) survives the change.
- Use only the API below. Spacing, colours, and sizes are tokens such as "md" or "accent". There are no other colours, fonts, or pixel sizes. When the user asks for something the API cannot do, do the closest thing it can and say so in your sentence.
- The view's function gets the data shown with the request. Read the fields it has; never invent others.
- When the request is a question and not a change, answer it and leave out the code block.

${palette ? `In this app the tones look like this: ${palette}. When the user names a colour, use the tone closest to it.\n` : ""}Custom blocks this app has: ${blocks}.
Host actions this app has: ${actions}.
Images this app shows: ${host.images?.length ? `addresses starting with ${host.images.map((i) => `"${i}"`).join(" or ")}` : "none; leave images out"}.

The API:

${host.spec}`;
}

function request(name: string, source: string, data: unknown, error: string | null, text: string): string {
  const parts = [
    `The view is "${name}". Its script now:`,
    "```luau\n" + source.trim() + "\n```",
    "The data its function gets:",
    "```json\n" + JSON.stringify(data ?? null, null, 2) + "\n```",
  ];
  if (error) parts.push(`The script has this error now:\n${error}`);
  parts.push(`My request: ${text}`);
  return parts.join("\n\n");
}

/** The model's sentence and the last Luau block in its answer, if any. */
export function split(content: string): { text: string; script: string | null } {
  const blocks = [...content.matchAll(/```[ \t]*(luau|lua)?[^\n]*\n([\s\S]*?)```/g)];
  const scripts = blocks.map((m) => m[2] ?? "").filter((code) => /\bui\.|\breturn\b/.test(code));
  const text = content
    .replace(/```[\s\S]*?(```|$)/g, " ")
    .replace(/[*_#`]+/g, "")
    .replace(/\s+/g, " ")
    .trim();
  const script = scripts.length ? scripts[scripts.length - 1]!.trim() + "\n" : null;
  return { text, script };
}

async function complete(key: string, messages: Message[]): Promise<{ content: string; cost: number | null }> {
  const abort = new AbortController();
  const timer = setTimeout(() => abort.abort(), 120_000);
  try {
    const r = await fetch(`${OPENROUTER}/chat/completions`, {
      method: "POST",
      signal: abort.signal,
      headers: {
        Authorization: `Bearer ${key}`,
        "Content-Type": "application/json",
        "X-Title": "Ulana example",
      },
      body: JSON.stringify({
        model: MODEL,
        messages,
        temperature: 0.2,
        max_tokens: 8000,
        reasoning: { effort: "low" },
        usage: { include: true },
      }),
    });
    const body = (await r.json().catch(() => null)) as {
      error?: { message?: string };
      choices?: { message?: { content?: string | null } }[];
      usage?: { cost?: number };
    } | null;
    if (r.status === 401) throw new Error("OpenRouter refused the key. Connect it again.");
    if (r.status === 402) throw new Error("The OpenRouter account is out of credits.");
    if (!r.ok) throw new Error(body?.error?.message ?? `OpenRouter answered ${r.status}.`);
    const content = body?.choices?.[0]?.message?.content ?? "";
    if (!content.trim()) throw new Error("The model sent an empty answer. Try again.");
    return { content, cost: typeof body?.usage?.cost === "number" ? body.usage.cost : null };
  } catch (e) {
    if (abort.signal.aborted) throw new Error("The model took more than two minutes. Try again.");
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

const secs = (since: number) => Math.round((Date.now() - since) / 100) / 10;
