import type { Frame } from "./types";

/**
 * A light check that a value looks like a frame. The server checks every
 * tree fully before sending it; this only guards against a wrong response.
 */
export function isFrame(value: unknown): value is Frame {
  if (typeof value !== "object" || value === null) return false;
  const v = value as { version?: unknown; root?: unknown };
  return (
    typeof v.version === "number" &&
    typeof v.root === "object" &&
    v.root !== null &&
    typeof (v.root as { type?: unknown }).type === "string"
  );
}
