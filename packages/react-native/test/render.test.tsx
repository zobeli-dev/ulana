import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { UlanaView, defaultTheme, makeTheme } from "../src";
import type { CustomBlockProps, Frame } from "../src";

const agenda: Frame = {
  version: 3,
  root: {
    type: "stack",
    gap: "md",
    children: [
      {
        type: "row",
        justify: "between",
        children: [
          { type: "text", text: "Today", style: "heading" },
          { type: "badge", text: "2 events", tone: "accent" },
        ],
      },
      { type: "toggle", label: "Compact", value: true, action: { kind: "event", id: "h1" } },
      {
        type: "list",
        children: [
          { type: "item", title: "Standup", subtitle: "Room 2", trailing: "09:00" },
          { type: "item", title: "Lunch", trailing: "12:30", action: { kind: "host", name: "send", value: "lunch" } },
        ],
      },
      { type: "progress", value: 0.5, label: "1 of 2 done" },
      { type: "card", title: "Notes", children: [{ type: "text", text: "Bring the slides", tone: "muted", lines: 2 }] },
      { type: "divider" },
      { type: "spacer", size: "lg" },
      { type: "button", label: "Ask", action: { kind: "host", name: "send", value: "hi" }, style: "secondary" },
      { type: "custom", name: "calendar", props: { range: "today" } },
    ],
  },
};

function Calendar({ props }: CustomBlockProps) {
  return <span data-calendar={String(props.range)}>calendar</span>;
}

const draw = (frame: Frame, blocks = {}) => renderToStaticMarkup(<UlanaView frame={frame} blocks={blocks} />);

test("every block draws", () => {
  const html = draw(agenda, { calendar: Calendar });
  for (const text of ["Today", "2 events", "Compact", "Standup", "Room 2", "09:00", "Lunch", "1 of 2 done", "Notes", "Bring the slides", "Ask"]) {
    expect(html).toContain(text);
  }
  expect(html).toContain('data-calendar="today"');
  expect(html).toContain('role="switch"');
  expect(html).toContain('role="progressbar"');
  expect(html).toContain('aria-valuenow="50"');
  expect(html).toContain("background-color:rgba(255,255,255,1.00);height:20px");
  expect(html).not.toContain("cannot show");
});

test("a custom block the app does not have says so", () => {
  const html = draw({ version: 1, root: { type: "custom", name: "map", props: {} } });
  expect(html).toContain("This app cannot show “map” yet.");
});

test("an unknown block type does not break the screen", () => {
  const frame = { version: 1, root: { type: "stack", children: [{ type: "video" }, { type: "text", text: "still here" }] } };
  const html = draw(frame as unknown as Frame);
  expect(html).toContain("cannot show “video”");
  expect(html).toContain("still here");
});

test("the theme reaches the blocks", () => {
  const theme = makeTheme({ tones: { accent: { fg: "#C2410C", soft: "#FFEDD5" } } });
  const html = renderToStaticMarkup(
    <UlanaView frame={{ version: 1, root: { type: "badge", text: "hi", tone: "accent" } }} theme={theme} />,
  );
  expect(html.toLowerCase()).toContain("rgba(194,65,12,1.00)");
  expect(theme.tones.muted).toEqual(defaultTheme.tones.muted);
});
