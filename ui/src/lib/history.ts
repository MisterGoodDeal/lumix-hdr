import { t } from "../i18n";
import type { SessionSummary } from "../types";

export function formatSessionDate(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return iso;
  }
  try {
    return new Intl.DateTimeFormat(locale, {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(date);
  } catch {
    return iso;
  }
}

export function sessionStateLabel(kind: string): string {
  switch (kind) {
    case "done":
      return t("history.state.done");
    case "aborted":
      return t("history.state.aborted");
    case "failed":
      return t("history.state.failed");
    default:
      return t("history.state.other", { kind });
  }
}

export function sessionStateColor(
  kind: string,
): "success" | "warning" | "danger" | "default" {
  switch (kind) {
    case "done":
      return "success";
    case "aborted":
      return "warning";
    case "failed":
      return "danger";
    default:
      return "default";
  }
}

export function formatEv(value: number): string {
  const rounded = Math.round(value * 100) / 100;
  const sign = rounded > 0 ? "+" : "";
  return `${sign}${rounded}`;
}

export function sessionCardTitle(summary: SessionSummary): string {
  return summary.session_name.trim() || summary.session_dir.split(/[/\\]/).pop() || "session";
}
