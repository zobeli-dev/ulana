// Draws the views `ulana serve` runs. Save a .luau file and the screen
// changes, keeping its state.
//
//   cargo run -p ulana-cli -- serve examples/views --block calendar --action send
//   cd examples/expo && bun run start
//
// On a phone, serve with `--bind 0.0.0.0:8790` and set EXPO_PUBLIC_ULANA_URL
// to this machine's address on the network, such as http://192.168.1.20:8790.

import { useCallback, useEffect, useRef, useState } from "react";
import { Platform, Pressable, ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaProvider, SafeAreaView } from "react-native-safe-area-context";
import { StatusBar } from "expo-status-bar";
import { UlanaView, defaultTheme, isFrame } from "@zobeli/ulana-react-native";
import type { CustomBlockProps, Frame, Json } from "@zobeli/ulana-react-native";

const SERVER =
  process.env.EXPO_PUBLIC_ULANA_URL ??
  (Platform.OS === "android" ? "http://10.0.2.2:8790" : "http://127.0.0.1:8790");

/** What `ulana serve` answers for one view. */
type Snapshot = {
  name: string;
  revision: number;
  frame: Frame | null;
  error: string | null;
  logs: string[];
};

type Listed = { name: string; revision: number };

const theme = defaultTheme;
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export default function App() {
  return (
    <SafeAreaProvider>
      <StatusBar style="dark" />
      <SafeAreaView style={styles.screen} edges={["top", "left", "right"]}>
        <Example />
      </SafeAreaView>
    </SafeAreaProvider>
  );
}

function Example() {
  const views = useViews();
  const [picked, setPicked] = useState<string | null>(null);
  const name = picked && views.list.some((v) => v.name === picked) ? picked : (views.list[0]?.name ?? null);

  return (
    <View style={styles.screen}>
      <View style={styles.header}>
        <Text style={styles.title}>Ulana</Text>
        <View style={styles.server}>
          <View style={[styles.dot, { backgroundColor: views.offline ? theme.tones.danger.fg : theme.tones.success.fg }]} />
          <Text style={styles.caption} numberOfLines={1}>
            {SERVER}
          </Text>
        </View>
      </View>

      {views.offline ? (
        <View style={styles.inset}>
          <Notice tone="danger" title="Cannot reach ulana serve">
            {`${views.offline}\n\nStart it with:\ncargo run -p ulana-cli -- serve examples/views --block calendar --action send`}
          </Notice>
        </View>
      ) : null}

      <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.tabs} style={styles.tabsBar}>
        {views.list.map((v) => (
          <Pressable
            key={v.name}
            accessibilityRole="tab"
            accessibilityState={{ selected: v.name === name }}
            onPress={() => setPicked(v.name)}
            style={[styles.tab, v.name === name && styles.tabOn]}
          >
            <Text style={[styles.tabText, v.name === name && styles.tabTextOn]}>{v.name}</Text>
          </Pressable>
        ))}
      </ScrollView>

      {name ? <Live key={name} name={name} /> : null}
    </View>
  );
}

/** Draws one view and keeps it in step with the server. */
function Live({ name }: { name: string }) {
  const { snapshot, offline, send } = useSnapshot(name);
  const [said, setSaid] = useState<string | null>(null);

  useEffect(() => {
    if (!said) return;
    const t = setTimeout(() => setSaid(null), 3000);
    return () => clearTimeout(t);
  }, [said]);

  const onHostAction = useCallback((action: string, value: Json) => {
    setSaid(`${action}: ${typeof value === "string" ? value : JSON.stringify(value)}`);
  }, []);

  return (
    <>
      <ScrollView contentContainerStyle={styles.body} keyboardShouldPersistTaps="handled">
        {offline ? <Notice tone="warning" title="Reconnecting">{offline}</Notice> : null}
        {snapshot?.error ? <Notice tone="danger" title="The view has an error">{snapshot.error}</Notice> : null}
        {snapshot?.frame ? (
          <UlanaView
            frame={snapshot.frame}
            theme={theme}
            onEvent={send}
            onHostAction={onHostAction}
            blocks={{ calendar: CalendarBlock }}
          />
        ) : snapshot && !snapshot.error ? (
          <Text style={styles.caption}>Nothing to show yet.</Text>
        ) : null}
        {snapshot && snapshot.logs.length > 0 ? <Logs lines={snapshot.logs} /> : null}
      </ScrollView>
      {said ? (
        <View style={styles.toast} accessibilityLiveRegion="polite">
          <Text style={styles.toastText}>{`Host action · ${said}`}</Text>
        </View>
      ) : null}
    </>
  );
}

/** The list of views, read again every few seconds so new files show up. */
function useViews() {
  const [list, setList] = useState<Listed[]>([]);
  const [offline, setOffline] = useState<string | null>(null);
  useEffect(() => {
    let alive = true;
    (async () => {
      while (alive) {
        try {
          const r = await fetch(`${SERVER}/views`);
          if (!r.ok) throw new Error(`HTTP ${r.status}`);
          const views = (await r.json()) as Listed[];
          if (alive) {
            setList(views);
            setOffline(null);
          }
        } catch (e) {
          if (alive) setOffline(String(e));
        }
        await sleep(2000);
      }
    })();
    return () => {
      alive = false;
    };
  }, []);
  return { list, offline };
}

