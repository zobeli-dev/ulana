// The OpenRouter key: in the system keychain on a phone, in this browser's
// storage on the web. It is only ever sent to OpenRouter.

import { Platform } from "react-native";
import * as SecureStore from "expo-secure-store";

const NAME = "ulana.openrouter.key";

export async function loadKey(): Promise<string | null> {
  try {
    if (Platform.OS === "web") return globalThis.localStorage?.getItem(NAME) ?? null;
    return await SecureStore.getItemAsync(NAME);
  } catch {
    return null;
  }
}

export async function saveKey(key: string): Promise<void> {
  try {
    if (Platform.OS === "web") globalThis.localStorage?.setItem(NAME, key);
    else await SecureStore.setItemAsync(NAME, key);
  } catch {
    // Not kept; it still works until the app closes.
  }
}

export async function forgetKey(): Promise<void> {
  try {
    if (Platform.OS === "web") globalThis.localStorage?.removeItem(NAME);
    else await SecureStore.deleteItemAsync(NAME);
  } catch {
    // Nothing to forget.
  }
}
