import { Circle } from "@phosphor-icons/react";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  Alert,
  Button,
  Chip,
  Description,
  Input,
  Label,
  Tabs,
  TextField,
  toast,
} from "@heroui/react";

import { BracketStepper } from "../components/BracketStepper";
import { ErrorDetails } from "../components/ErrorDetails";
import { ExposureToolbar } from "../components/ExposureToolbar";
import { useT } from "../i18n/react";
import {
  cancelCapture,
  onCaptureProgress,
  previewBracketPlan,
  startCapture,
} from "../lib/capture";
import {
  connectionHelpText,
  connectionStatusColor,
  connectionStatusLabel,
  osLabel,
} from "../lib/connection-status";
import {
  getCameraSettings,
  isTauri,
  onCameraSettingsChanged,
  onLiveViewFrame,
  openPath,
  parseAppError,
  previewSrc,
  setCameraSettings,
  setCaptureDefaults,
} from "../lib/tauri";
import { useAppStore } from "../store/app-store";
import type {
  AppError,
  BracketPlan,
  BracketRequest,
  CameraDefaults,
  CaptureProgress,
  CaptureSessionResult,
  ExposureCapabilities,
} from "../types";
import { FRAME_COUNT_OPTIONS, STEP_EV_OPTIONS } from "../types";

const SETTINGS_DEBOUNCE_MS = 200;
const EXPOSURE_KEYS = new Set(["q", "a", "w", "s", "e", "d"]);

function formatStepEv(value: number): string {
  if (Math.abs(value - 1 / 3) < 0.001) {
    return "1/3 IL";
  }
  if (Math.abs(value - 0.5) < 0.001) {
    return "0.5 IL";
  }
  if (Math.abs(value - 2 / 3) < 0.001) {
    return "2/3 IL";
  }
  if (Number.isInteger(value)) {
    return `${value} IL`;
  }
  return `${value} IL`;
}

function formatCenterEv(value: number): string {
  const sign = value > 0 ? "+" : "";
  return `${sign}${value} IL`;
}

function formatAperture(value: number): string {
  return `f/${value.toFixed(1)}`;
}

function nearestIndex(values: number[], current: number): number {
  if (values.length === 0) {
    return 0;
  }
  let best = 0;
  let bestErr = Math.abs(values[0] - current);
  for (let i = 1; i < values.length; i += 1) {
    const err = Math.abs(values[i] - current);
    if (err < bestErr) {
      best = i;
      bestErr = err;
    }
  }
  return best;
}

function base64ToObjectUrl(base64: string): string {
  const binary = window.atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  const blob = new Blob([bytes], { type: "image/jpeg" });
  return URL.createObjectURL(blob);
}

