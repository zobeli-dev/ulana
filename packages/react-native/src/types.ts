// The frame a view renders. Mirrors `crates/ulana/src/block.rs`; the JSON
// Schema in `spec/frame.schema.json` is generated from those types.

export type Space = "none" | "xs" | "sm" | "md" | "lg" | "xl";
export type Align = "start" | "center" | "end" | "stretch";
export type Justify = "start" | "center" | "end" | "between";
export type TextStyle = "title" | "heading" | "body" | "caption" | "mono";
export type Tone = "default" | "muted" | "accent" | "success" | "warning" | "danger";
export type ButtonStyle = "primary" | "secondary" | "plain";

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

/** What a press or a change does. */
export type Action =
  /** Run the view's own handler, then render again. */
  | { kind: "event"; id: string }
  /** Ask the host app to do something it named. */
  | { kind: "host"; name: string; value?: Json };

export type StackBlock = {
  type: "stack";
  gap?: Space;
  padding?: Space;
  align?: Align;
  children: Block[];
};

export type RowBlock = {
  type: "row";
  gap?: Space;
  padding?: Space;
  align?: Align;
  justify?: Justify;
  wrap?: boolean;
  children: Block[];
};

export type TextBlock = {
  type: "text";
  text: string;
  style?: TextStyle;
  tone?: Tone;
  lines?: number;
};

export type CardBlock = { type: "card"; title?: string; children: Block[] };
export type ListBlock = { type: "list"; children: Block[] };

export type ItemBlock = {
  type: "item";
  title: string;
  subtitle?: string;
  trailing?: string;
  action?: Action;
};

export type ButtonBlock = { type: "button"; label: string; action: Action; style?: ButtonStyle };
export type ToggleBlock = { type: "toggle"; label: string; value: boolean; action?: Action };
export type ProgressBlock = { type: "progress"; value: number; label?: string; tone?: Tone };
export type BadgeBlock = { type: "badge"; text: string; tone?: Tone };
export type DividerBlock = { type: "divider" };
export type SpacerBlock = { type: "spacer"; size?: Space };
export type CustomBlock = { type: "custom"; name: string; props: { [key: string]: Json } };

export type Block =
  | StackBlock
  | RowBlock
  | TextBlock
  | CardBlock
  | ListBlock
  | ItemBlock
  | ButtonBlock
  | ToggleBlock
  | ProgressBlock
  | BadgeBlock
  | DividerBlock
  | SpacerBlock
  | CustomBlock;

/** One rendered tree and the version its events must name. */
export type Frame = { version: number; root: Block };
