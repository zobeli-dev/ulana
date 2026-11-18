// Talking to `ulana serve`.

import { Platform } from "react-native";
import type { Frame, Json } from "@zobeli/ulana-react-native";

export const SERVER =
  process.env.EXPO_PUBLIC_ULANA_URL ??
  (Platform.OS === "android" ? "http://10.0.2.2:8790" : "http://127.0.0.1:8790");

/** What the server answers for one view. */
export type Snapshot = {
  name: string;
  /** Goes up whenever the frame, the error, or the logs change. */
  revision: number;
  /** The last frame that rendered; it stays while the script has an error. */
  frame: Frame | null;
  error: string | null;
  logs: string[];
};

export type Listed = { name: string; revision: number };

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

export const listViews = () => call<Listed[]>(`${SERVER}/views`);

/** The view now, or after it changes when `since` is its current revision. */
export const getSnapshot = (name: string, since: number, signal: AbortSignal) =>
  call<Snapshot>(since >= 0 ? `${view(name)}?since=${since}` : view(name), { signal });

export const sendEvent = (name: string, version: number, id: string, value: Json) =>
  call<Snapshot>(`${view(name)}/events`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ version, id, value }),
  });

export const getHost = () => call<HostInfo>(`${SERVER}/host`);

export const getSource = (name: string) => call<Source>(`${view(name)}/source`);

/** Write a view's script. The server loads it at once and answers with the result. */
export const putSource = (name: string, source: string) =>
  call<Snapshot>(`${view(name)}/source`, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ source }),
  });
