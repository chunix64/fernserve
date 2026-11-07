import { version } from "@/../package.json";

export default function Footer() {
  return (
    <div class="footer">
      <div class="version">
        <a href="https://github.com/chunix64/fernserve">
          fernserve
        </a>/{version}
      </div>
    </div>
  );
}