/**
 * The latest snapshot of one view. A request waits on the server until the
 * view changes, so a saved file shows up at once.
 */
function useSnapshot(name: string) {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [offline, setOffline] = useState<string | null>(null);
  const latest = useRef(-1);

  const accept = useCallback((s: Snapshot) => {
    if (s.revision < latest.current) return;
    if (s.frame !== null && !isFrame(s.frame)) throw new Error("the server sent something that is not a frame");
    latest.current = s.revision;
    setSnapshot(s);
  }, []);

  useEffect(() => {
    let alive = true;
    const abort = new AbortController();
    (async () => {
      while (alive) {
        try {
          const since = latest.current >= 0 ? `?since=${latest.current}` : "";
          const r = await fetch(`${SERVER}/views/${encodeURIComponent(name)}${since}`, { signal: abort.signal });
          if (!r.ok) throw new Error(`HTTP ${r.status}`);
          accept((await r.json()) as Snapshot);
          setOffline(null);
        } catch (e) {
          if (!alive) return;
          setOffline(String(e));
          await sleep(1000);
        }
      }
    })();
    return () => {
      alive = false;
      abort.abort();
    };
  }, [name, accept]);

  const send = useCallback(
    (version: number, id: string, value: Json) => {
      fetch(`${SERVER}/views/${encodeURIComponent(name)}/events`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ version, id, value }),
      })
        .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
        .then((s) => accept(s as Snapshot))
        .catch((e: unknown) => setOffline(String(e)));
    },
    [name, accept],
  );

  return { snapshot, offline, send };
}

/** Stands in for the app's own calendar, to show a `custom` block. */
function CalendarBlock({ props, theme: t }: CustomBlockProps) {
  return (
    <View style={[styles.custom, { borderColor: t.tones.accent.fg, backgroundColor: t.tones.accent.soft }]}>
      <Text style={[styles.customTitle, { color: t.tones.accent.fg }]}>Calendar block from the app</Text>
      <Text style={styles.mono}>{JSON.stringify(props)}</Text>
    </View>
  );
}

function Notice({ tone, title, children }: { tone: "danger" | "warning"; title: string; children: string }) {
  const colors = theme.tones[tone];
  return (
    <View style={[styles.notice, { backgroundColor: colors.soft }]} accessibilityRole="alert">
      <Text style={[styles.noticeTitle, { color: colors.fg }]}>{title}</Text>
      <Text style={[styles.mono, { color: colors.fg }]} selectable>
        {children}
      </Text>
    </View>
  );
}

function Logs({ lines }: { lines: string[] }) {
  const [open, setOpen] = useState(false);
  return (
    <View style={styles.logs}>
      <Pressable accessibilityRole="button" onPress={() => setOpen(!open)} style={styles.logsHead}>
        <Text style={styles.caption}>{`${open ? "Hide" : "Show"} print output (${lines.length})`}</Text>
      </Pressable>
      {open ? (
        <Text style={styles.mono} selectable>
          {lines.join("\n")}
        </Text>
      ) : null}
    </View>
  );
}

const mono = Platform.select({ ios: "Menlo", android: "monospace", default: "ui-monospace, Menlo, monospace" });

const styles = StyleSheet.create({
  screen: { flex: 1, backgroundColor: theme.colors.canvas },
  header: { paddingHorizontal: 16, paddingTop: 8, paddingBottom: 4, gap: 2 },
  title: { fontSize: 22, fontWeight: "600", color: theme.colors.ink },
  server: { flexDirection: "row", alignItems: "center", gap: 6 },
  dot: { width: 8, height: 8, borderRadius: 4 },
  caption: { fontSize: 13, lineHeight: 18, color: theme.tones.muted.fg, flexShrink: 1 },
  tabsBar: { flexGrow: 0 },
  tabs: { paddingHorizontal: 16, paddingVertical: 8, gap: 8 },
  tab: {
    minHeight: 36,
    paddingHorizontal: 14,
    borderRadius: 18,
    justifyContent: "center",
    backgroundColor: theme.colors.surface,
    borderWidth: 1,
    borderColor: theme.colors.border,
  },
  tabOn: { backgroundColor: theme.colors.ink, borderColor: theme.colors.ink },
  tabText: { fontSize: 14, color: theme.colors.ink },
  tabTextOn: { color: theme.colors.surface },
  body: { padding: 16, gap: 12, paddingBottom: 48 },
  inset: { paddingHorizontal: 16, paddingTop: 8 },
  notice: { borderRadius: 12, padding: 12, gap: 4 },
  noticeTitle: { fontSize: 14, fontWeight: "600" },
  mono: { fontFamily: mono, fontSize: 12, lineHeight: 17, color: theme.colors.ink },
  custom: { borderWidth: 1, borderStyle: "dashed", borderRadius: 12, padding: 12, gap: 4 },
  customTitle: { fontSize: 14, fontWeight: "600" },
  logs: { borderTopWidth: StyleSheet.hairlineWidth, borderTopColor: theme.colors.border, paddingTop: 8, gap: 6 },
  logsHead: { minHeight: 44, justifyContent: "center" },
  toast: {
    position: "absolute",
    left: 16,
    right: 16,
    bottom: 24,
    padding: 12,
    borderRadius: 12,
    backgroundColor: theme.colors.ink,
  },
  toastText: { color: theme.colors.surface, fontSize: 14 },
});
