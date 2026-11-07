const UNITS: [name: string, seconds: number][] = [
  ["year", 365 * 86400],
  ["month", 30 * 86400],
  ["week", 7 * 86400],
  ["day", 86400],
  ["hour", 3600],
  ["minute", 60],
  ["second", 1],
];

export function humanizeUnixTime(date: Date): string {
  const pad = (num: number) => String(num).padStart(2, "0");

  const year = date.getUTCFullYear();
  const month = pad(date.getUTCMonth() + 1);
  const day = pad(date.getUTCDate());
  const hours = pad(date.getUTCHours());
  const minutes = pad(date.getUTCMinutes());
  const seconds = pad(date.getUTCSeconds());

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds} +00:00`;
}

export function getTimeAgoString(date: Date, now: Date = new Date()): string {
  const date_ms = date.getTime();
  if (!Number.isFinite(date_ms)) return ""; // Invalid Date -> NaN

  const diff = Math.floor((now.getTime() - date_ms) / 1000);
  const abs = Math.abs(diff);

  if (abs < 10) return "now";

  for (const [name, secs] of UNITS) {
    if (abs >= secs) {
      const n = Math.floor(abs / secs);
      const label = n === 1
        ? `${name === "hour" ? "an" : "a"} ${name}`
        : `${n} ${name}s`;
      return diff > 0 ? `${label} ago` : `in ${label}`;
    }
  }

  return "now"; // unreachable
}
