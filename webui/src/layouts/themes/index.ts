export const THEMES = [
  { id: "default", label: "Default (light/dark)" },
  { id: "squirrel", label: "Squirrel (light)" },
  { id: "archlinux", label: "Arch Linux (dark)" },
  { id: "ayu_dark", label: "Ayu Dark (dark)" },
  { id: "zenburn", label: "Zenburn (dark)" },
  { id: "monokai", label: "Monokai (dark)" },
];

export type ThemeId = (typeof THEMES)[number]["id"];

const KEY = "theme";

export function getTheme() {
  return (localStorage.getItem(KEY) as ThemeId) ?? "default";
}

export function setTheme(id: ThemeId) {
  document.documentElement.dataset.theme = id;
  localStorage.setItem(KEY, id);
}
