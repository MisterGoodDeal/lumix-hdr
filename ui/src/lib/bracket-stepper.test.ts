import { describe, expect, it } from "vitest";

import type { CaptureProgress, CaptureState } from "../types";
import {
  bracketStepStatus,
  connectorFill,
  formatRelativeEv,
} from "./bracket-stepper";

function progress(state: CaptureState): CaptureProgress {
  return { state, progress: 0.4, message: "…" };
}

describe("bracketStepStatus", () => {
  it("keeps every shot pending before capture", () => {
    expect(bracketStepStatus(0, 5, null)).toBe("pending");
    expect(bracketStepStatus(2, 5, progress({ kind: "preparing" }))).toBe(
      "pending",
    );
  });

  it("marks previous shots done and the active frame current", () => {
    const current = progress({
      kind: "trigger",
      frame: 2,
      total: 5,
    });
    expect(bracketStepStatus(0, 5, current)).toBe("completed");
    expect(bracketStepStatus(1, 5, current)).toBe("completed");
    expect(bracketStepStatus(2, 5, current)).toBe("current");
    expect(bracketStepStatus(3, 5, current)).toBe("pending");
    expect(bracketStepStatus(4, 5, current)).toBe("pending");
  });

  it("completes every shot while merging", () => {
    expect(bracketStepStatus(0, 3, progress({ kind: "merging" }))).toBe(
      "completed",
    );
    expect(bracketStepStatus(2, 3, progress({ kind: "done" }))).toBe(
      "completed",
    );
  });
});

describe("connectorFill", () => {
  it("fills the line after a completed step", () => {
    expect(connectorFill("completed", "completed")).toBe("full");
    expect(connectorFill("completed", "current")).toBe("half");
    expect(connectorFill("current", "pending")).toBe("half");
    expect(connectorFill("pending", "pending")).toBe("empty");
  });
});

describe("formatRelativeEv", () => {
  it("signs positive exposures", () => {
    expect(formatRelativeEv(2)).toBe("+2 IL");
    expect(formatRelativeEv(0)).toBe("0 IL");
    expect(formatRelativeEv(-1)).toBe("-1 IL");
  });
});
