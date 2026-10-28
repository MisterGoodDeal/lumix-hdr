import { describe, expect, it } from "vitest";

import { formatEv, sessionStateColor } from "./history";

describe("formatEv", () => {
  it("keeps sign for positive values", () => {
    expect(formatEv(1)).toBe("+1");
    expect(formatEv(-1)).toBe("-1");
  });
});

describe("sessionStateColor", () => {
  it("maps done to success", () => {
    expect(sessionStateColor("done")).toBe("success");
  });
});
