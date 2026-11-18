// The frame a view renders. Mirrors `crates/ulana/src/block.rs`; the JSON
// Schema in `spec/frame.schema.json` is generated from those types.

export type Space = "none" | "xs" | "sm" | "md" | "lg" | "xl";
export type Align = "start" | "center" | "end" | "stretch";
export type Justify = "start" | "center" | "end" | "between";
export type TextStyle = "display" | "title" | "heading" | "body" | "caption" | "mono";
export type Tone = "default" | "muted" | "accent" | "success" | "warning" | "danger";
export type ButtonStyle = "primary" | "secondary" | "plain";
export type FieldKind = "text" | "number" | "email" | "search";

/** The icons a view may name. Each host maps them to its own set. */
export const ICON_NAMES = [
  "add", "remove", "close", "check", "chevron_left", "chevron_right", "search", "settings", "info",
  "warning", "error", "star", "heart", "bookmark", "calendar", "clock", "location", "person", "people",
  "mail", "phone", "message", "home", "bell", "sun", "moon", "cloud", "rain", "snow", "wind", "bolt",
  "cart", "gift", "music", "camera", "photo", "link", "share", "trash", "edit", "refresh", "lock",
  "flag", "play", "pause",
] as const;
export type IconName = (typeof ICON_NAMES)[number];

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

export type CardBlock = { type: "card"; title?: string; tone?: Tone; padding?: Space; children: Block[] };
/** Children in equal columns, row by row. */
export type GridBlock = { type: "grid"; columns: number; gap?: Space; children: Block[] };
export type ListBlock = { type: "list"; children: Block[] };

export type ItemBlock = {
  type: "item";
  title: string;
  icon?: IconName;
  subtitle?: string;
  trailing?: string;
  action?: Action;
};

export type ButtonBlock = { type: "button"; label: string; action: Action; icon?: IconName; style?: ButtonStyle };
/** A text input. `action` gets the text after each change, `submit` when return is pressed. */
export type FieldBlock = {
  type: "field";
  value: string;
  label?: string;
  placeholder?: string;
  kind?: FieldKind;
  action?: Action;
  submit?: Action;
};
/** One of 2 to 8 options, side by side. */
export type ChoiceBlock = { type: "choice"; options: string[]; value: string; label?: string; action?: Action };
export type IconBlock = { type: "icon"; name: IconName; tone?: Tone; style?: TextStyle; label?: string };
/** A picture from an address the host allows. `aspect` is width / height. */
export type ImageBlock = { type: "image"; src: string; alt: string; aspect?: number };
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
  | GridBlock
  | ListBlock
  | ItemBlock
  | ButtonBlock
  | ToggleBlock
  | FieldBlock
  | ChoiceBlock
  | IconBlock
  | ImageBlock
  | ProgressBlock
  | BadgeBlock
  | DividerBlock
  | SpacerBlock
  | CustomBlock;

/** One rendered tree and the version its events must name. */
export type Frame = { version: number; root: Block };
