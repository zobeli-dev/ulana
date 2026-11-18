// Draws the views `ulana serve` runs, with a chat that asks a model to
// change them. Saving a .luau file by hand changes the screen too; either
// way the view keeps its state.
//
//   cargo run -p ulana-cli -- serve examples/views --edit --block calendar --action send --image https://picsum.photos/
//   cd examples/expo && bun run web
//
// Settings are in .env (see .env.example).

import { useCallback, useEffect, useState } from "react";
import { Platform, Pressable, ScrollView, StyleSheet, Text, View, useWindowDimensions } from "react-native";
import { SafeAreaProvider, SafeAreaView } from "react-native-safe-area-context";
import { StatusBar } from "expo-status-bar";
import { UlanaView, defaultTheme, useServerView, useServerViews } from "@zobeli/ulana-react-native";
import type { CustomBlockProps, Json } from "@zobeli/ulana-react-native";

import { Chat } from "./src/Chat";
import { ionicons } from "./src/icons";
import { SERVER, getHost, putSource, type HostInfo } from "./src/server";

const theme = defaultTheme;
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/** Where a new view starts. */
const STARTER = `-- A new view. Ask the chat to build something here.
return function(data)
	return ui.card{ title = "New view",
		ui.text{ "Tell the chat what this screen should show.", tone = "muted" },
	}
end
`;

export default function App() {
  return (
    <SafeAreaProvider>
      <StatusBar style="dark" />
      <SafeAreaView style={styles.screen} edges={["top", "left", "right", "bottom"]}>
        <Example />
      </SafeAreaView>
    </SafeAreaProvider>
  );
}

function Example() {
  const { width } = useWindowDimensions();
  const wide = width >= 900;
  const views = useServerViews(SERVER);
  const host = useHost();
  const [picked, setPicked] = useState<string | null>(null);
  const [chatOpen, setChatOpen] = useState(true);
  const name = picked && views.names.includes(picked) ? picked : (views.names[0] ?? picked);
  const live = useServerView(SERVER, name);
  const [failed, setFailed] = useState<string | null>(null);

  async function newView() {
    const taken = new Set(views.names);
    let n = 1;
    while (taken.has(`view-${n}`)) n++;
    const fresh = `view-${n}`;
    try {
      await putSource(fresh, STARTER);
      views.refresh();
      setPicked(fresh);
    } catch (e) {
      setFailed(e instanceof Error ? e.message : String(e));
    }
  }

  const chat = (
    <Chat
      view={name}
      host={host}
      error={live.error}
      onSnapshot={live.accept}
      style={wide ? styles.side : styles.bottom}
    />
  );

  return (
    <View style={styles.screen}>
      <View style={styles.header}>
        <View style={{ flex: 1 }}>
          <Text style={styles.title}>Ulana</Text>
          <View style={styles.server}>
            <View style={[styles.dot, { backgroundColor: views.offline ? theme.tones.danger.fg : theme.tones.success.fg }]} />
            <Text style={styles.caption} numberOfLines={1}>
              {SERVER}
            </Text>
          </View>
        </View>
        {!wide ? (
          <Pressable accessibilityRole="button" onPress={() => setChatOpen(!chatOpen)} style={styles.headButton}>
            <Text style={styles.headButtonText}>{chatOpen ? "Hide chat" : "Chat"}</Text>
          </Pressable>
        ) : null}
      </View>

      {views.offline ? (
        <View style={styles.inset}>
          <Notice tone="danger" title="Cannot reach ulana serve">
            {`${views.offline}\n\nStart it with:\ncargo run -p ulana-cli -- serve examples/views --edit --block calendar --action send --image https://picsum.photos/`}
          </Notice>
        </View>
      ) : null}

      <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.tabs} style={styles.tabsBar}>
        {views.names.map((v) => (
          <Pressable
            key={v}
            accessibilityRole="tab"
            accessibilityState={{ selected: v === name }}
            onPress={() => setPicked(v)}
            style={[styles.tab, v === name && styles.tabOn]}
          >
            <Text style={[styles.tabText, v === name && styles.tabTextOn]}>{v}</Text>
          </Pressable>
        ))}
        {host?.edit ? (
          <Pressable accessibilityRole="button" onPress={() => void newView()} style={[styles.tab, styles.tabNew]}>
            <Text style={styles.tabText}>+ New view</Text>
          </Pressable>
        ) : null}
      </ScrollView>

      <View style={[styles.main, wide && { flexDirection: "row" }]}>
        <Live live={live} failed={failed} />
        {wide || chatOpen ? chat : null}
      </View>
    </View>
  );
}

type LiveState = ReturnType<typeof useServerView>;

/** Draws the picked view. */
function Live({ live, failed }: { live: LiveState; failed: string | null }) {
  const { snapshot, send } = live;
  const offline = live.offline ?? failed;
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
    <View style={{ flex: 1 }}>
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
            icons={ionicons}
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
    </View>
  );
}

/** What the server says about itself: the view API and what it accepts. */
function useHost() {
  const [host, setHost] = useState<HostInfo | null>(null);
  useEffect(() => {
    let alive = true;
    (async () => {
      while (alive) {
        try {
          const h = await getHost();
          if (alive) setHost(h);
          return;
        } catch {
          await sleep(2000);
        }
      }
    })();
    return () => {
      alive = false;
    };
  }, []);
  return host;
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
  header: { flexDirection: "row", alignItems: "center", paddingHorizontal: 16, paddingTop: 8, paddingBottom: 4, gap: 8 },
  title: { fontSize: 22, fontWeight: "600", color: theme.colors.ink },
  server: { flexDirection: "row", alignItems: "center", gap: 6 },
  dot: { width: 8, height: 8, borderRadius: 4 },
  caption: { fontSize: 13, lineHeight: 18, color: theme.tones.muted.fg, flexShrink: 1 },
  headButton: { minHeight: 44, justifyContent: "center", paddingHorizontal: 12 },
  headButtonText: { fontSize: 15, color: theme.tones.accent.fg, fontWeight: "500" },
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
  tabNew: { borderStyle: "dashed" },
  tabText: { fontSize: 14, color: theme.colors.ink },
  tabTextOn: { color: theme.colors.surface },
  main: { flex: 1 },
  side: { width: 380, borderLeftWidth: StyleSheet.hairlineWidth },
  bottom: { height: "48%", borderTopWidth: StyleSheet.hairlineWidth },
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
