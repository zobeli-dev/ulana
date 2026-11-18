// Following views that run on a server speaking the `ulana serve` protocol:
//
//   GET  /views                       the views and their revisions
//   GET  /views/<name>?since=<rev>    the view, after it changes (long poll)
//   POST /views/<name>/events         { version, id, value }

import { useCallback, useEffect, useRef, useState } from "react";
import type { ComponentType } from "react";
import { Text, View } from "react-native";

import { isFrame } from "./frame";
import { defaultTheme, type Theme } from "./theme";
import type { Frame, Json } from "./types";
import { UlanaView, type CustomBlockProps, type IconRenderer } from "./UlanaView";

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

export type ServerView = {
  snapshot: Snapshot | null;
  frame: Frame | null;
  /** The script's error, written to be read by a person or a model. */
  error: string | null;
  logs: string[];
  /** Set while the server cannot be reached; the session keeps trying. */
  offline: string | null;
  /** Pass to `UlanaView`'s `onEvent`. */
  send: (version: number, id: string, value: Json) => void;
  /** Take a snapshot from elsewhere, such as the answer to writing a script. */
  accept: (snapshot: Snapshot) => void;
};

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const trim = (url: string) => url.replace(/\/+$/, "");

/**
 * Follow one view on a server. The request waits on the server until the
 * view changes, so a saved script shows up at once.
 */
export function useServerView(url: string, name: string | null): ServerView {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [offline, setOffline] = useState<string | null>(null);
  const latest = useRef(-1);
  const current = useRef(name);
  current.current = name;
  const base = trim(url);

  const accept = useCallback((s: Snapshot) => {
    if (s.name !== current.current || s.revision < latest.current) return;
    if (s.frame !== null && !isFrame(s.frame)) return;
    latest.current = s.revision;
    setSnapshot(s);
  }, []);

  useEffect(() => {
    latest.current = -1;
    setSnapshot(null);
    if (!name) return;
    let alive = true;
    const abort = new AbortController();
    const view = `${base}/views/${encodeURIComponent(name)}`;
    (async () => {
      while (alive) {
        try {
          const since = latest.current >= 0 ? `?since=${latest.current}` : "";
          const r = await fetch(view + since, { signal: abort.signal });
          if (!r.ok) throw new Error(await message(r));
          accept((await r.json()) as Snapshot);
          setOffline(null);
        } catch (e) {
          if (!alive) return;
          setOffline(e instanceof Error ? e.message : String(e));
          await sleep(1000);
        }
      }
    })();
    return () => {
      alive = false;
      abort.abort();
    };
  }, [base, name, accept]);

  const send = useCallback(
    (version: number, id: string, value: Json) => {
      if (!name) return;
      fetch(`${base}/views/${encodeURIComponent(name)}/events`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ version, id, value }),
      })
        .then(async (r) => (r.ok ? ((await r.json()) as Snapshot) : Promise.reject(new Error(await message(r)))))
        .then(accept)
        .catch((e: unknown) => setOffline(e instanceof Error ? e.message : String(e)));
    },
    [base, name, accept],
  );

  return {
    snapshot,
    frame: snapshot?.frame ?? null,
    error: snapshot?.error ?? null,
    logs: snapshot?.logs ?? [],
    offline,
    send,
    accept,
  };
}

/** The views a server has, read again every `everyMs` so new ones show up. */
export function useServerViews(url: string, everyMs = 2000): { names: string[]; offline: string | null; refresh: () => void } {
  const [names, setNames] = useState<string[]>([]);
  const [offline, setOffline] = useState<string | null>(null);
  const [tick, setTick] = useState(0);
  const base = trim(url);
  useEffect(() => {
    let alive = true;
    (async () => {
      while (alive) {
        try {
          const r = await fetch(`${base}/views`);
          if (!r.ok) throw new Error(await message(r));
          const list = (await r.json()) as { name: string }[];
          if (alive) {
            setNames(list.map((v) => v.name));
            setOffline(null);
          }
        } catch (e) {
          if (alive) setOffline(e instanceof Error ? e.message : String(e));
        }
        await sleep(everyMs);
      }
    })();
    return () => {
      alive = false;
    };
  }, [base, everyMs, tick]);
  return { names, offline, refresh: () => setTick((t) => t + 1) };
}

export type UlanaServerViewProps = {
  /** Where the server is, such as `http://127.0.0.1:8790`. */
  url: string;
  /** The view to draw. */
  name: string;
  theme?: Theme;
  onHostAction?: (name: string, value: Json) => void;
  blocks?: Record<string, ComponentType<CustomBlockProps>>;
  icons?: IconRenderer;
  /** Show the script's error above the view. Turn it off in production. Default true. */
  showErrors?: boolean;
};

/**
 * One view from a server, kept up to date, with its error shown above the
 * last frame that worked.
 *
 * ```tsx
 * <UlanaServerView url="http://127.0.0.1:8790" name="counter" />
 * ```
 */
export function UlanaServerView({ url, name, theme = defaultTheme, showErrors = true, ...rest }: UlanaServerViewProps) {
  const view = useServerView(url, name);
  const note = (text: string, tone: "danger" | "muted") => (
    <View style={{ backgroundColor: theme.tones[tone].soft, borderRadius: theme.radius.control, padding: theme.space.sm }}>
      <Text style={{ color: theme.tones[tone].fg, fontSize: theme.text.caption.fontSize, fontFamily: theme.fonts.mono }}>{text}</Text>
    </View>
  );
  return (
    <View style={{ gap: theme.space.md }}>
      {showErrors && view.error ? note(view.error, "danger") : null}
      {view.frame ? (
        <UlanaView frame={view.frame} theme={theme} onEvent={view.send} {...rest} />
      ) : view.offline ? (
        note(`Cannot reach ${url}: ${view.offline}`, "muted")
      ) : null}
    </View>
  );
}

async function message(r: Response): Promise<string> {
  const body = (await r.json().catch(() => null)) as { error?: string } | null;
  return body?.error ?? `HTTP ${r.status}`;
}
