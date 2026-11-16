import type { WifiEntry } from "@/models";
import WifiItem from "../WifiItem";
import { useMemo, useState } from "preact/hooks";

interface WifiScanBoxProps {
  wifiEntries: WifiEntry[];
}

type SortKey = "ssid" | "auth_method" | "rssi" | "channel";
type SortOrder = "asc" | "desc";
type SortState = { key: SortKey; order: SortOrder };

const SORT_LABELS: Record<SortKey, string> = {
  ssid: "ssid",
  auth_method: "auth_method",
  rssi: "rssi",
  channel: "channel",
};

export default function WifiScanBox(
  { wifiEntries }: WifiScanBoxProps,
) {
  const [sort, setSort] = useState<SortState>({ key: "ssid", order: "asc" });

  const sortedNodes = useMemo(
    () => sortNodes(wifiEntries, sort),
    [wifiEntries, sort],
  );

  const toggleSort = (key: SortKey) =>
    setSort((prev) => ({
      key,
      order: prev.key === key && prev.order === "asc" ? "desc" : "asc",
    }));

  const header = (key: SortKey, label: string) => (
    <SortLink
      sortKey={key}
      label={label}
      sort={sort}
      onSort={toggleSort}
    />
  );

  return (
    <table>
      <thead>
        <tr>
          <th class="ssid">{header("ssid", "SSID")}</th>
          <th class="auth">{header("auth_method", "Auth")}</th>
          <th class="rssi">{header("rssi", "RSSI")}</th>
          <th class="channel">{header("channel", "Channel")}</th>
        </tr>
      </thead>
      <tbody>
        {sortedNodes.map((sortedNode) => <WifiItem wifiEntry={sortedNode} />)}
      </tbody>
    </table>
  );
}

function sortNodes(nodes: WifiEntry[], { key, order }: SortState) {
  let isAsc = order == "asc" ? 1 : -1;

  return [...nodes].sort((a, b) => {
    switch (key) {
      case "rssi": {
        return ((a.rssi || 0) - (b.rssi || 0)) * isAsc;
      }
      case "channel": {
        return ((a.channel || 0) - (b.channel || 0)) * isAsc;
      }
      case "auth_method": {
        return a.auth_method.localeCompare(
          b.auth_method,
          undefined,
          { numeric: true, sensitivity: "base" },
        ) * isAsc;
      }
      default:
        return a.ssid.localeCompare(
          b.ssid,
          undefined,
          { numeric: true, sensitivity: "base" },
        ) * isAsc;
    }
  });
}

type SortLinkProps = {
  sortKey: SortKey;
  label: string;
  sort: SortState;
  onSort: (key: SortKey) => void;
};

function SortLink({ sortKey, label, sort, onSort }: SortLinkProps) {
  const active = sort.key === sortKey;
  const nextOrder: SortOrder = active && sort.order === "asc" ? "desc" : "asc";
  const nextLabel = nextOrder === "asc" ? "ascending" : "descending";

  return (
    <span class={active ? "active" : ""}>
      <span class="chevron">{active && sort.order === "asc" ? "▴" : "▾"}</span>
      <a
        role="button"
        tabIndex={0}
        title={`Sort by ${SORT_LABELS[sortKey]} in ${nextLabel} order`}
        onClick={() => onSort(sortKey)}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onSort(sortKey);
          }
        }}
      >
        {label}
      </a>
    </span>
  );
}
