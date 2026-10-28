export type LogLevel = "error" | "warn" | "info" | "debug" | "trace";

export type Screen =
  | "connexion"
  | "prise-de-vue"
  | "resultat"
  | "historique"
  | "reglages";

export type FusionMethod = "mertens" | "debevec";

export type TonemapKind = "reinhard" | "mantiuk";

export type MertensWeights = {
  contrast: number;
  saturation: number;
  exposure: number;
};

export type FusionOptions = {
  align: boolean;
  method: FusionMethod;
  mertens: MertensWeights;
  tonemap: TonemapKind;
  output_jpeg: boolean;
  output_tiff: boolean;
  jpeg_quality: number;
};

export type FusionOutputs = {
  jpeg: string | null;
  tiff: string | null;
  image_count: number;
  method: FusionMethod;
  aligned: boolean;
};

export const DEFAULT_FUSION_OPTIONS: FusionOptions = {
  align: true,
  method: "mertens",
  mertens: { contrast: 1, saturation: 1, exposure: 1 },
  tonemap: "reinhard",
  output_jpeg: true,
  output_tiff: false,
  jpeg_quality: 95,
};

export type AppStatus = {
  version: string;
  os: string;
  demo_mode: boolean;
  demo_forced_by_env: boolean;
  camera_available: boolean;
  log_level: LogLevel;
  log_dir: string;
  sessions_dir: string;
  fusion: FusionOptions;
};

export type AppError = {
  message: string;
  detail?: string;
};

export type BracketRequest = {
  frame_count: number;
  step_ev: number;
  center_ev: number;
  session_name: string;
};

export type ExposureSlot = {
  index: number;
  relative_ev: number;
  shutter_seconds: number;
  shutter_label: string;
};

export type CameraSettingsDto = {
  mode: "M";
  aperture: number;
  iso: number;
  shutter_seconds: number;
  shutter_label: string;
};

export type BracketPlan = {
  request: BracketRequest;
  base: CameraSettingsDto;
  slots: ExposureSlot[];
};

export type CaptureState =
  | { kind: "idle" }
  | { kind: "preparing" }
  | { kind: "set_exposure"; frame: number; total: number }
  | { kind: "verify_exposure"; frame: number; total: number }
  | { kind: "trigger"; frame: number; total: number }
  | { kind: "wait_capture_done"; frame: number; total: number }
  | { kind: "download"; frame: number; total: number }
  | { kind: "verify_file"; frame: number; total: number }
  | { kind: "restoring" }
  | { kind: "merging" }
  | { kind: "done" }
  | { kind: "aborted" }
  | { kind: "failed"; step: string; error: string };

export type CaptureProgress = {
  state: CaptureState;
  progress: number;
  message: string;
};

export type CaptureSessionResult = {
  session_dir: string;
  manifest_path: string;
  fused_jpeg: string | null;
  state: CaptureState;
  originals: string[];
};

export const DEFAULT_BRACKET_REQUEST: BracketRequest = {
  frame_count: 5,
  step_ev: 1,
  center_ev: 0,
  session_name: "hdr",
};

export const FRAME_COUNT_OPTIONS = [3, 5, 7, 9] as const;
export const STEP_EV_OPTIONS = [
  1 / 3,
  0.5,
  2 / 3,
  1,
  1.5,
  2,
  3,
] as const;
