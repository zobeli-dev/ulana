import { Component, createContext, Fragment, useContext, useEffect, useMemo, useState } from "react";
import type { ComponentType, ReactNode } from "react";
import { Pressable, StyleSheet, Switch, Text, View } from "react-native";
import type { FlexAlignType, TextStyle as RNTextStyle, ViewStyle } from "react-native";

import { defaultTheme, type Theme } from "./theme";
import type {
  Action,
  Align,
  Block,
  ButtonBlock,
  Frame,
  ItemBlock,
  Json,
  Justify,
  TextStyle,
  ToggleBlock,
  Tone,
} from "./types";

/** What a host component for a `custom` block receives. */
export type CustomBlockProps = {
  props: { [key: string]: Json };
  theme: Theme;
};

export type UlanaViewProps = {
  frame: Frame;
  theme?: Theme;
  /** A press or change on a view's own handler. Run it, render again, and pass the new frame. */
  onEvent?: (version: number, id: string, value: Json) => void;
  /** A press on an action the host named, such as `send`. */
  onHostAction?: (name: string, value: Json) => void;
  /** The host's own blocks, by the name a view gives `ui.custom`. */
  blocks?: Record<string, ComponentType<CustomBlockProps>>;
};

type Context = {
  theme: Theme;
  run: (action: Action, value: Json) => void;
  blocks: Record<string, ComponentType<CustomBlockProps>>;
};

const Ctx = createContext<Context | null>(null);

function useUlana(): Context {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("Ulana blocks must be inside <UlanaView>");
  return ctx;
}

/** Draws a frame with React Native's own components. */
export function UlanaView({ frame, theme = defaultTheme, onEvent, onHostAction, blocks = {} }: UlanaViewProps) {
  const ctx = useMemo<Context>(
    () => ({
      theme,
      blocks,
      run: (action, value) => {
        if (action.kind === "event") onEvent?.(frame.version, action.id, value);
        else onHostAction?.(action.name, action.value ?? null);
      },
    }),
    [theme, blocks, frame.version, onEvent, onHostAction],
  );
  return (
    <Ctx.Provider value={ctx}>
      <Guard theme={theme} label="This view could not be drawn.">
        <BlockView block={frame.root} />
      </Guard>
    </Ctx.Provider>
  );
}

function BlockView({ block }: { block: Block }): ReactNode {
  const { theme } = useUlana();
  switch (block.type) {
    case "stack":
      return (
        <View
          style={{
            gap: theme.space[block.gap ?? "none"],
            padding: theme.space[block.padding ?? "none"],
            alignItems: align(block.align ?? "stretch"),
          }}
        >
          <Children blocks={block.children} />
        </View>
      );
    case "row":
      return (
        <View
          style={{
            flexDirection: "row",
            flexWrap: block.wrap ? "wrap" : "nowrap",
            gap: theme.space[block.gap ?? "sm"],
            padding: theme.space[block.padding ?? "none"],
            alignItems: align(block.align ?? "center"),
            justifyContent: justify(block.justify ?? "start"),
          }}
        >
          <Children blocks={block.children} />
        </View>
      );
    case "text":
      return (
        <Text
          style={textStyle(theme, block.style ?? "body", block.tone ?? "default")}
          numberOfLines={block.lines}
          accessibilityRole={block.style === "title" || block.style === "heading" ? "header" : "text"}
        >
          {block.text}
        </Text>
      );
    case "card":
      return (
        <View
          style={{
            backgroundColor: theme.colors.surface,
            borderColor: theme.colors.border,
            borderWidth: StyleSheet.hairlineWidth,
            borderRadius: theme.radius.card,
            padding: theme.space.md,
            gap: theme.space.sm,
          }}
        >
          {block.title ? (
            <Text style={textStyle(theme, "heading", "default")} accessibilityRole="header">
              {block.title}
            </Text>
          ) : null}
          <Children blocks={block.children} />
        </View>
      );
    case "list":
      return (
        <View>
          {block.children.map((child, i) => (
            <Fragment key={i}>
              {i > 0 ? <Line /> : null}
              <BlockView block={child} />
            </Fragment>
          ))}
        </View>
      );
    case "item":
      return <Item block={block} />;
    case "button":
      return <Button block={block} />;
    case "toggle":
      return <Toggle block={block} />;
    case "progress": {
      const tone = theme.tones[block.tone ?? "accent"];
      const value = Math.min(1, Math.max(0, block.value));
      return (
        <View
          style={{ gap: theme.space.xs }}
          accessibilityRole="progressbar"
          accessibilityLabel={block.label}
          accessibilityValue={{ min: 0, max: 100, now: Math.round(value * 100) }}
        >
          {block.label ? <Text style={textStyle(theme, "caption", "muted")}>{block.label}</Text> : null}
          <View style={{ height: 6, borderRadius: 3, backgroundColor: tone.soft, overflow: "hidden" }}>
            <View style={{ width: `${value * 100}%`, height: "100%", backgroundColor: tone.fg }} />
          </View>
        </View>
      );
    }
    case "badge": {
      const tone = theme.tones[block.tone ?? "default"];
      return (
        <View
          style={{
            alignSelf: "flex-start",
            backgroundColor: tone.soft,
            borderRadius: theme.radius.pill,
            paddingHorizontal: theme.space.sm,
            paddingVertical: 2,
          }}
        >
          <Text style={[textStyle(theme, "caption", "default"), { color: tone.fg, fontFamily: theme.fonts.medium }]}>
            {block.text}
          </Text>
        </View>
      );
    }
    case "divider":
      return <Line />;
    case "spacer":
      return block.size ? <View style={{ width: theme.space[block.size], height: theme.space[block.size] }} /> : <View style={{ flex: 1 }} />;
    case "custom":
      return <Custom name={block.name} props={block.props} />;
    default:
      return <Unknown kind={(block as { type?: unknown }).type} />;
  }
}

