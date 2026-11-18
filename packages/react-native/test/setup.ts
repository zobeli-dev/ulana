// Tests draw with react-native-web, so the renderer runs without a device.
import { mock } from "bun:test";

const web = await import("react-native-web");
mock.module("react-native", () => web);
