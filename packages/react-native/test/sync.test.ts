// The TypeScript types are written by hand; the JSON Schema is generated
// from the Rust types. This test fails when the two drift apart.

import { expect, test } from "bun:test";

import schema from "../../../spec/frame.schema.json";
import { ICON_NAMES } from "../src/types";
import type { Action, Align, Block, ButtonStyle, FieldKind, IconName, Justify, Space, TextStyle, Tone } from "../src/types";

type Kind = Block["type"];
type KeysOf<K extends Kind> = keyof Extract<Block, { type: K }>;

// Every field of every block, checked against the TypeScript types both ways.
const fields = {
  stack: ["type", "gap", "padding", "align", "children"],
  row: ["type", "gap", "padding", "align", "justify", "wrap", "children"],
  text: ["type", "text", "style", "tone", "lines"],
  card: ["type", "title", "tone", "padding", "children"],
  grid: ["type", "columns", "gap", "children"],
  field: ["type", "value", "label", "placeholder", "kind", "action", "submit"],
  choice: ["type", "options", "value", "label", "action"],
  icon: ["type", "name", "tone", "style", "label"],
  image: ["type", "src", "alt", "aspect"],
  list: ["type", "children"],
  item: ["type", "title", "icon", "subtitle", "trailing", "action"],
  button: ["type", "label", "action", "icon", "style"],
  toggle: ["type", "label", "value", "action"],
  progress: ["type", "value", "label", "tone"],
  badge: ["type", "text", "tone"],
  divider: ["type"],
  spacer: ["type", "size"],
  custom: ["type", "name", "props"],
} as const satisfies { [K in Kind]: readonly KeysOf<K>[] };

type Assert<T extends never> = T;
export type MissingFields = Assert<{ [K in Kind]: Exclude<KeysOf<K>, (typeof fields)[K][number]> }[Kind]>;

const actionFields = {
  event: ["kind", "id"],
  host: ["kind", "name", "value"],
} as const satisfies { [K in Action["kind"]]: readonly (keyof Extract<Action, { kind: K }>)[] };

function all<T extends string>() {
  return <const A extends readonly T[]>(a: A & ([T] extends [A[number]] ? unknown : never)) => a;
}

const enums = {
  Space: all<Space>()(["none", "xs", "sm", "md", "lg", "xl"]),
  Align: all<Align>()(["start", "center", "end", "stretch"]),
  Justify: all<Justify>()(["start", "center", "end", "between"]),
  TextStyle: all<TextStyle>()(["display", "title", "heading", "body", "caption", "mono"]),
  FieldKind: all<FieldKind>()(["text", "number", "email", "search"]),
  IconName: all<IconName>()(ICON_NAMES),
  Tone: all<Tone>()(["default", "muted", "accent", "success", "warning", "danger"]),
  ButtonStyle: all<ButtonStyle>()(["primary", "secondary", "plain"]),
};

type Variant = { properties: Record<string, { const?: string }> };
const defs = schema.$defs as unknown as Record<string, { oneOf?: Variant[]; enum?: string[] }>;

function variants(def: string, tag: string): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  for (const v of defs[def]?.oneOf ?? []) {
    const name = v.properties[tag]?.const;
    if (name) out[name] = Object.keys(v.properties).sort();
  }
  return out;
}

const sorted = (o: Record<string, readonly string[]>) =>
  Object.fromEntries(Object.entries(o).map(([k, v]) => [k, [...v].sort()]));

test("blocks have the same fields as the schema", () => {
  expect(sorted(fields)).toEqual(variants("Block", "type"));
});

test("actions have the same fields as the schema", () => {
  expect(sorted(actionFields)).toEqual(variants("Action", "kind"));
});

test("tokens have the same values as the schema", () => {
  for (const [name, values] of Object.entries(enums)) {
    expect<string[]>([...values].sort()).toEqual([...(defs[name]?.enum ?? [])].sort());
  }
});
