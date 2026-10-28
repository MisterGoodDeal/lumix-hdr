import { Fragment } from "react";
import { ProgressBar } from "@heroui/react";
import { Camera, Check, DotsNine, Images, X } from "@phosphor-icons/react";

import { useT } from "../i18n/react";
import {
  bracketStepStatus,
  connectorRatio,
  formatRelativeEv,
  mergePhaseProgress,
  mergeStepStatus,
  shotPhaseProgress,
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

function CircleIcon({
  status,
  kind,
}: {
  status: StepStatus;
  kind: "shot" | "merge";
}) {
  if (status === "completed") {
    return <Check size={16} weight="bold" />;
  }
  if (status === "current") {
    return kind === "merge" ? (
      <Images size={16} weight="fill" />
    ) : (
      <Camera size={16} weight="fill" />
    );
  }
  if (status === "failed") {
    return <X size={16} weight="bold" />;
  }
  return kind === "merge" ? (
    <Images size={16} weight="bold" />
  ) : (
    <DotsNine size={16} weight="bold" />
  );
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

function Connector({
  ratio,
  active,
}: {
  ratio: number;
  active: boolean;
}) {
  const pct = Math.round(connectorRatio(ratio) * 100);
  return (
    <div
      className="relative mt-[15px] h-1.5 min-w-3 w-full overflow-hidden rounded-full bg-foreground/15"
      aria-hidden
    >
      <span
        className={`absolute inset-y-0 left-0 rounded-full transition-[width] duration-300 ease-out ${
          active ? "bg-blue-600" : "bg-emerald-500"
        }`}
        style={{ width: `${pct}%` }}
      />
    </div>
  );
}

function StepCircle({
  status,
  kind,
}: {
  status: StepStatus;
  kind: "shot" | "merge";
}) {
  return (
    <span
      className={`flex size-8 shrink-0 items-center justify-center rounded-full ${circleClass(status)}`}
      aria-current={status === "current" ? "step" : undefined}
    >
      <CircleIcon status={status} kind={kind} />
    </span>
  );
}

function StepMiniBar({
  status,
  phase,
  label,
}: {
  status: StepStatus;
  phase: number;
  label: string;
}) {
  if (status !== "current") {
    return null;
  }
  const value = Math.round(Math.min(1, Math.max(0, phase)) * 100);
  return (
    <ProgressBar
      aria-label={label}
      value={value}
      minValue={0}
      maxValue={100}
      className="mt-2 w-full max-w-[6.5rem]"
    >
      <ProgressBar.Track className="h-1.5">
        <ProgressBar.Fill className="transition-[width] duration-300 ease-out" />
      </ProgressBar.Track>
    </ProgressBar>
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
  const phases = slots.map((slot) =>
    shotPhaseProgress(slot.index, slots.length, progress),
  );
  const mergeStatus = mergeStepStatus(progress);
  const mergePhase = mergePhaseProgress(progress);
  const lastShotPhase = phases[phases.length - 1] ?? 0;

  return (
    <div className="w-full rounded-3xl border border-foreground/10 bg-foreground/[0.03] px-5 py-5">
      <p className="mb-4 text-xs font-medium uppercase tracking-wide text-muted">
        {t("studio.plan", { iso, aperture: apertureLabel })}
      </p>
      <ol className="flex w-full min-w-0 items-start overflow-x-auto pb-1">
        {slots.map((slot, i) => {
          const status = statuses[i] ?? "pending";
          const phase = phases[i] ?? 0;
          const nextStatus = statuses[i + 1];
          return (
            <Fragment key={slot.index}>
              <li className="flex w-[5.75rem] shrink-0 flex-col items-center text-center sm:w-28">
                <StepCircle status={status} kind="shot" />
                <p className="mt-2 text-[10px] font-medium uppercase tracking-wider text-muted">
                  {t("studio.stepN", { n: slot.index + 1 })}
                </p>
                <p className="mt-0.5 text-sm font-semibold leading-tight">
                  {formatRelativeEv(slot.relative_ev)}
                </p>
                <p className="text-xs text-muted">{slot.shutter_label}</p>
                <StepMiniBar
                  status={status}
                  phase={phase}
                  label={t("studio.stepProgress", { n: slot.index + 1 })}
                />
                <span
                  className={`mt-2 max-w-full truncate rounded-full px-2 py-0.5 text-[10px] font-medium ${pillClass(status)}`}
                >
                  {t(statusLabelKey(status))}
                </span>
              </li>
              {nextStatus !== undefined ? (
                <li className="flex min-w-3 flex-1 basis-0 items-start self-stretch px-1">
                  <Connector
                    ratio={phase}
                    active={status === "current"}
                  />
                </li>
              ) : null}
            </Fragment>
          );
        })}

        {slots.length > 0 ? (
          <>
            <li className="flex min-w-3 flex-1 basis-0 items-start self-stretch px-1">
              <Connector
                ratio={lastShotPhase}
                active={statuses[statuses.length - 1] === "current"}
              />
            </li>
            <li className="flex w-[5.75rem] shrink-0 flex-col items-center text-center sm:w-28">
              <StepCircle status={mergeStatus} kind="merge" />
              <p className="mt-2 text-[10px] font-medium uppercase tracking-wider text-muted">
                {t("studio.mergeStep")}
              </p>
              <p className="mt-0.5 text-sm font-semibold leading-tight">
                {t("studio.mergeStepTitle")}
              </p>
              <p className="text-xs text-muted">{t("studio.mergeStepHint")}</p>
              <StepMiniBar
                status={mergeStatus}
                phase={mergePhase}
                label={t("studio.mergeStepTitle")}
              />
              <span
                className={`mt-2 max-w-full truncate rounded-full px-2 py-0.5 text-[10px] font-medium ${pillClass(mergeStatus)}`}
              >
                {t(statusLabelKey(mergeStatus))}
              </span>
            </li>
          </>
        ) : null}
      </ol>
      {progress?.message ? (
        <p className="mt-3 text-center text-xs text-muted">{progress.message}</p>
      ) : null}
    </div>
  );
}
