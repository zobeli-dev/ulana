// The chat panel: connect an OpenRouter key, ask for a change, and see the
// view change. Each answer that changed the view can be undone.

import { useEffect, useRef, useState } from "react";
import { ActivityIndicator, Linking, Platform, Pressable, ScrollView, StyleSheet, Text, TextInput, View } from "react-native";
import type { NativeSyntheticEvent, TextInputKeyPressEventData, ViewStyle } from "react-native";
import { defaultTheme as theme } from "@zobeli/ulana-react-native";

import { MODEL, changeView, checkKey, type Turn } from "./ai";
import { forgetKey, loadKey, saveKey } from "./keyStore";
import type { Snapshot } from "@zobeli/ulana-react-native";

import { putSource, type HostInfo } from "./server";

type Line =
  | { kind: "user"; id: number; text: string; view: string }
  | {
      kind: "ai";
      id: number;
      text: string;
      view: string;
      change: null | {
        error: string | null;
        tries: number;
        seconds: number;
        cost: number | null;
        previous: string;
        undone: boolean;
      };
    }
  | { kind: "note"; id: number; text: string; tone: "danger" | "muted" };

/** A line before it has an id. */
type NewLine = Line extends infer L ? (L extends Line ? Omit<L, "id"> : never) : never;

const SUGGESTIONS = [
  "Make it feel warmer and more spacious",
  "Add a button that resets everything",
  "Show a short summary at the top",
  "Put each item in its own card",
];

let nextId = 1;

/** The theme's colours, so the model can match "orange" to a tone. */
const TONES = Object.fromEntries(Object.entries(theme.tones).map(([tone, c]) => [tone, c.fg]));

