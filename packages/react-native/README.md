# @zobeli/ulana-react-native

Draw [Ulana](https://github.com/zobeli-dev/ulana) views with React Native's own components on iOS, Android, and the web. Ulana lets a model (or a person) change your app's screens with small Luau scripts that run in a sandbox and are checked before they are drawn.

```tsx
import { UlanaServerView } from "@zobeli/ulana-react-native";

<UlanaServerView url="http://127.0.0.1:8790" name="counter" />;
```

- `UlanaServerView`, `useServerView`, and `useServerViews` follow views on a server running `ulana serve` or its protocol.
- `UlanaView` draws a frame you got any other way.
- `makeTheme`, your own blocks (`blocks`), your own icons (`icons`), and host actions (`onHostAction`).

Guide: [docs/react-native.md](https://github.com/zobeli-dev/ulana/blob/main/docs/react-native.md). MIT licensed.
