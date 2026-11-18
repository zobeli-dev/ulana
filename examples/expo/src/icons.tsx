// Ulana's icon names drawn with Ionicons, which ships with Expo.

import Ionicons from "@expo/vector-icons/Ionicons";
import type { ComponentProps } from "react";
import type { IconName, IconRenderer } from "@zobeli/ulana-react-native";

type Ion = ComponentProps<typeof Ionicons>["name"];

const NAMES: Record<IconName, Ion> = {
  add: "add", remove: "remove", close: "close", check: "checkmark", chevron_left: "chevron-back",
  chevron_right: "chevron-forward", search: "search", settings: "settings-outline",
  info: "information-circle-outline", warning: "warning-outline", error: "close-circle-outline",
  star: "star", heart: "heart", bookmark: "bookmark-outline", calendar: "calendar-outline",
  clock: "time-outline", location: "location-outline", person: "person-outline", people: "people-outline",
  mail: "mail-outline", phone: "call-outline", message: "chatbubble-outline", home: "home-outline",
  bell: "notifications-outline", sun: "sunny", moon: "moon", cloud: "cloud", rain: "rainy", snow: "snow",
  wind: "leaf-outline", bolt: "flash", cart: "cart-outline", gift: "gift-outline",
  music: "musical-notes-outline", camera: "camera-outline", photo: "image-outline", link: "link-outline",
  share: "share-outline", trash: "trash-outline", edit: "create-outline", refresh: "refresh",
  lock: "lock-closed-outline", flag: "flag-outline", play: "play", pause: "pause",
};

export const ionicons: IconRenderer = ({ name, size, color }) => (
  <Ionicons name={NAMES[name] ?? "ellipse-outline"} size={size} color={color} />
);
