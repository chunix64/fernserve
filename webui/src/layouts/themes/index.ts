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
  if (id == "default") {
    document.body.removeAttribute("data-theme");
  } else {
    document.body.setAttribute("data-theme", id);
  }
  localStorage.setItem(KEY, id);
}

export function loadTheme() {
  setTheme(getTheme());
}

export function themeEvents() {
  window.addEventListener("load", loadTheme);
  window.addEventListener("storage", loadTheme);

  if (document.readyState === "complete") loadTheme();

  return () => {
    window.removeEventListener("load", loadTheme);
    window.removeEventListener("storage", loadTheme);
  };
}
