const LOCALE = "pt-BR";

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const exp = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** exp;
  return `${value.toLocaleString(LOCALE, { maximumFractionDigits: value < 10 && exp > 0 ? 1 : 0 })} ${units[exp]}`;
}

const relative = new Intl.RelativeTimeFormat(LOCALE, { numeric: "auto" });

export function formatRelative(iso: string | null | undefined): string {
  if (!iso) return "nunca";
  const diffSeconds = (new Date(iso).getTime() - Date.now()) / 1000;
  const abs = Math.abs(diffSeconds);
  if (abs < 45) return "agora mesmo";
  if (abs < 3600) return relative.format(Math.round(diffSeconds / 60), "minute");
  if (abs < 86400) return relative.format(Math.round(diffSeconds / 3600), "hour");
  if (abs < 86400 * 30) return relative.format(Math.round(diffSeconds / 86400), "day");
  return formatDateTime(iso);
}

export function formatDateTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleString(LOCALE, { dateStyle: "short", timeStyle: "short" });
}

export function formatTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleTimeString(LOCALE, { hour: "2-digit", minute: "2-digit" });
}

/** Cor determinística (matiz) a partir de um texto — usada nas "capas" dos saves. */
export function hueFrom(text: string): number {
  let hash = 0;
  for (let i = 0; i < text.length; i++) hash = (hash * 31 + text.charCodeAt(i)) | 0;
  return Math.abs(hash) % 360;
}

export function cn(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(" ");
}
