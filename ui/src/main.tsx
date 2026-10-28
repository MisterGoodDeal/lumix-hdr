import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "./App";
import { applyLocalePreference } from "./i18n";
import { applyThemePreference } from "./theme";
import "./index.css";

applyLocalePreference("system");
applyThemePreference("system");

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Root element missing");
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
