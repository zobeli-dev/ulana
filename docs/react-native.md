# Ulana in React Native and Expo

`@zobeli/ulana-react-native` draws Ulana views with React Native's own components on iOS, Android, and the web. Views run on a server (`ulana serve`, or your own built with the [Rust crate](rust.md)); the app follows them and draws each frame.

## Install

The package is not on npm yet. Until it is, add it from a clone:

- **Copy it.** Copy [`packages/react-native`](../packages/react-native) into your repository, for example as `packages/ulana-react-native`, and depend on it with `"@zobeli/ulana-react-native": "workspace:*"` (Bun, pnpm, or Yarn workspaces) or `"file:./packages/ulana-react-native"` (npm).
- **Or point at a clone.** `bun add ../ulana/packages/react-native`.

It ships TypeScript source, which Metro compiles. Its peer dependencies are `react` 19 and `react-native` 0.80 or later. It has no other dependencies.

In a monorepo, make sure there is one copy of React. Expo's Metro config finds Bun workspaces by itself; the Expo example in [`examples/expo`](../examples/expo) is set up that way.

## One view in one line

```tsx
import { UlanaServerView } from "@zobeli/ulana-react-native";

export function CounterScreen() {
  return (
    <ScrollView contentContainerStyle={{ padding: 16 }}>
      <UlanaServerView url="http://127.0.0.1:8790" name="counter" />
    </ScrollView>
  );
}
```

`UlanaServerView` follows the view, draws each frame, sends presses back, and shows the script's error above the last frame that worked. Pass `showErrors={false}` in production. It takes the same `theme`, `blocks`, `icons`, and `onHostAction` as `UlanaView`.

The Android emulator reaches your computer at `http://10.0.2.2:8790`. A phone on the same Wi-Fi needs the computer's address, with `ulana serve --bind 0.0.0.0:8790`.

## Hooks

For your own layout:

```tsx
import { UlanaView, useServerView, useServerViews } from "@zobeli/ulana-react-native";

function Views() {
  const { names } = useServerViews(URL);              // every view the server has, refreshed every 2 s
  const [name, setName] = useState("counter");
  const view = useServerView(URL, name);              // { frame, error, logs, offline, send, accept, snapshot }

  return (
    <>
      <Tabs names={names} value={name} onChange={setName} />
      {view.error ? <ErrorNote text={view.error} /> : null}
      {view.frame ? <UlanaView frame={view.frame} onEvent={view.send} /> : null}
    </>
  );
}
```

`useServerView` long-polls the server, so a saved script shows up at once. `send` posts a press and draws the answer. `accept(snapshot)` takes a snapshot you got elsewhere, such as the answer to writing a new script.

## Drawing a frame yourself

```tsx
<UlanaView
  frame={frame}                                    // { version, root }, checked by the runtime
  theme={theme}
  onEvent={(version, id, value) => runOnServer(version, id, value).then(setFrame)}
  onHostAction={(name, value) => name === "send" && sendMessage(String(value))}
  blocks={{ calendar: CalendarBlock }}             // your own blocks, by the name a view gives ui.custom
  icons={myIcons}                                  // see below
/>
```

- `onEvent` is a press or change on the view's own handlers. Run it where the view runs, render again, and pass the new frame.
- `onHostAction` is a press on an action your app named (`ui.host("send", …)`). Your app decides what happens.
- A custom block gets `{ props, theme }`. A name you did not register shows a short note, and a block that throws is caught by an error boundary, so one broken block never takes the screen down.
- `isFrame(value)` is a light check for a response you are about to draw. The runtime has already checked the tree.

## Theme

A view only names tokens. The theme decides how they look:

```tsx
import { makeTheme } from "@zobeli/ulana-react-native";

const theme = makeTheme({
  colors: { canvas: "#F1F2F4", ink: "#14171C" },
  tones: { accent: { fg: "#C2410C", soft: "#FFEDD5" } },
  fonts: { regular: "IBMPlexSans-Regular", medium: "IBMPlexSans-Medium", semibold: "IBMPlexSans-SemiBold", mono: "IBMPlexMono-Regular" },
  radius: { card: 18 },
});
```

`makeTheme` merges each group one level deep into `defaultTheme`: `colors`, `tones` (`fg` and a `soft` background for each of `default`, `muted`, `accent`, `success`, `warning`, `danger`), `space` (`none` to `xl`, in points), `radius`, `text` (size, line height, and weight per style, `display` to `mono`), `fonts`, and `hitSize` (44 by default).

## Icons

Views name icons from a fixed set (`ICON_NAMES`: `add`, `check`, `star`, `sun`, `calendar`, …). By default the package draws them as text glyphs, so it needs no icon font. Pass your own set for better ones. With Expo:

```tsx
import Ionicons from "@expo/vector-icons/Ionicons";
import type { IconName, IconRenderer } from "@zobeli/ulana-react-native";

const NAMES: Record<IconName, React.ComponentProps<typeof Ionicons>["name"]> = { add: "add", star: "star", /* … */ };
const icons: IconRenderer = ({ name, size, color }) => <Ionicons name={NAMES[name]} size={size} color={color} />;
```

[`examples/expo/src/icons.tsx`](../examples/expo/src/icons.tsx) maps every name.

## Accessibility

Headings use the header role. Buttons, items with actions, switches, choices (a radio group), fields, and progress bars use their native roles. Icons with a `label` are announced, and icons without one are hidden. Images require `alt`. Pressable things are at least 44 points tall.

## Tests

```sh
cd packages/react-native && bun run typecheck && bun test
```

`test/render.test.tsx` draws every block through react-native-web with no device. `test/sync.test.ts` fails when the TypeScript types drift from the Rust schema.
