import { useEffect, useState } from "react";
import {
  Button,
  Card,
  Description,
  Label,
  ListBox,
  ProgressBar,
  Select,
  Switch,
  toast,
} from "@heroui/react";

import { ErrorDetails } from "../components/ErrorDetails";
import { fusionMethodLabel, jpegCountLabel, tonemapLabel } from "../lib/fusion";
import {
  cancelFusion,
  isTauri,
  listJpegFolder,
  mergeImageFolder,
  onFusionProgress,
  openPath,
  parseAppError,
  pickImageFolder,
  previewSrc,
} from "../lib/tauri";
import { useAppStore } from "../store/app-store";
import type { AppError, FusionMethod, FusionOutputs, TonemapKind } from "../types";

export function ResultScreen() {
  const status = useAppStore((state) => state.status);
  const error = useAppStore((state) => state.error);
  const setFusionOptions = useAppStore((state) => state.setFusionOptions);
  const [folder, setFolder] = useState<string | null>(null);
  const [files, setFiles] = useState<string[]>([]);
  const [progress, setProgress] = useState(0);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<FusionOutputs | null>(null);
  const [mergeError, setMergeError] = useState<AppError | null>(null);

  const fusion = status?.fusion;

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;
    void onFusionProgress((value) => {
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

  if (!status || !fusion) {
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>Résultat</Card.Title>
          <Card.Description>Chargement…</Card.Description>
        </Card.Header>
      </Card>
    );
  }

  const canMerge = Boolean(folder) && files.length >= 2 && !running;
  const preview = previewSrc(result?.jpeg ?? null);

  async function chooseFolder() {
    const selected = await pickImageFolder();
    if (!selected) {
      return;
    }
    setFolder(selected);
    setResult(null);
    setProgress(0);
    try {
      const listed = await listJpegFolder(selected);
      setFiles(listed);
    } catch {
      setFiles([]);
    }
  }

  async function runMerge() {
    if (!folder || !fusion) {
      return;
    }
    setRunning(true);
    setProgress(0);
    setResult(null);
    setMergeError(null);
    try {
      const outputs = await mergeImageFolder(folder, fusion);
      setResult(outputs);
      setProgress(1);
    } catch (cause) {
      const parsed = parseAppError(cause);
      setMergeError(parsed);
      toast.danger(parsed.message, { description: parsed.detail });
    } finally {
      setRunning(false);
    }
  }

  return (
    <div className="flex max-w-5xl flex-col gap-4 lg:flex-row">
      <Card className="flex-1">
        <Card.Header>
          <Card.Title>Résultat</Card.Title>
          <Card.Description>
            Fusionnez un dossier de JPEG sans appareil. La comparaison
            avant/après détaillée arrivera plus tard.
          </Card.Description>
        </Card.Header>
        <Card.Content className="flex flex-col gap-5">
          <div className="flex flex-col gap-2">
            <Button variant="secondary" onPress={() => void chooseFolder()}>
              Choisir un dossier
            </Button>
            <p className="break-all text-sm text-muted">
              {folder ?? "Aucun dossier sélectionné"}
            </p>
            {folder ? (
              <Description>{jpegCountLabel(files.length)}</Description>
            ) : null}
          </div>

          <Switch
            isSelected={fusion.align}
            onChange={(align) => {
              void setFusionOptions({ ...fusion, align });
            }}
          >
            <Switch.Content>
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
              Aligner les images (AlignMTB)
            </Switch.Content>
          </Switch>

          <Select
            selectedKey={fusion.method}
            onSelectionChange={(key) => {
              if (key === "mertens" || key === "debevec") {
                void setFusionOptions({
                  ...fusion,
                  method: key as FusionMethod,
                });
              }
            }}
          >
            <Label>Méthode de fusion</Label>
            <Select.Trigger>
              <Select.Value />
              <Select.Indicator />
            </Select.Trigger>
            <Select.Popover>
              <ListBox>
                {(["mertens", "debevec"] as const).map((method) => (
                  <ListBox.Item
                    id={method}
                    key={method}
                    textValue={fusionMethodLabel(method)}
                  >
                    {fusionMethodLabel(method)}
                    <ListBox.ItemIndicator />
                  </ListBox.Item>
                ))}
              </ListBox>
            </Select.Popover>
          </Select>

          {fusion.method === "debevec" ? (
            <Select
              selectedKey={fusion.tonemap}
              onSelectionChange={(key) => {
                if (key === "reinhard" || key === "mantiuk") {
                  void setFusionOptions({
                    ...fusion,
                    tonemap: key as TonemapKind,
                  });
                }
              }}
            >
              <Label>Tone mapping</Label>
              <Select.Trigger>
                <Select.Value />
                <Select.Indicator />
              </Select.Trigger>
              <Select.Popover>
                <ListBox>
                  {(["reinhard", "mantiuk"] as const).map((kind) => (
                    <ListBox.Item
                      id={kind}
                      key={kind}
                      textValue={tonemapLabel(kind)}
                    >
                      {tonemapLabel(kind)}
                      <ListBox.ItemIndicator />
                    </ListBox.Item>
                  ))}
                </ListBox>
              </Select.Popover>
            </Select>
          ) : null}

          <Switch
            isSelected={fusion.output_jpeg}
            onChange={(output_jpeg) => {
              void setFusionOptions({ ...fusion, output_jpeg });
            }}
          >
            <Switch.Content>
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
              JPEG qualité 95
            </Switch.Content>
          </Switch>

          <Switch
            isSelected={fusion.output_tiff}
            onChange={(output_tiff) => {
              void setFusionOptions({ ...fusion, output_tiff });
            }}
          >
            <Switch.Content>
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
              TIFF 16 bits
            </Switch.Content>
          </Switch>

          <div className="flex gap-3">
            <Button
              variant="primary"
              isDisabled={!canMerge}
              onPress={() => void runMerge()}
            >
              Fusionner
            </Button>
            <Button
              variant="secondary"
              isDisabled={!running}
              onPress={() => {
                void cancelFusion();
              }}
            >
              Annuler
            </Button>
          </div>

          {running || progress > 0 ? (
            <ProgressBar
              value={Math.round(progress * 100)}
              minValue={0}
              maxValue={100}
            >
              <Label>Progression</Label>
              <ProgressBar.Track>
                <ProgressBar.Fill />
              </ProgressBar.Track>
              <ProgressBar.Output />
            </ProgressBar>
          ) : null}

          {!isTauri() ? (
            <Description>
              Dans le navigateur, le choix de dossier et la fusion sont
              simulés. Relancez via Yarn Tauri pour un vrai OpenCV.
            </Description>
          ) : null}

          <ErrorDetails error={mergeError ?? error} />
        </Card.Content>
      </Card>

      <Card className="flex-1">
        <Card.Header>
          <Card.Title>Aperçu</Card.Title>
          <Card.Description>
            {result
              ? `${result.image_count} images · ${fusionMethodLabel(result.method)}`
              : "Aucun résultat pour l'instant"}
          </Card.Description>
        </Card.Header>
        <Card.Content className="flex flex-col gap-4">
          {preview ? (
            <img
              src={preview}
              alt="Résultat HDR fusionné"
              className="max-h-[480px] w-full rounded-lg object-contain bg-black/40"
            />
          ) : result?.jpeg ? (
            <p className="text-sm text-muted">
              Fichier écrit : {result.jpeg}
            </p>
          ) : (
            <p className="text-muted">
              Choisissez un dossier contenant au moins deux JPEG, puis lancez
              la fusion.
            </p>
          )}
          {result?.jpeg || result?.tiff || folder ? (
            <Button
              variant="secondary"
              onPress={() => {
                if (folder) {
                  void openPath(folder);
                }
              }}
            >
              Ouvrir le dossier
            </Button>
          ) : null}
        </Card.Content>
      </Card>
    </div>
  );
}
