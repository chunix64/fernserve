import { useState } from "preact/hooks";

import { getTheme, setTheme, type ThemeId, THEMES } from "@/layouts/themes";

export default function Header() {
  const [themeState, setThemeState] = useState<ThemeId>(getTheme);

  const selectTheme = (id: ThemeId) => {
    setThemeState(id);
    setTheme(id);
  };

  return (
    <>
      <nav>
        <div>
          <p>Change theme...</p>
          <ul class="theme">
            {THEMES.map(({ id, label }) => (
              <li key={id} data-theme={id}>
                <a
                  title={`"Switch to ${label} theme`}
                  aria-pressed={themeState == id}
                  onClick={() => selectTheme(id)}
                >
                  {label}
                </a>
              </li>
            ))}
          </ul>
        </div>
      </nav>
    </>
  );
}
