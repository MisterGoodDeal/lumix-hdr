import { useEffect, useMemo, useState } from "react";
import {
  Button,
  Card,
  Description,
  Input,
  Label,
  ListBox,
  ProgressBar,
  Select,
  TextField,
  toast,
} from "@heroui/react";

import { ErrorDetails } from "../components/ErrorDetails";
import {
  cancelCapture,
  captureStateLabel,
  onCaptureProgress,
  previewBracketPlan,
  startCapture,
} from "../lib/capture";
import { isTauri, openPath, parseAppError, previewSrc } from "../lib/tauri";
import { useAppStore } from "../store/app-store";
import type {
  AppError,
  BracketPlan,
  BracketRequest,
  CaptureProgress,
  CaptureSessionResult,
} from "../types";
import {
  DEFAULT_BRACKET_REQUEST,
  FRAME_COUNT_OPTIONS,
  STEP_EV_OPTIONS,
} from "../types";

function formatStepEv(value: number): string {
  if (Math.abs(value - 1 / 3) < 0.001) {
    return "1/3 IL";
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

export function CaptureScreen() {
  const status = useAppStore((state) => state.status);
  const demoConnected = useAppStore((state) => state.demoConnected);
  const error = useAppStore((state) => state.error);

  const [request, setRequest] = useState<BracketRequest>(DEFAULT_BRACKET_REQUEST);
  const [plan, setPlan] = useState<BracketPlan | null>(null);
  const [planError, setPlanError] = useState<AppError | null>(null);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<CaptureProgress | null>(null);
  const [result, setResult] = useState<CaptureSessionResult | null>(null);
  const [runError, setRunError] = useState<AppError | null>(null);

  const ready = Boolean(status?.demo_mode && demoConnected);
  const activePlan = ready ? plan : null;
  const canShoot = ready && !running && activePlan !== null;

  const preview = previewSrc(result?.fused_jpeg ?? null);

  const fakeLiveStyle = useMemo(() => {
    const ev = request.center_ev;
    const light = Math.max(8, Math.min(72, 36 + ev * 10));
    return {
      background: `linear-gradient(160deg, hsl(210 20% ${light + 8}%), hsl(30 25% ${light - 4}%), hsl(210 15% ${light - 10}%))`,
    } as const;
  }, [request.center_ev]);

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
    if (!ready) {
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
        if (!cancelled) {
          setPlan(null);
          setPlanError(parseAppError(cause));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [request, ready]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
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
      }
      if (event.code === "Escape" && running) {
        event.preventDefault();
        void cancelCapture();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  async function runCapture() {
    if (!canShoot) {
      return;
    }
    setRunning(true);
    setProgress(null);
    setResult(null);
    setRunError(null);
    try {
      const session = await startCapture(request);
      setResult(session);
      setProgress({
        state: session.state,
        progress: 1,
        message: "Terminé",
      });
      toast.success("Prise de vue terminée", {
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
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>Prise de vue</Card.Title>
          <Card.Description>Chargement…</Card.Description>
        </Card.Header>
      </Card>
    );
  }

  if (!status.demo_mode) {
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>Prise de vue</Card.Title>
          <Card.Description>
            Activez le mode démo dans Réglages pour utiliser MockCamera (phase
            3). Le SDK LUMIX arrivera à la phase 4.
          </Card.Description>
        </Card.Header>
      </Card>
    );
  }

  if (!demoConnected) {
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>Prise de vue</Card.Title>
          <Card.Description>
            Connectez la démo depuis l’écran Connexion, puis revenez ici.
          </Card.Description>
        </Card.Header>
      </Card>
    );
  }

  return (
    <div className="flex max-w-5xl flex-col gap-4 lg:flex-row">
      <Card className="flex-1">
        <Card.Header>
          <Card.Title>Prise de vue</Card.Title>
          <Card.Description>
            Bracketing simulé (MockCamera). Espace = déclencher, Échap =
            annuler.
          </Card.Description>
        </Card.Header>
        <Card.Content className="flex flex-col gap-5">
          <TextField
            value={request.session_name}
            onChange={(session_name) => {
              setRequest({ ...request, session_name });
            }}
          >
            <Label>Nom de séance</Label>
            <Input />
            <Description>Lettres, chiffres, - et _</Description>
          </TextField>

          <Select
            selectedKey={String(request.frame_count)}
            onSelectionChange={(key) => {
              const value = Number(key);
              if (FRAME_COUNT_OPTIONS.includes(value as (typeof FRAME_COUNT_OPTIONS)[number])) {
                setRequest({ ...request, frame_count: value });
              }
            }}
          >
            <Label>Nombre de vues</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {FRAME_COUNT_OPTIONS.map((count) => (
                  <ListBox.Item
                    id={String(count)}
                    key={count}
                    textValue={`${count} vues`}
                  >
                    {count} vues
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>

          <Select
            selectedKey={String(request.step_ev)}
            onSelectionChange={(key) => {
              const value = Number(key);
              if (
                STEP_EV_OPTIONS.some((option) => Math.abs(option - value) < 0.001)
              ) {
                setRequest({ ...request, step_ev: value });
              }
            }}
          >
            <Label>Écart entre vues</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {STEP_EV_OPTIONS.map((step) => (
                  <ListBox.Item
                    id={String(step)}
                    key={step}
                    textValue={formatStepEv(step)}
                  >
                    {formatStepEv(step)}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>

          <Select
            selectedKey={String(request.center_ev)}
            onSelectionChange={(key) => {
              const value = Number(key);
              if (value >= -3 && value <= 3) {
                setRequest({ ...request, center_ev: value });
              }
            }}
          >
            <Label>Centre d’exposition</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {[-3, -2, -1, 0, 1, 2, 3].map((center) => (
                  <ListBox.Item
                    id={String(center)}
                    key={center}
                    textValue={formatCenterEv(center)}
                  >
                    {formatCenterEv(center)}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>

          {activePlan ? (
            <div className="rounded-lg border border-foreground/10 p-3 text-sm">
              <p className="mb-2 font-medium">
                Plan (mode M, ISO {activePlan.base.iso}, f/
                {activePlan.base.aperture})
              </p>
              <ul className="flex flex-col gap-1 text-muted">
                {activePlan.slots.map((slot) => (
                  <li key={slot.index}>
                    #{slot.index + 1} · {formatCenterEv(slot.relative_ev)} ·{" "}
                    {slot.shutter_label}
                  </li>
                ))}
              </ul>
            </div>
          ) : null}

          <div className="flex gap-3">
            <Button
              variant="primary"
              isDisabled={!canShoot}
              onPress={() => void runCapture()}
            >
              Déclencher
            </Button>
            <Button
              variant="secondary"
              isDisabled={!running}
              onPress={() => {
                void cancelCapture();
              }}
            >
              Annuler
            </Button>
          </div>

          {running || (progress && progress.progress > 0) ? (
            <ProgressBar
              value={Math.round((progress?.progress ?? 0) * 100)}
              minValue={0}
              maxValue={100}
            >
              <Label>{captureStateLabel(progress)}</Label>
              <ProgressBar.Track>
                <ProgressBar.Fill />
              </ProgressBar.Track>
              <ProgressBar.Output />
            </ProgressBar>
          ) : null}

          {!isTauri() ? (
            <Description>
              Hors Tauri, la prise de vue est simulée côté navigateur.
            </Description>
          ) : (
            <Description>
              Sessions : {status.sessions_dir}
            </Description>
          )}

          <ErrorDetails error={runError ?? planError ?? error} />
        </Card.Content>
      </Card>

      <Card className="flex-1">
        <Card.Header>
          <Card.Title>Aperçu</Card.Title>
          <Card.Description>
            {result
              ? "Résultat de la dernière séance"
              : "Live view factice (phase 5 pour le flux réel)"}
          </Card.Description>
        </Card.Header>
        <Card.Content className="flex flex-col gap-4">
          {preview ? (
            <img
              src={preview}
              alt="Résultat de la séance"
              className="max-h-[420px] w-full rounded-lg object-contain bg-black/40"
            />
          ) : (
            <div
              className="flex h-64 items-end rounded-lg p-4 text-sm text-white/90"
              style={fakeLiveStyle}
            >
              Aperçu démo · centre {formatCenterEv(request.center_ev)}
            </div>
          )}
          {result?.session_dir ? (
            <Button
              variant="secondary"
              onPress={() => {
                void openPath(result.session_dir);
              }}
            >
              Ouvrir la séance
            </Button>
          ) : null}
        </Card.Content>
      </Card>
    </div>
  );
}
