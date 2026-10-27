export type LogLevel = "error" | "warn" | "info" | "debug" | "trace";

export type Screen =
  | "connexion"
  | "prise-de-vue"
  | "resultat"
  | "historique"
  | "reglages";

export type AppStatus = {
  version: string;
  os: string;
  demo_mode: boolean;
  demo_forced_by_env: boolean;
  camera_available: boolean;
  log_level: LogLevel;
  log_dir: string;
};

export type AppError = {
  message: string;
  detail?: string;
};
