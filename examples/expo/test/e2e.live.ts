// A live run against OpenRouter, not part of `bun test`. It asks for a
// change to each example view and prints what happened. Point it at a copy
// of the views, because it rewrites them:
//
//   cp -r examples/views /tmp/ulana-views
//   cargo run -p ulana-cli -- serve /tmp/ulana-views --bind 127.0.0.1:8792 --edit --block calendar --action send
//   OPENROUTER_API_KEY=… EXPO_PUBLIC_ULANA_URL=http://127.0.0.1:8792 bun --preload ./test/setup.ts run test/e2e.live.ts

import { MODEL, changeView } from "../src/ai";
import { getHost } from "../src/server";

const key = process.env.OPENROUTER_API_KEY;
if (!key) throw new Error("Set OPENROUTER_API_KEY.");
const host = await getHost();
console.log(`model: ${MODEL}`);
for (const [name, request] of [
  ["counter", "Make the number huge and orange, and add a reset button under the others."],
  ["agenda", "Show each event as its own card with the time as a badge, and drop the compact switch."],
  ["todo", "Add a short line at the top that says how many tasks are left, in green when none are."],
] as const) {
  const change = await changeView({
    key,
    host,
    name,
    request,
    history: [],
    error: null,
    tones: { default: "#18181B", muted: "#71717A", accent: "#2563EB", success: "#15803D", warning: "#B45309", danger: "#B91C1C" },
    onStatus: (s) => console.log(`  … ${s}`),
  });
  console.log(`${name}: ${change.reply}`);
  console.log(
    `  tries ${change.tries} · ${change.seconds} s · $${change.cost?.toFixed(5) ?? "?"} · error: ${change.error ?? "none"}`,
  );
}
