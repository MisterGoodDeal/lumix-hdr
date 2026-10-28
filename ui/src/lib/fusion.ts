import { t } from "../i18n";
import type { FusionMethod, TonemapKind } from "../types";

export function fusionMethodLabel(method: FusionMethod): string {
  switch (method) {
    case "mertens":
      return t("fusion.mertens");
    case "debevec":
      return t("fusion.debevec");
  }
}

export function tonemapLabel(kind: TonemapKind): string {
  switch (kind) {
    case "reinhard":
      return "Reinhard";
    case "mantiuk":
      return "Mantiuk";
  }
}

export function jpegCountLabel(count: number): string {
  if (count === 0) {
    return t("fusion.jpegNone");
  }
  if (count === 1) {
    return t("fusion.jpegOne");
  }
  return t("fusion.jpegMany", { count });
}
