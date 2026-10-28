import { beforeEach, describe, expect, it } from "vitest";

import { setLocale } from "../i18n";
import { fusionMethodLabel, jpegCountLabel, tonemapLabel } from "./fusion";

beforeEach(() => {
  setLocale("en");
});

describe("fusionMethodLabel", () => {
  it("labels mertens", () => {
    expect(fusionMethodLabel("mertens")).toBe("Mertens (natural)");
  });

  it("labels debevec", () => {
    expect(fusionMethodLabel("debevec")).toBe("Debevec + tone mapping");
  });
});

describe("tonemapLabel", () => {
  it("labels reinhard", () => {
    expect(tonemapLabel("reinhard")).toBe("Reinhard");
  });
});

describe("jpegCountLabel", () => {
  it("handles empty", () => {
    expect(jpegCountLabel(0)).toBe("No JPEG");
  });

  it("rejects a single image", () => {
    expect(jpegCountLabel(1)).toBe("1 JPEG — need at least 2");
  });

  it("counts a series", () => {
    expect(jpegCountLabel(5)).toBe("5 JPEG");
  });
});