export function Chat(props: {
  view: string | null;
  host: HostInfo | null;
  error: string | null;
  onSnapshot: (s: Snapshot) => void;
  style?: ViewStyle;
}) {
  const [key, setKey] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [lines, setLines] = useState<Line[]>([]);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const scroll = useRef<ScrollView>(null);

  useEffect(() => {
    loadKey().then((k) => {
      setKey(k);
      setLoaded(true);
    });
  }, []);

  const add = (line: NewLine) => setLines((all) => [...all, { ...line, id: nextId++ } as Line]);

  const history = (): Turn[] =>
    lines.flatMap((l): Turn[] =>
      l.kind === "user" ? [{ role: "user", content: l.text }] : l.kind === "ai" ? [{ role: "assistant", content: l.text }] : [],
    );

  async function send(text: string) {
    const request = text.trim();
    if (!request || busy || !key || !props.view || !props.host) return;
    const view = props.view;
    setDraft("");
    add({ kind: "user", text: request, view });
    setBusy("Thinking…");
    try {
      const change = await changeView({
        key,
        host: props.host,
        name: view,
        request,
        history: history(),
        error: props.error,
        tones: TONES,
        onStatus: setBusy,
      });
      if (change.snapshot) props.onSnapshot(change.snapshot);
      add({
        kind: "ai",
        text: change.reply,
        view,
        change: change.changed
          ? {
              error: change.error,
              tries: change.tries,
              seconds: change.seconds,
              cost: change.cost,
              previous: change.previous,
              undone: false,
            }
          : null,
      });
    } catch (e) {
      add({ kind: "note", text: e instanceof Error ? e.message : String(e), tone: "danger" });
    } finally {
      setBusy(null);
    }
  }

  async function undo(line: Extract<Line, { kind: "ai" }>) {
    if (!line.change || line.change.undone || busy) return;
    setBusy("Putting it back…");
    try {
      props.onSnapshot(await putSource(line.view, line.change.previous));
      setLines((all) =>
        all.map((l) => (l.id === line.id && l.kind === "ai" && l.change ? { ...l, change: { ...l.change, undone: true } } : l)),
      );
    } catch (e) {
      add({ kind: "note", text: e instanceof Error ? e.message : String(e), tone: "danger" });
    } finally {
      setBusy(null);
    }
  }

  const onKeyPress = (e: NativeSyntheticEvent<TextInputKeyPressEventData>) => {
    const ne = e.nativeEvent as TextInputKeyPressEventData & { shiftKey?: boolean };
    if (Platform.OS === "web" && ne.key === "Enter" && !ne.shiftKey) {
      e.preventDefault();
      void send(draft);
    }
  };

  const ready = !!key && !!props.view && !!props.host?.edit;

  return (
    <View style={[styles.panel, props.style]}>
      <View style={styles.head}>
        <View style={{ flex: 1 }}>
          <Text style={styles.title}>Chat</Text>
          <Text style={styles.caption} numberOfLines={1}>
            {props.view ? `Changes “${props.view}” · ${MODEL}` : MODEL}
          </Text>
        </View>
        {key ? (
          <Pressable
            accessibilityRole="button"
            onPress={() => {
              void forgetKey();
              setKey(null);
            }}
            style={styles.small}
          >
            <Text style={styles.smallText}>Forget key</Text>
          </Pressable>
        ) : null}
      </View>

      {!loaded ? null : !key ? (
        <Connect onKey={setKey} />
      ) : (
        <>
          <ScrollView
            ref={scroll}
            style={{ flex: 1 }}
            contentContainerStyle={styles.lines}
            onContentSizeChange={() => scroll.current?.scrollToEnd({ animated: true })}
            keyboardShouldPersistTaps="handled"
          >
            {props.host && !props.host.edit ? (
              <Text style={[styles.note, { color: theme.tones.danger.fg }]}>
                {"The server does not let the chat write views. Start it with --edit:\ncargo run -p ulana-cli -- serve examples/views --edit --block calendar --action send --image https://picsum.photos/"}
              </Text>
            ) : null}
            {lines.length === 0 ? (
              <View style={{ gap: 8 }}>
                <Text style={styles.caption}>Ask for any change to the view on the left. The model rewrites its Luau script, and the screen changes.</Text>
                <View style={styles.chips}>
                  {SUGGESTIONS.map((s) => (
                    <Pressable key={s} accessibilityRole="button" disabled={!ready} onPress={() => void send(s)} style={styles.chip}>
                      <Text style={styles.chipText}>{s}</Text>
                    </Pressable>
                  ))}
                </View>
              </View>
            ) : null}
            {lines.map((line) => (
              <LineView key={line.id} line={line} onUndo={undo} disabled={!!busy} />
            ))}
            {busy ? (
              <View style={styles.busy}>
                <ActivityIndicator size="small" color={theme.tones.muted.fg} />
                <Text style={styles.caption}>{busy}</Text>
              </View>
            ) : null}
          </ScrollView>
          <View style={styles.composer}>
            <TextInput
              value={draft}
              onChangeText={setDraft}
              onKeyPress={onKeyPress}
              placeholder={ready ? "Make the count huge and orange…" : "Pick a view to change"}
              placeholderTextColor={theme.tones.muted.fg}
              editable={ready && !busy}
              multiline
              style={styles.input}
              accessibilityLabel="Your request"
            />
            <Pressable
              accessibilityRole="button"
              accessibilityLabel="Send"
              disabled={!ready || !!busy || !draft.trim()}
              onPress={() => void send(draft)}
              style={[styles.send, (!ready || !!busy || !draft.trim()) && { opacity: 0.4 }]}
            >
              <Text style={styles.sendText}>Send</Text>
            </Pressable>
          </View>
        </>
      )}
    </View>
  );
}

function LineView({ line, onUndo, disabled }: { line: Line; onUndo: (l: Extract<Line, { kind: "ai" }>) => void; disabled: boolean }) {
  if (line.kind === "user") {
    return (
      <View style={styles.user}>
        <Text style={styles.userText}>{line.text}</Text>
      </View>
    );
  }
  if (line.kind === "note") {
    return <Text style={[styles.note, { color: line.tone === "danger" ? theme.tones.danger.fg : theme.tones.muted.fg }]}>{line.text}</Text>;
  }
  const c = line.change;
  const meta = c
    ? [
        c.error ? "The script still has an error" : c.tries > 1 ? `Fixed itself after ${c.tries - 1} error${c.tries > 2 ? "s" : ""}` : `Changed “${line.view}”`,
        `${c.seconds} s`,
        c.cost !== null ? `$${c.cost.toFixed(4)}` : null,
      ]
        .filter(Boolean)
        .join(" · ")
    : null;
  return (
    <View style={styles.ai}>
      <Text style={styles.aiText}>{line.text}</Text>
      {c ? (
        <View style={styles.meta}>
          <Text style={[styles.caption, c.error ? { color: theme.tones.danger.fg } : null]}>{meta}</Text>
          {c.undone ? (
            <Text style={styles.caption}>Undone</Text>
          ) : (
            <Pressable accessibilityRole="button" disabled={disabled} onPress={() => onUndo(line)} style={styles.small}>
              <Text style={styles.smallText}>Undo</Text>
            </Pressable>
          )}
        </View>
      ) : null}
    </View>
  );
}

