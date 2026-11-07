export default function Header() {
  return (
    <>
      <nav>
        <div>
          <p>Change theme...</p>
          <ul class="theme">
            <li data-theme="default">
              <a
                href="javascript:updateColorScheme(&quot;default&quot;)"
                title="Switch to Default (light/dark) theme"
              >
                Default (light/dark)
              </a>
            </li>
            <li data-theme="squirrel">
              <a
                href="javascript:updateColorScheme(&quot;squirrel&quot;)"
                title="Switch to Squirrel (light) theme"
              >
                Squirrel (light)
              </a>
            </li>
            <li data-theme="archlinux">
              <a
                href="javascript:updateColorScheme(&quot;archlinux&quot;)"
                title="Switch to Arch Linux (dark) theme"
              >
                Arch Linux (dark)
              </a>
            </li>
            <li data-theme="ayu_dark">
              <a
                href="javascript:updateColorScheme(&quot;ayu_dark&quot;)"
                title="Switch to Ayu Dark (dark) theme"
              >
                Ayu Dark (dark)
              </a>
            </li>
            <li data-theme="zenburn">
              <a
                href="javascript:updateColorScheme(&quot;zenburn&quot;)"
                title="Switch to Zenburn (dark) theme"
              >
                Zenburn (dark)
              </a>
            </li>
            <li data-theme="monokai">
              <a
                href="javascript:updateColorScheme(&quot;monokai&quot;)"
                title="Switch to Monokai (dark) theme"
              >
                Monokai (dark)
              </a>
            </li>
          </ul>
        </div>
      </nav>
    </>
  );
}
