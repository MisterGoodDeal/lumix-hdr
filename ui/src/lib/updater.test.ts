import { describe, expect, it } from "vitest";

import { updateProgressPercent } from "./updater";

describe("updateProgressPercent", () => {
  it("retourne null sans total", () => {
    expect(updateProgressPercent({ downloaded: 10, total: null })).toBeNull();
  });

  it("calcule le pourcentage", () => {
    expect(updateProgressPercent({ downloaded: 25, total: 100 })).toBe(25);
  });

  it("plafonne à 100", () => {
    expect(updateProgressPercent({ downloaded: 120, total: 100 })).toBe(100);
  });
});