function Connect({ onKey }: { onKey: (key: string) => void }) {
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);

  async function connect() {
    const key = value.trim();
    if (!key || checking) return;
    setChecking(true);
    setError(null);
    try {
      await checkKey(key);
      await saveKey(key);
      onKey(key);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setChecking(false);
    }
  }

  return (
    <View style={styles.connect}>
      <Text style={styles.connectTitle}>Connect OpenRouter</Text>
      <Text style={styles.caption}>
        {`The chat asks ${MODEL} through your OpenRouter key. The key is kept on this device and sent only to OpenRouter.`}
      </Text>
      <TextInput
        value={value}
        onChangeText={setValue}
        onSubmitEditing={() => void connect()}
        placeholder="sk-or-…"
        placeholderTextColor={theme.tones.muted.fg}
        secureTextEntry
        autoCapitalize="none"
        autoCorrect={false}
        autoComplete="off"
        textContentType="none"
        style={styles.keyInput}
        accessibilityLabel="OpenRouter API key"
      />
      {error ? <Text style={[styles.caption, { color: theme.tones.danger.fg }]}>{error}</Text> : null}
      <View style={styles.row}>
        <Pressable
          accessibilityRole="button"
          disabled={!value.trim() || checking}
          onPress={() => void connect()}
          style={[styles.send, (!value.trim() || checking) && { opacity: 0.4 }]}
        >
          <Text style={styles.sendText}>{checking ? "Checking…" : "Connect"}</Text>
        </Pressable>
        <Pressable accessibilityRole="link" onPress={() => void Linking.openURL("https://openrouter.ai/keys")} style={styles.small}>
          <Text style={styles.smallText}>Get a key</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  panel: { backgroundColor: theme.colors.surface, borderColor: theme.colors.border },
  head: { flexDirection: "row", alignItems: "center", gap: 8, paddingHorizontal: 16, paddingTop: 12, paddingBottom: 8 },
  title: { fontSize: 16, fontWeight: "600", color: theme.colors.ink },
  caption: { fontSize: 13, lineHeight: 18, color: theme.tones.muted.fg },
  lines: { padding: 16, gap: 12 },
  chips: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  chip: {
    minHeight: 36,
    justifyContent: "center",
    paddingHorizontal: 12,
    borderRadius: 18,
    borderWidth: 1,
    borderColor: theme.colors.border,
    backgroundColor: theme.colors.canvas,
  },
  chipText: { fontSize: 13, color: theme.colors.ink },
  user: { alignSelf: "flex-end", maxWidth: "88%", backgroundColor: theme.colors.ink, borderRadius: 14, paddingHorizontal: 12, paddingVertical: 8 },
  userText: { fontSize: 15, lineHeight: 21, color: theme.colors.surface },
  ai: { alignSelf: "flex-start", maxWidth: "92%", gap: 4 },
  aiText: { fontSize: 15, lineHeight: 21, color: theme.colors.ink },
  meta: { flexDirection: "row", alignItems: "center", gap: 8, flexWrap: "wrap" },
  note: { fontSize: 13, lineHeight: 18 },
  busy: { flexDirection: "row", alignItems: "center", gap: 8 },
  composer: {
    flexDirection: "row",
    alignItems: "flex-end",
    gap: 8,
    padding: 12,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: theme.colors.border,
  },
  input: {
    flex: 1,
    minHeight: 44,
    maxHeight: 120,
    paddingHorizontal: 12,
    paddingTop: 11,
    paddingBottom: 11,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: theme.colors.border,
    backgroundColor: theme.colors.canvas,
    fontSize: 15,
    color: theme.colors.ink,
  },
  send: {
    minHeight: 44,
    paddingHorizontal: 16,
    borderRadius: 12,
    justifyContent: "center",
    backgroundColor: theme.tones.accent.fg,
  },
  sendText: { color: theme.colors.onAccent, fontSize: 15, fontWeight: "500" },
  small: { minHeight: 32, justifyContent: "center", paddingHorizontal: 8 },
  smallText: { fontSize: 13, color: theme.tones.accent.fg, fontWeight: "500" },
  connect: { padding: 16, gap: 10 },
  connectTitle: { fontSize: 15, fontWeight: "600", color: theme.colors.ink },
  keyInput: {
    minHeight: 44,
    paddingHorizontal: 12,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: theme.colors.border,
    backgroundColor: theme.colors.canvas,
    fontSize: 15,
    color: theme.colors.ink,
  },
  row: { flexDirection: "row", alignItems: "center", gap: 8 },
});
