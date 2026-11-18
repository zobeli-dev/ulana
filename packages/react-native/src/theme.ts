import type { Space, TextStyle, Tone } from "./types";

/** A tone's colour, and a soft version for backgrounds such as a badge. */
export type ToneColors = { fg: string; soft: string };

/**
 * Everything the renderer draws with. A view only names tokens ("md",
 * "accent"); the host decides what they look like.
 */
export type Theme = {
  colors: {
    canvas: string;
    surface: string;
    ink: string;
    border: string;
    onAccent: string;
  };
  tones: Record<Tone, ToneColors>;
  space: Record<Space, number>;
  radius: { card: number; control: number; pill: number };
  text: Record<TextStyle, { fontSize: number; lineHeight: number; fontWeight?: "400" | "500" | "600" | "700" }>;
  fonts: { regular?: string; medium?: string; semibold?: string; mono?: string };
  /** The smallest side of anything pressable, in points. */
  hitSize: number;
};

export const defaultTheme: Theme = {
  colors: {
    canvas: "#F4F4F5",
    surface: "#FFFFFF",
    ink: "#18181B",
    border: "#E4E4E7",
    onAccent: "#FFFFFF",
  },
  tones: {
    default: { fg: "#18181B", soft: "#F4F4F5" },
    muted: { fg: "#71717A", soft: "#F4F4F5" },
    accent: { fg: "#2563EB", soft: "#DBEAFE" },
    success: { fg: "#15803D", soft: "#DCFCE7" },
    warning: { fg: "#B45309", soft: "#FEF3C7" },
    danger: { fg: "#B91C1C", soft: "#FEE2E2" },
  },
  space: { none: 0, xs: 4, sm: 8, md: 16, lg: 24, xl: 32 },
  radius: { card: 14, control: 10, pill: 999 },
  text: {
    title: { fontSize: 28, lineHeight: 34, fontWeight: "600" },
    heading: { fontSize: 20, lineHeight: 26, fontWeight: "600" },
    body: { fontSize: 16, lineHeight: 22, fontWeight: "400" },
    caption: { fontSize: 13, lineHeight: 18, fontWeight: "400" },
    mono: { fontSize: 14, lineHeight: 20, fontWeight: "400" },
  },
  fonts: {},
  hitSize: 44,
};

/** A theme with some values replaced; nested groups merge one level deep. */
export function makeTheme(overrides: {
  [K in keyof Theme]?: Theme[K] extends object ? Partial<Theme[K]> : Theme[K];
}): Theme {
  const out = { ...defaultTheme } as Record<string, unknown>;
  for (const [key, value] of Object.entries(overrides)) {
    const base = (defaultTheme as Record<string, unknown>)[key];
    out[key] =
      typeof base === "object" && base !== null && typeof value === "object" && value !== null
        ? { ...base, ...value }
        : value;
  }
  return out as Theme;
}
