type NumberFormatter = (value: number, options?: Intl.NumberFormatOptions) => string;

/** KB below one megabyte, MB with one decimal above it. */
export function formatBytes(bytes: number, formatNumber: NumberFormatter): string {
  return bytes < 1024 * 1024
    ? `${formatNumber(Math.max(1, Math.round(bytes / 1024)))} KB`
    : `${formatNumber(bytes / (1024 * 1024), { maximumFractionDigits: 1 })} MB`;
}
