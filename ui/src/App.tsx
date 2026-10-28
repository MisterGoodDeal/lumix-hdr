import type { Icon } from "@phosphor-icons/react";
import {
  Camera,
  ClockCounterClockwise,
  Image,
  Sliders,
} from "@phosphor-icons/react";
import { useEffect } from "react";
import { Button, Spinner, Toast } from "@heroui/react";

import type { TranslationKey } from "./i18n";
import { useT } from "./i18n/react";
import { PlaceholderScreen } from "./screens/PlaceholderScreen";
import { ResultScreen } from "./screens/ResultScreen";
import { SettingsScreen } from "./screens/SettingsScreen";
import { StudioScreen } from "./screens/StudioScreen";
import { useAppStore } from "./store/app-store";
import type { Screen } from "./types";

const SCREENS: {
  id: Screen;
  labelKey: TranslationKey;
  Icon: Icon;
}[] = [
  { id: "studio", labelKey: "nav.studio", Icon: Camera },
  { id: "resultat", labelKey: "nav.result", Icon: Image },
  { id: "historique", labelKey: "nav.history", Icon: ClockCounterClockwise },
  { id: "reglages", labelKey: "nav.settings", Icon: Sliders },
];

export default function App() {
  const t = useT();
  const loading = useAppStore((state) => state.loading);
  const screen = useAppStore((state) => state.screen);
  const setScreen = useAppStore((state) => state.setScreen);
  const loadStatus = useAppStore((state) => state.loadStatus);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      <Toast.Provider placement="top start" />
      <main className="flex flex-1 flex-col overflow-auto px-4 pb-24 pt-4">
        {loading ? (
          <div className="flex items-center gap-3">
            <Spinner />
            <span>{t("common.loading")}</span>
          </div>
        ) : (
          <>
            {screen === "studio" ? <StudioScreen /> : null}
            {screen === "resultat" ? <ResultScreen /> : null}
            {screen === "historique" ? (
              <PlaceholderScreen
                title={t("nav.history")}
                description={t("app.historySoon")}
              />
            ) : null}
            {screen === "reglages" ? <SettingsScreen /> : null}
          </>
        )}
      </main>

      {!loading ? (
        <nav
          aria-label={t("nav.aria")}
          className="fixed inset-x-0 bottom-0 z-40 border-t border-foreground/10 bg-background/95 backdrop-blur-md"
        >
          <div className="mx-auto flex max-w-3xl items-stretch justify-around gap-1 px-2 py-2">
            {SCREENS.map(({ id, labelKey, Icon }) => {
              const selected = screen === id;
              return (
                <Button
                  key={id}
                  variant={selected ? "primary" : "ghost"}
                  size="sm"
                  aria-current={selected ? "page" : undefined}
                  className="h-auto min-w-0 flex-1 flex-col gap-1 px-2 py-2"
                  onPress={() => setScreen(id)}
                >
                  <Icon
                    size={32}
                    weight={selected ? "fill" : "regular"}
                    aria-hidden
                  />
                  <span className="text-[11px] leading-none">
                    {t(labelKey)}
                  </span>
                </Button>
              );
            })}
          </div>
        </nav>
      ) : null}
    </div>
  );
}
