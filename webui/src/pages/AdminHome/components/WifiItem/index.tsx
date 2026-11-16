import type { WifiEntry } from "@/models";

interface WifiItemProp {
  wifiEntry: WifiEntry;
}

export default function WifiItem({ wifiEntry }: WifiItemProp) {
  return (
    <tr key={wifiEntry.ssid}>
      <td>{wifiEntry.ssid}</td>
      <td>{wifiEntry.auth_method}</td>
      <td>{wifiEntry.rssi}</td>
      <td>{wifiEntry.channel}</td>
    </tr>
  );
}
