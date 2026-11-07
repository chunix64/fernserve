export function getSizeString(size_byte: number): string {
  const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

  if (!Number.isFinite(size_byte) || size_byte < 0) return "0.0 B";

  let size = size_byte;
  let i = 0;

  while (i < units.length - 1 && Math.round(size * 10) / 10 >= 1000) {
    size /= 1024;
    i++;
  }

  return i === 0
    ? `${Math.round(size).toFixed(1)} B`
    : `${size.toFixed(1)} ${units[i]}`;
}
