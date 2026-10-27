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
  fusion: FusionOptions;
};

export type AppError = {
  message: string;
  detail?: string;
};
