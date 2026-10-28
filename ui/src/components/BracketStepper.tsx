import { Camera, Check, DotsNine, X } from "@phosphor-icons/react";

import { useT } from "../i18n/react";
import {
  bracketStepStatus,
  connectorFill,
  formatRelativeEv,
  type StepStatus,
} from "../lib/bracket-stepper";
import type { CaptureProgress, ExposureSlot } from "../types";

type Props = {
  slots: ExposureSlot[];
  progress: CaptureProgress | null;
  iso: number;
  apertureLabel: string;
};

function statusLabelKey(status: StepStatus) {
  switch (status) {
    case "completed":
      return "studio.stepCompleted" as const;
    case "current":
      return "studio.stepInProgress" as const;
    case "failed":
      return "studio.stepFailed" as const;
    case "pending":
      return "studio.stepPending" as const;
  }
}

function CircleIcon({ status }: { status: StepStatus }) {
  if (status === "completed") {
    return <Check size={16} weight="bold" />;
  }
  if (status === "current") {
    return <Camera size={16} weight="fill" />;
  }
  if (status === "failed") {
    return <X size={16} weight="bold" />;
  }
  return <DotsNine size={16} weight="bold" />;
}

function circleClass(status: StepStatus): string {
  switch (status) {
    case "completed":
      return "bg-emerald-500 text-white";
    case "current":
      return "bg-blue-600 text-white ring-4 ring-blue-600/25";
    case "failed":
      return "bg-red-500 text-white";
    case "pending":
      return "border border-dashed border-foreground/25 bg-transparent text-muted";
  }
}

function pillClass(status: StepStatus): string {
  switch (status) {
    case "completed":
      return "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400";
    case "current":
      return "bg-blue-600/15 text-blue-600 dark:text-blue-400";
    case "failed":
      return "bg-red-500/15 text-red-600 dark:text-red-400";
    case "pending":
      return "bg-foreground/8 text-muted";
  }
}

function Connector({ fill }: { fill: "full" | "half" | "empty" }) {
  return (
    <div className="relative mx-1 mt-[15px] h-0.5 min-w-4 flex-1 overflow-hidden rounded-full bg-foreground/15">
      {fill === "full" ? (
        <span className="absolute inset-0 bg-emerald-500" />
      ) : null}
      {fill === "half" ? (
        <span className="absolute inset-y-0 left-0 w-1/2 bg-blue-600" />
      ) : null}
    </div>
  );
}

export function BracketStepper({
  slots,
  progress,
  iso,
  apertureLabel,
}: Props) {
  const t = useT();
  const statuses = slots.map((slot) =>
    bracketStepStatus(slot.index, slots.length, progress),
  );

  return (
    <div className="rounded-3xl border border-foreground/10 bg-foreground/[0.03] px-5 py-5">
      <p className="mb-4 text-xs font-medium uppercase tracking-wide text-muted">
        {t("studio.plan", { iso, aperture: apertureLabel })}
      </p>
      <ol className="flex overflow-x-auto pb-1">
        {slots.map((slot, i) => {
          const status = statuses[i] ?? "pending";
          const next = statuses[i + 1];
          return (
            <li key={slot.index} className="flex min-w-0 flex-1 items-start">
              <div className="flex min-w-[5.5rem] flex-col items-center text-center">
                <span
                  className={`flex size-8 items-center justify-center rounded-full ${circleClass(status)}`}
                  aria-current={status === "current" ? "step" : undefined}
                >
                  <CircleIcon status={status} />
                </span>
                <p className="mt-2 text-[10px] font-medium uppercase tracking-wider text-muted">
                  {t("studio.stepN", { n: slot.index + 1 })}
                </p>
                <p className="mt-0.5 text-sm font-semibold leading-tight">
                  {formatRelativeEv(slot.relative_ev)}
                </p>
                <p className="text-xs text-muted">{slot.shutter_label}</p>
                <span
                  className={`mt-2 rounded-full px-2 py-0.5 text-[10px] font-medium ${pillClass(status)}`}
                >
                  {t(statusLabelKey(status))}
                </span>
              </div>
              {next !== undefined ? (
                <Connector fill={connectorFill(status, next)} />
              ) : null}
            </li>
          );
        })}
      </ol>
      {progress?.message ? (
        <p className="mt-3 text-center text-xs text-muted">{progress.message}</p>
      ) : null}
    </div>
  );
}