export function StudioScreen() {
  const t = useT();
  const status = useAppStore((state) => state.status);
  const cameraConnected = useAppStore((state) => state.cameraConnected);
  const connectedModel = useAppStore((state) => state.connectedModel);
  const error = useAppStore((state) => state.error);
  const connectCamera = useAppStore((state) => state.connectCamera);
  const disconnectCamera = useAppStore((state) => state.disconnectCamera);
  const startListeningDisconnect = useAppStore(
    (state) => state.startListeningDisconnect,
  );

  const [requestOverride, setRequestOverride] = useState<BracketRequest | null>(
    null,
  );
  const [plan, setPlan] = useState<BracketPlan | null>(null);
  const [planError, setPlanError] = useState<AppError | null>(null);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<CaptureProgress | null>(null);
  const [result, setResult] = useState<CaptureSessionResult | null>(null);
  const [runError, setRunError] = useState<AppError | null>(null);
  const [capabilities, setCapabilities] = useState<ExposureCapabilities | null>(
    null,
  );
  const [liveUrl, setLiveUrl] = useState<string | null>(null);
  const capabilitiesRef = useRef(capabilities);
  capabilitiesRef.current = capabilities;
  const runningRef = useRef(running);
  runningRef.current = running;
  const debounceTimerRef = useRef<number | null>(null);
  const pendingSettingsRef = useRef<CameraDefaults | null>(null);
  const ignoreRemoteUntilRef = useRef(0);
  const interactionDepthRef = useRef(0);
  const keysHeldRef = useRef(new Set<string>());

  const request = useMemo<BracketRequest>(() => {
    if (requestOverride) {
      return requestOverride;
    }
    return {
      session_name: status?.capture.session_name ?? "hdr",
      frame_count: status?.capture.frame_count ?? 5,
      step_ev: status?.capture.step_ev ?? 1,
      center_ev: status?.capture.center_ev ?? 0,
    };
  }, [requestOverride, status?.capture]);

  const ready = Boolean(status && cameraConnected);
  const canConnect = Boolean(
    status && (status.demo_mode || status.camera_available),
  );
  const activePlan = ready ? plan : null;
  const canShoot = ready && !running && activePlan !== null;
  // Live View prime : sinon le JPEG fusionné fige la preview après capture.
  const resultPreview = previewSrc(result?.fused_jpeg ?? null);
  const displayUrl = cameraConnected ? (liveUrl ?? resultPreview) : null;
  const showingResult = Boolean(cameraConnected && resultPreview && !liveUrl);
  const activeCapabilities = ready ? capabilities : null;

  const statusLabel = status
    ? connectionStatusLabel(status, cameraConnected)
    : "…";
  const statusColor = status
    ? connectionStatusColor(status, cameraConnected)
    : "default";

  function updateRequest(next: BracketRequest) {
    setRequestOverride(next);
  }

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void startListeningDisconnect().then((fn) => {
      if (!cancelled) {
        unlisten = fn;
      } else {
        fn();
      }
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [startListeningDisconnect]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    let currentUrl: string | null = null;
    void onLiveViewFrame((frame) => {
      if (cancelled) {
        return;
      }
      const next = base64ToObjectUrl(frame.jpeg_base64);
      if (currentUrl) {
        URL.revokeObjectURL(currentUrl);
      }
      currentUrl = next;
      setLiveUrl(next);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
      if (currentUrl) {
        URL.revokeObjectURL(currentUrl);
      }
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void onCaptureProgress((value) => {
      if (!cancelled) {
        setProgress(value);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    // Pendant le bracket la caméra est retirée du slot : ne pas relire / toaster.
    if (!ready || running) {
      return;
    }
    let cancelled = false;
    void getCameraSettings()
      .then((caps) => {
        if (!cancelled) {
          setCapabilities(caps);
        }
      })
      .catch((cause) => {
        if (cancelled || runningRef.current) {
          return;
        }
        const parsed = parseAppError(cause);
        // Course normale (déco / capture) — pas de spam toast.
        if (
          parsed.message.includes("Pas d’appareil") ||
          parsed.message.includes("Prise de vue")
        ) {
          return;
        }
        toast.danger(t("studio.readSettingsFail"), {
          description: parsed.message,
        });
      });
    return () => {
      cancelled = true;
    };
  }, [ready, connectedModel, running, t]);

  useEffect(() => {
    if (!ready || running) {
      return;
    }
    let cancelled = false;
    void previewBracketPlan(request)
      .then((next) => {
        if (!cancelled) {
          setPlan(next);
          setPlanError(null);
        }
      })
      .catch((cause) => {
        if (!cancelled && !runningRef.current) {
          setPlan(null);
          setPlanError(parseAppError(cause));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [request, ready, running]);

  useEffect(() => {
    if (!status || !ready) {
      return;
    }
    const handle = window.setTimeout(() => {
      void setCaptureDefaults({
        session_name: request.session_name,
        frame_count: request.frame_count,
        step_ev: request.step_ev,
        center_ev: request.center_ev,
      }).catch(() => undefined);
    }, 400);
    return () => window.clearTimeout(handle);
  }, [request, ready, status]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void onCameraSettingsChanged((settings) => {
      if (cancelled) {
        return;
      }
      if (Date.now() < ignoreRemoteUntilRef.current) {
        return;
      }
      if (pendingSettingsRef.current) {
        return;
      }
      setCapabilities((prev) => (prev ? { ...prev, settings } : prev));
      setPlan(null);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    return () => {
      if (debounceTimerRef.current !== null) {
        window.clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable)
      ) {
        return;
      }
      if (event.code === "Space" && canShoot) {
        event.preventDefault();
        void runCapture();
        return;
      }
      if (event.code === "Escape" && running) {
        event.preventDefault();
        void cancelCapture();
        return;
      }
      if (!ready || runningRef.current) {
        return;
      }
      const key = event.key.toLowerCase();
      if (!EXPOSURE_KEYS.has(key)) {
        return;
      }
      event.preventDefault();
      keysHeldRef.current.add(key);
      cancelPendingFlush();
      if (key === "q") {
        stepIso(-1);
      } else if (key === "a") {
        stepIso(1);
      } else if (key === "w") {
        stepAperture(-1);
      } else if (key === "s") {
        stepAperture(1);
      } else if (key === "e") {
        stepShutter(-1);
      } else if (key === "d") {
        stepShutter(1);
      }
    }

    function onKeyUp(event: KeyboardEvent) {
      const key = event.key.toLowerCase();
      if (!EXPOSURE_KEYS.has(key)) {
        return;
      }
      keysHeldRef.current.delete(key);
      if (
        keysHeldRef.current.size === 0 &&
        interactionDepthRef.current === 0
      ) {
        armFlushAfterIdle();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  });

  function cancelPendingFlush() {
    if (debounceTimerRef.current !== null) {
      window.clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }
  }

  function armFlushAfterIdle() {
    const caps = capabilitiesRef.current;
    if (!caps || runningRef.current) {
      return;
    }
    pendingSettingsRef.current = {
      iso: caps.settings.iso,
      aperture: caps.settings.aperture,
      shutter_seconds: caps.settings.shutter_seconds,
    };
    ignoreRemoteUntilRef.current = Date.now() + SETTINGS_DEBOUNCE_MS + 500;
    cancelPendingFlush();
    debounceTimerRef.current = window.setTimeout(() => {
      void flushPendingSettings();
    }, SETTINGS_DEBOUNCE_MS);
  }

  function onInteractionStart() {
    interactionDepthRef.current += 1;
    cancelPendingFlush();
  }

  function onInteractionEnd() {
    interactionDepthRef.current = Math.max(0, interactionDepthRef.current - 1);
    if (
      interactionDepthRef.current === 0 &&
      keysHeldRef.current.size === 0
    ) {
      armFlushAfterIdle();
    }
  }

  async function flushPendingSettings() {
    const pending = pendingSettingsRef.current;
    pendingSettingsRef.current = null;
    debounceTimerRef.current = null;
    if (!pending || runningRef.current) {
      return;
    }
    const toastId = toast.info(t("studio.sending"), {
      isLoading: true,
      timeout: 0,
    });
    try {
      const caps = await setCameraSettings(pending);
      toast.close(toastId);
      setCapabilities(caps);
      setPlan(null);
    } catch (cause) {
      toast.close(toastId);
      toast.danger(t("studio.settingsRejected"), {
        description: parseAppError(cause).message,
      });
      try {
        const caps = await getCameraSettings();
        setCapabilities(caps);
      } catch {
        // ignore
      }
    }
  }

  function applyExposurePatchLocal(patch: {
    iso?: number;
    aperture?: number;
    shutter_seconds?: number;
  }) {
    const current = capabilitiesRef.current;
    if (!current || runningRef.current) {
      return;
    }
    const settings = { ...current.settings };
    if (patch.iso !== undefined) {
      settings.iso = patch.iso;
    }
    if (patch.aperture !== undefined) {
      settings.aperture = patch.aperture;
    }
    if (patch.shutter_seconds !== undefined) {
      const idx = nearestIndex(current.shutter_seconds, patch.shutter_seconds);
      settings.shutter_seconds =
        current.shutter_seconds[idx] ?? patch.shutter_seconds;
      settings.shutter_label =
        current.shutter_labels[idx] ?? settings.shutter_label;
    }
    const next = { ...current, settings };
    capabilitiesRef.current = next;
    pendingSettingsRef.current = {
      iso: settings.iso,
      aperture: settings.aperture,
      shutter_seconds: settings.shutter_seconds,
    };
    setCapabilities(next);
  }

  function stepIso(delta: number) {
    const current = capabilitiesRef.current;
    if (!current) {
      return;
    }
    const idx = nearestIndex(current.iso_values, current.settings.iso);
    const next = current.iso_values[idx + delta];
    if (next === undefined) {
      return;
    }
    applyExposurePatchLocal({ iso: next });
  }

  function stepAperture(delta: number) {
    const current = capabilitiesRef.current;
    if (!current) {
      return;
    }
    const idx = nearestIndex(
      current.aperture_values,
      current.settings.aperture,
    );
    const next = current.aperture_values[idx + delta];
    if (next === undefined) {
      return;
    }
    applyExposurePatchLocal({ aperture: next });
  }

  function stepShutter(delta: number) {
    const current = capabilitiesRef.current;
    if (!current) {
      return;
    }
    const idx = nearestIndex(
      current.shutter_seconds,
      current.settings.shutter_seconds,
    );
    // Table triée rapide → lent : + = plus lent (plus d'exposition).
    const next = current.shutter_seconds[idx + delta];
    if (next === undefined) {
      return;
    }
    applyExposurePatchLocal({ shutter_seconds: next });
  }

  async function runCapture() {
    if (!canShoot) {
      return;
    }
    if (debounceTimerRef.current !== null || pendingSettingsRef.current) {
      cancelPendingFlush();
      await flushPendingSettings();
    }
    setRunning(true);
    setProgress(null);
    setResult(null);
    setRunError(null);
    // Invalide l’ancienne frame LV pour ne pas garder un gel pendant le bracket.
    setLiveUrl((prev) => {
      if (prev) {
        URL.revokeObjectURL(prev);
      }
      return null;
    });
    // Live View : start_capture pause/reprend côté Rust.
    try {
      const session = await startCapture(request);
      setResult(session);
      setProgress({
        state: session.state,
        progress: 1,
        message: "Terminé",
      });
      toast.success(t("studio.sessionDone"), {
        description: session.session_dir,
      });
    } catch (cause) {
      const parsed = parseAppError(cause);
      setRunError(parsed);
      toast.danger(parsed.message, { description: parsed.detail });
    } finally {
      setRunning(false);
    }
  }

  if (!status) {
    return <p className="text-muted">{t("common.loading")}</p>;
  }

  const stepperProgress: CaptureProgress | null =
    running
      ? progress
      : result?.state.kind === "done" &&
          activePlan !== null &&
          result.originals.length === activePlan.slots.length
        ? { state: { kind: "done" }, progress: 1, message: "" }
        : null;

  return (
    <div className="flex flex-col gap-4">
      {activePlan ? (
        <BracketStepper
          slots={activePlan.slots}
          progress={stepperProgress}
          iso={activePlan.base.iso}
          apertureLabel={formatAperture(activePlan.base.aperture)}
        />
      ) : null}

      <div className="flex flex-col gap-4 xl:flex-row">
        <section className="flex min-w-0 flex-1 flex-col gap-4">
        <div className="relative min-h-[420px] overflow-hidden rounded-xl bg-black/50">
          {displayUrl ? (
            <img
              src={displayUrl}
              alt={
                showingResult ? t("studio.resultAlt") : t("studio.liveViewAlt")
              }
              className="h-full max-h-[70vh] w-full object-contain"
            />
          ) : (
            <div className="flex h-full min-h-[420px] flex-col items-center justify-center gap-3 px-6 text-center text-white/80">
              <p className="text-lg font-medium">
                {cameraConnected
                  ? t("studio.liveViewPending")
                  : t("studio.plugToStart")}
              </p>
              <p className="max-w-md text-sm text-white/60">
                {connectionHelpText(status)}
              </p>
              {!cameraConnected ? (
                <Button
                  variant="primary"
                  isDisabled={!canConnect}
                  onPress={() => void connectCamera()}
                >
                  {t("common.connect")}
                </Button>
              ) : null}
            </div>
          )}
        </div>

        {cameraConnected && activeCapabilities ? (
          <div className="flex justify-center px-1">
            <ExposureToolbar
              isoLabel={String(activeCapabilities.settings.iso)}
              apertureLabel={formatAperture(
                activeCapabilities.settings.aperture,
              )}
              shutterLabel={activeCapabilities.settings.shutter_label}
              isDisabled={running}
              onIsoStep={stepIso}
              onApertureStep={stepAperture}
              onShutterStep={stepShutter}
              onInteractionStart={onInteractionStart}
              onInteractionEnd={onInteractionEnd}
            />
          </div>
        ) : null}
      </section>

      <aside className="flex w-full flex-col gap-4 xl:w-[26rem]">
        <div className="flex flex-wrap items-center gap-2">
          <Chip size="sm" color={statusColor} variant="primary">
            <Circle weight="fill" size={6} aria-hidden />
            <Chip.Label>{statusLabel}</Chip.Label>
          </Chip>
          <Chip size="sm" variant="primary">
            <Circle weight="fill" size={6} aria-hidden />
            <Chip.Label>{osLabel(status.os)}</Chip.Label>
          </Chip>
          {connectedModel ? (
            <Chip size="sm" color="success" variant="primary">
              <Circle weight="fill" size={6} aria-hidden />
              <Chip.Label>{connectedModel}</Chip.Label>
            </Chip>
          ) : null}
        </div>

        {!status.demo_mode && !status.camera_available ? (
          <Alert status="warning">
            <Alert.Indicator />
            <Alert.Content>
              {t("studio.windowsOnly")}
            </Alert.Content>
          </Alert>
        ) : null}

        <div className="flex gap-2">
          {cameraConnected ? (
            <Button
              variant="secondary"
              isDisabled={running}
              onPress={() => {
                setResult(null);
                setCapabilities(null);
                setPlan(null);
                setLiveUrl((prev) => {
                  if (prev) {
                    URL.revokeObjectURL(prev);
                  }
                  return null;
                });
                void disconnectCamera();
              }}
            >
              {t("common.disconnect")}
            </Button>
          ) : (
            <Button
              variant="primary"
              isDisabled={!canConnect}
              onPress={() => {
                setResult(null);
                setLiveUrl((prev) => {
                  if (prev) {
                    URL.revokeObjectURL(prev);
                  }
                  return null;
                });
                void connectCamera();
              }}
            >
              {t("common.connect")}
            </Button>
          )}
        </div>

        <TextField
          value={request.session_name}
          isDisabled={!ready || running}
          onChange={(session_name) => {
            updateRequest({ ...request, session_name });
          }}
        >
          <Label>{t("studio.sessionName")}</Label>
          <Input />
          <Description>{t("studio.sessionNameHint")}</Description>
        </TextField>

        <div className="flex flex-col gap-2">
          <Label>{t("studio.frameCount")}</Label>
          <Tabs
            selectedKey={String(request.frame_count)}
            onSelectionChange={(key) => {
              const value = Number(key);
              if (
                FRAME_COUNT_OPTIONS.includes(
                  value as (typeof FRAME_COUNT_OPTIONS)[number],
                )
              ) {
                updateRequest({ ...request, frame_count: value });
              }
            }}
          >
            <Tabs.ListContainer className="min-h-11 py-1">
              <Tabs.List
                aria-label={t("studio.frameCount")}
                className="min-h-9 gap-1 px-1"
              >
                {FRAME_COUNT_OPTIONS.map((count) => (
                  <Tabs.Tab
                    id={String(count)}
                    key={count}
                    isDisabled={!ready || running}
                    className="whitespace-nowrap px-3 py-2"
                  >
                    {count}
                    <Tabs.Indicator />
                  </Tabs.Tab>
                ))}
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </div>

        <div className="flex flex-col gap-2">
          <Label>{t("studio.stepEv")}</Label>
          <Tabs
            selectedKey={String(request.step_ev)}
            onSelectionChange={(key) => {
              const value = Number(key);
              if (
                STEP_EV_OPTIONS.some(
                  (option) => Math.abs(option - value) < 0.001,
                )
              ) {
                updateRequest({ ...request, step_ev: value });
              }
            }}
          >
            <Tabs.ListContainer className="min-h-11 py-1">
              <Tabs.List
                aria-label={t("studio.stepEv")}
                className="min-h-9 gap-1 px-1"
              >
                {STEP_EV_OPTIONS.map((step) => (
                  <Tabs.Tab
                    id={String(step)}
                    key={step}
                    isDisabled={!ready || running}
                    className="whitespace-nowrap px-3 py-2"
                  >
                    {formatStepEv(step)}
                    <Tabs.Indicator />
                  </Tabs.Tab>
                ))}
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </div>

        <div className="flex flex-col gap-2">
          <Label>{t("studio.centerEv")}</Label>
          <Tabs
            selectedKey={String(request.center_ev)}
            onSelectionChange={(key) => {
              const value = Number(key);
              if (value >= -3 && value <= 3) {
                updateRequest({ ...request, center_ev: value });
              }
            }}
          >
            <Tabs.ListContainer className="min-h-11 py-1">
              <Tabs.List
                aria-label={t("studio.centerEv")}
                className="min-h-9 gap-1 px-1"
              >
                {[-3, -2, -1, 0, 1, 2, 3].map((center) => (
                  <Tabs.Tab
                    id={String(center)}
                    key={center}
                    isDisabled={!ready || running}
                    className="whitespace-nowrap px-3 py-2"
                  >
                    {formatCenterEv(center)}
                    <Tabs.Indicator />
                  </Tabs.Tab>
                ))}
              </Tabs.List>
            </Tabs.ListContainer>
          </Tabs>
        </div>

        <div className="flex gap-2">
          <Button
            variant="primary"
            isDisabled={!canShoot}
            onPress={() => void runCapture()}
          >
            {t("studio.trigger")}
          </Button>
          <Button
            variant="secondary"
            isDisabled={!running}
            onPress={() => {
              void cancelCapture();
            }}
          >
            {t("common.cancel")}
          </Button>
        </div>

        {result?.session_dir ? (
          <Button
            variant="secondary"
            onPress={() => {
              void openPath(result.session_dir);
            }}
          >
            {t("studio.openSession")}
          </Button>
        ) : null}

        {!isTauri() ? (
          <Description>{t("studio.browserPreview")}</Description>
        ) : null}

        <ErrorDetails error={runError ?? planError ?? error} />
      </aside>
      </div>
    </div>
  );
}
