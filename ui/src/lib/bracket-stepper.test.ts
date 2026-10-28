import { describe, expect, it } from "vitest";

import type { CaptureProgress, CaptureState } from "../types";
import {
  bracketStepStatus,
  connectorFill,
  connectorRatio,
  formatRelativeEv,
  mergePhaseProgress,
  mergeStepStatus,
  shotPhaseProgress,
} from "./bracket-stepper";

function progress(
  state: CaptureState,
  value = 0.4,
): CaptureProgress {
  return { state, progress: value, message: "…" };
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

describe("shotPhaseProgress", () => {
  it("is 1 for past shots and 0 for future ones", () => {
    const current = progress({ kind: "download", frame: 1, total: 3 });
    expect(shotPhaseProgress(0, 3, current)).toBe(1);
    expect(shotPhaseProgress(2, 3, current)).toBe(0);
  });

  it("advances within the active shot", () => {
    expect(
      shotPhaseProgress(
        1,
        3,
        progress({ kind: "set_exposure", frame: 1, total: 3 }),
      ),
    ).toBeLessThan(
      shotPhaseProgress(
        1,
        3,
        progress({ kind: "verify_file", frame: 1, total: 3 }),
      ),
    );
  });
});

describe("mergeStepStatus", () => {
  it("stays pending while shooting or restoring", () => {
    expect(mergeStepStatus(null)).toBe("pending");
    expect(mergeStepStatus(progress({ kind: "preparing" }))).toBe("pending");
    expect(
      mergeStepStatus(progress({ kind: "trigger", frame: 2, total: 5 })),
    ).toBe("pending");
    expect(mergeStepStatus(progress({ kind: "restoring" }))).toBe("pending");
  });

  it("marks merge current then completed", () => {
    expect(mergeStepStatus(progress({ kind: "merging" }))).toBe("current");
    expect(mergeStepStatus(progress({ kind: "done" }))).toBe("completed");
  });

  it("marks merge failed when the step name mentions fusion", () => {
    expect(
      mergeStepStatus(
        progress({ kind: "failed", step: "merging", error: "boom" }),
      ),
    ).toBe("failed");
    expect(
      mergeStepStatus(
        progress({ kind: "failed", step: "trigger", error: "boom" }),
      ),
    ).toBe("pending");
  });
});

describe("mergePhaseProgress", () => {
  it("maps the global gauge onto the merge step", () => {
    expect(mergePhaseProgress(progress({ kind: "merging" }, 0.88))).toBeCloseTo(
      0.08,
      2,
    );
    expect(mergePhaseProgress(progress({ kind: "merging" }, 0.93))).toBeCloseTo(
      0.5,
      2,
    );
    expect(mergePhaseProgress(progress({ kind: "done" }, 1))).toBe(1);
  });
});

describe("connectorRatio", () => {
  it("fills fully when the left step is done", () => {
    expect(connectorRatio(1)).toBe(1);
    expect(connectorRatio(0.58)).toBeCloseTo(0.58);
    expect(connectorRatio(0)).toBe(0);
  });
});

describe("connectorFill", () => {
  it("marks a completed left step as full toward the next", () => {
    expect(connectorFill("completed", "completed")).toBe("full");
    expect(connectorFill("completed", "current")).toBe("full");
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
