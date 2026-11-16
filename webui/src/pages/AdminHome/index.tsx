import FernClient from "@/api/client";
import { WIFI_SCAN_INTERVAL_MS } from "@/configs";
import type { WifiEntry } from "@/models";
import WifiScanBox from "@/pages/AdminHome/components/WifiScanBox";
import { useEffect, useState } from "preact/hooks";

export default function AdminHome() {
  let [wifiEntries, setWifiEntries] = useState<WifiEntry[]>([]);

  useEffect(() => {
    (async () => setWifiEntries(await FernClient.scanWifi()))();

    setInterval(async () => {
      setWifiEntries(await FernClient.scanWifi());
    }, WIFI_SCAN_INTERVAL_MS);
  }, []);

  return (
    <>
      <div className={"header"}></div>
      <WifiScanBox wifiEntries={wifiEntries} />
      <div className={"footer"}></div>
    </>
  );
}
