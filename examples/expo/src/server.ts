// The parts of `ulana serve` the chat needs beyond following a view (which
// `useServerView` from the package does): the view API, and reading and
// writing a script.

import { Platform } from "react-native";
import type { Json, Snapshot } from "@zobeli/ulana-react-native";

export const SERVER =
  process.env.EXPO_PUBLIC_ULANA_URL ??
  (Platform.OS === "android" ? "http://10.0.2.2:8790" : "http://127.0.0.1:8790");

/** What a model needs to write a view for this app. */
export type HostInfo = { spec: string; blocks: string[]; actions: string[]; images?: string[]; edit: boolean };

export type Source = { name: string; source: string; data: Json };

const view = (name: string) => `${SERVER}/views/${encodeURIComponent(name)}`;

async function call<T>(url: string, init?: RequestInit): Promise<T> {
  const r = await fetch(url, init);
  if (!r.ok) {
    const body = (await r.json().catch(() => null)) as { error?: string } | null;
    throw new Error(body?.error ?? `HTTP ${r.status}`);
  }
  return (await r.json()) as T;
}

export const getHost = () => call<HostInfo>(`${SERVER}/host`);

export const getSource = (name: string) => call<Source>(`${view(name)}/source`);

/** Write a view's script. The server loads it at once and answers with the result. */
export const putSource = (name: string, source: string) =>
  call<Snapshot>(`${view(name)}/source`, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ source }),
  });