function Children({ blocks }: { blocks: Block[] }) {
  return (
    <>
      {blocks.map((child, i) => (
        <BlockView key={i} block={child} />
      ))}
    </>
  );
}

function Item({ block }: { block: ItemBlock }) {
  const { theme, run } = useUlana();
  const body = (
    <View style={{ flexDirection: "row", alignItems: "center", gap: theme.space.sm, minHeight: theme.hitSize, paddingVertical: theme.space.sm }}>
      <View style={{ flex: 1 }}>
        <Text style={textStyle(theme, "body", "default")}>{block.title}</Text>
        {block.subtitle ? <Text style={textStyle(theme, "caption", "muted")}>{block.subtitle}</Text> : null}
      </View>
      {block.trailing ? <Text style={textStyle(theme, "body", "muted")}>{block.trailing}</Text> : null}
    </View>
  );
  const action = block.action;
  if (!action) return body;
  return (
    <Pressable
      accessibilityRole="button"
      onPress={() => run(action, null)}
      style={({ pressed }) => ({ opacity: pressed ? 0.6 : 1 })}
    >
      {body}
    </Pressable>
  );
}

function Button({ block }: { block: ButtonBlock }) {
  const { theme, run } = useUlana();
  const style = block.style ?? "primary";
  const accent = theme.tones.accent.fg;
  const box: ViewStyle = {
    minHeight: theme.hitSize,
    paddingHorizontal: style === "plain" ? theme.space.xs : theme.space.md,
    borderRadius: theme.radius.control,
    alignItems: "center",
    justifyContent: "center",
    alignSelf: "flex-start",
    ...(style === "primary" ? { backgroundColor: accent } : {}),
    ...(style === "secondary"
      ? { backgroundColor: theme.colors.surface, borderColor: theme.colors.border, borderWidth: 1 }
      : {}),
  };
  const label: RNTextStyle = {
    ...textStyle(theme, "body", "default"),
    fontFamily: theme.fonts.medium,
    fontWeight: "500",
    color: style === "primary" ? theme.colors.onAccent : style === "plain" ? accent : theme.colors.ink,
  };
  return (
    <Pressable
      accessibilityRole="button"
      onPress={() => run(block.action, null)}
      style={({ pressed }) => [box, { opacity: pressed ? 0.7 : 1, transform: [{ scale: pressed ? 0.98 : 1 }] }]}
    >
      <Text style={label}>{block.label}</Text>
    </Pressable>
  );
}

function Toggle({ block }: { block: ToggleBlock }) {
  const { theme, run } = useUlana();
  // Show the new value at once; the next frame confirms or corrects it.
  const [shown, setShown] = useState(block.value);
  useEffect(() => setShown(block.value), [block.value]);
  const action = block.action;
  return (
    <View style={{ flexDirection: "row", alignItems: "center", gap: theme.space.sm, minHeight: theme.hitSize }}>
      <Text style={[textStyle(theme, "body", "default"), { flex: 1 }]}>{block.label}</Text>
      <Switch
        accessibilityLabel={block.label}
        value={shown}
        disabled={!action}
        trackColor={{ true: theme.tones.accent.fg, false: theme.colors.border }}
        onValueChange={(next) => {
          if (!action) return;
          setShown(next);
          run(action, next);
        }}
      />
    </View>
  );
}

function Custom({ name, props }: { name: string; props: { [key: string]: Json } }) {
  const { theme, blocks } = useUlana();
  const Host = blocks[name];
  if (!Host) return <Unknown kind={name} />;
  return (
    <Guard theme={theme} label={`The ${name} block could not be drawn.`}>
      <Host props={props} theme={theme} />
    </Guard>
  );
}

function Unknown({ kind }: { kind: unknown }) {
  const { theme } = useUlana();
  return <Text style={textStyle(theme, "caption", "muted")}>{`This app cannot show “${String(kind)}” yet.`}</Text>;
}

function Line() {
  const { theme } = useUlana();
  return <View style={{ height: StyleSheet.hairlineWidth, backgroundColor: theme.colors.border }} />;
}

/** Keeps one broken block from taking the screen down with it. */
class Guard extends Component<{ theme: Theme; label: string; children: ReactNode }, { failed: boolean }> {
  override state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  override render() {
    if (!this.state.failed) return this.props.children;
    return <Text style={textStyle(this.props.theme, "caption", "danger")}>{this.props.label}</Text>;
  }
}

function textStyle(theme: Theme, style: TextStyle, tone: Tone): RNTextStyle {
  const t = theme.text[style];
  const family =
    style === "mono"
      ? theme.fonts.mono
      : t.fontWeight === "600" || t.fontWeight === "700"
        ? (theme.fonts.semibold ?? theme.fonts.medium)
        : theme.fonts.regular;
  return {
    fontSize: t.fontSize,
    lineHeight: t.lineHeight,
    ...(t.fontWeight ? { fontWeight: t.fontWeight } : {}),
    ...(family ? { fontFamily: family } : style === "mono" ? { fontFamily: "Menlo" } : {}),
    color: tone === "default" ? theme.colors.ink : theme.tones[tone].fg,
  };
}

function align(a: Align): FlexAlignType {
  return a === "start" ? "flex-start" : a === "end" ? "flex-end" : a;
}

function justify(j: Justify): ViewStyle["justifyContent"] {
  return j === "start" ? "flex-start" : j === "end" ? "flex-end" : j === "between" ? "space-between" : "center";
}
