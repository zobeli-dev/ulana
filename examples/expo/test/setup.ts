// The example's logic runs under Bun without React Native.
import { mock } from "bun:test";

mock.module("react-native", () => ({ Platform: { OS: "web", select: (o: { default?: unknown }) => o.default } }));
