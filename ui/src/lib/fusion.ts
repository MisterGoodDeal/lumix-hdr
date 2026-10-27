import type { FusionMethod, TonemapKind } from "../types";

export function fusionMethodLabel(method: FusionMethod): string {
  switch (method) {
    case "mertens":
      return "Mertens (naturel)";
    case "debevec":
      return "Debevec + tone mapping";
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
    return "Aucun JPEG";
  }
  if (count === 1) {
    return "1 JPEG — il en faut au moins deux";
  }
  return `${count} JPEG prêts à fusionner`;
}
