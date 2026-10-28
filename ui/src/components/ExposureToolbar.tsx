import { Button, ButtonGroup, Kbd, Separator, Toolbar } from "@heroui/react";

import { useT } from "../i18n/react";

type ExposureToolbarProps = {
  isoLabel: string;
  apertureLabel: string;
  shutterLabel: string;
  isDisabled?: boolean;
  onIsoStep: (delta: number) => void;
  onApertureStep: (delta: number) => void;
  onShutterStep: (delta: number) => void;
  onInteractionStart: () => void;
  onInteractionEnd: () => void;
};

function StepGroup({
  label,
  valueLabel,
  kbdMinus,
  kbdPlus,
  isDisabled,
  onMinus,
  onPlus,
  onInteractionStart,
  onInteractionEnd,
  minusAria,
  plusAria,
}: {
  label: string;
  valueLabel: string;
  kbdMinus: string;
  kbdPlus: string;
  isDisabled?: boolean;
  onMinus: () => void;
  onPlus: () => void;
  onInteractionStart: () => void;
  onInteractionEnd: () => void;
  minusAria: string;
  plusAria: string;
}) {
  return (
    <div className="flex flex-col items-center gap-1">
      <span className="text-[10px] uppercase tracking-wide text-white/60">
        {label}
      </span>
      <ButtonGroup isDisabled={isDisabled}>
        <Button
          size="sm"
          variant="secondary"
          aria-label={minusAria}
          onPressStart={() => {
            onInteractionStart();
            onMinus();
          }}
          onPressEnd={onInteractionEnd}
        >
          −
        </Button>
        <Button size="sm" variant="secondary" isDisabled>
          {valueLabel}
        </Button>
        <Button
          size="sm"
          variant="secondary"
          aria-label={plusAria}
          onPressStart={() => {
            onInteractionStart();
            onPlus();
          }}
          onPressEnd={onInteractionEnd}
        >
          +
        </Button>
      </ButtonGroup>
      <span className="inline-flex items-center gap-1 text-[10px] text-white/50">
        <Kbd>
          <Kbd.Content>{kbdMinus}</Kbd.Content>
        </Kbd>
        <Kbd>
          <Kbd.Content>{kbdPlus}</Kbd.Content>
        </Kbd>
      </span>
    </div>
  );
}

export function ExposureToolbar({
  isoLabel,
  apertureLabel,
  shutterLabel,
  isDisabled = false,
  onIsoStep,
  onApertureStep,
  onShutterStep,
  onInteractionStart,
  onInteractionEnd,
}: ExposureToolbarProps) {
  const t = useT();
  const iso = t("studio.iso");
  const aperture = t("studio.aperture");
  const shutter = t("studio.shutter");

  return (
    <Toolbar
      aria-label={t("studio.exposureAria")}
      isAttached
      className="bg-black/50 px-2 py-2 backdrop-blur-sm"
    >
      <div className="flex flex-wrap items-end justify-center gap-3">
        <StepGroup
          label={iso}
          valueLabel={isoLabel}
          kbdMinus="Q"
          kbdPlus="A"
          isDisabled={isDisabled}
          onMinus={() => onIsoStep(-1)}
          onPlus={() => onIsoStep(1)}
          onInteractionStart={onInteractionStart}
          onInteractionEnd={onInteractionEnd}
          minusAria={t("studio.minus", { label: iso })}
          plusAria={t("studio.plus", { label: iso })}
        />
        <Separator />
        <StepGroup
          label={aperture}
          valueLabel={apertureLabel}
          kbdMinus="W"
          kbdPlus="S"
          isDisabled={isDisabled}
          onMinus={() => onApertureStep(-1)}
          onPlus={() => onApertureStep(1)}
          onInteractionStart={onInteractionStart}
          onInteractionEnd={onInteractionEnd}
          minusAria={t("studio.minus", { label: aperture })}
          plusAria={t("studio.plus", { label: aperture })}
        />
        <Separator />
        <StepGroup
          label={shutter}
          valueLabel={shutterLabel}
          kbdMinus="E"
          kbdPlus="D"
          isDisabled={isDisabled}
          onMinus={() => onShutterStep(-1)}
          onPlus={() => onShutterStep(1)}
          onInteractionStart={onInteractionStart}
          onInteractionEnd={onInteractionEnd}
          minusAria={t("studio.minus", { label: shutter })}
          plusAria={t("studio.plus", { label: shutter })}
        />
      </div>
    </Toolbar>
  );
}
