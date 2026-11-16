import type { WifiEntry } from "@/models";
import { useLocation } from "preact-iso";

interface WifiItemProp {
  wifiEntry: WifiEntry;
}

export default function WifiItem({ wifiEntry }: WifiItemProp) {
  const { path } = useLocation();

  const basePath = path.endsWith("/") ? path.slice(0, -1) : path;
  const href =
    `${basePath}/wifi/login?ssid=${wifiEntry.ssid}&auth=${wifiEntry.auth_method}`;

  return (
    <tr key={wifiEntry.ssid}>
      <td>
        <a href={href}>{wifiEntry.ssid}</a>
      </td>
      <td>{wifiEntry.auth_method}</td>
      <td>{wifiEntry.rssi}</td>
      <td>{wifiEntry.channel}</td>
    </tr>
  );
}
