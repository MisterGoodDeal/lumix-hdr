import { describe, expect, it } from "vitest";

import { fusionMethodLabel, jpegCountLabel, tonemapLabel } from "./fusion";

describe("fusionMethodLabel", () => {
  it("nomme Mertens", () => {
    expect(fusionMethodLabel("mertens")).toBe("Mertens (naturel)");
  });
});

describe("tonemapLabel", () => {
  it("nomme Reinhard", () => {
    expect(tonemapLabel("reinhard")).toBe("Reinhard");
  });
});

describe("jpegCountLabel", () => {
  it("signale l'absence d'image", () => {
    expect(jpegCountLabel(0)).toBe("Aucun JPEG");
  });

  it("refuse une seule image", () => {
    expect(jpegCountLabel(1)).toBe("1 JPEG — il en faut au moins deux");
  });

  it("compte une série", () => {
    expect(jpegCountLabel(5)).toBe("5 JPEG prêts à fusionner");
  });
});
