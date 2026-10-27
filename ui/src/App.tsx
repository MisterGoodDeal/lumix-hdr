import { useEffect } from "react";
import { Spinner, Tabs, Toast } from "@heroui/react";

import { ConnectionScreen } from "./screens/ConnectionScreen";
import { PlaceholderScreen } from "./screens/PlaceholderScreen";
import { SettingsScreen } from "./screens/SettingsScreen";
import { useAppStore } from "./store/app-store";
import type { Screen } from "./types";

const SCREENS: { id: Screen; label: string }[] = [
  { id: "connexion", label: "Connexion" },
  { id: "prise-de-vue", label: "Prise de vue" },
  { id: "resultat", label: "Résultat" },
  { id: "historique", label: "Historique" },
  { id: "reglages", label: "Réglages" },
];

export default function App() {
  const loading = useAppStore((state) => state.loading);
  const screen = useAppStore((state) => state.screen);
  const setScreen = useAppStore((state) => state.setScreen);
  const loadStatus = useAppStore((state) => state.loadStatus);

  useEffect(() => {
    document.documentElement.classList.add("dark");
    document.documentElement.setAttribute("data-theme", "dark");
    void loadStatus();
  }, [loadStatus]);

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      <Toast.Provider placement="top end" />
      <header className="border-b border-foreground/10 px-6 py-4">
        <h1 className="text-xl font-semibold">LumixHDR</h1>
        <p className="text-sm text-muted">
          Bracketing d'exposition et fusion HDR pour Panasonic GH6
        </p>
      </header>
      <main className="flex flex-1 flex-col gap-6 px-6 py-6">
        {loading ? (
          <div className="flex items-center gap-3">
            <Spinner />
            <span>Chargement…</span>
          </div>
        ) : (
          <Tabs
            selectedKey={screen}
            onSelectionChange={(key) => {
              if (typeof key === "string") {
                setScreen(key as Screen);
              }
            }}
            className="w-full"
          >
            <Tabs.ListContainer>
              <Tabs.List aria-label="Navigation principale">
                {SCREENS.map((item) => (
                  <Tabs.Tab key={item.id} id={item.id}>
                    {item.label}
                    <Tabs.Indicator />
                  </Tabs.Tab>
                ))}
              </Tabs.List>
            </Tabs.ListContainer>
            <Tabs.Panel id="connexion">
              <ConnectionScreen />
            </Tabs.Panel>
            <Tabs.Panel id="prise-de-vue">
              <PlaceholderScreen
                title="Prise de vue"
                description="La live view, les réglages rapides et le déclenchement de série arriveront à la phase 3."
              />
            </Tabs.Panel>
            <Tabs.Panel id="resultat">
              <PlaceholderScreen
                title="Résultat"
                description="La comparaison avant/après et la fusion HDR arriveront à la phase 2."
              />
            </Tabs.Panel>
            <Tabs.Panel id="historique">
              <PlaceholderScreen
                title="Historique"
                description="Les séquences précédentes seront listées à partir des manifestes à la phase 6."
              />
            </Tabs.Panel>
            <Tabs.Panel id="reglages">
              <SettingsScreen />
            </Tabs.Panel>
          </Tabs>
        )}
      </main>
    </div>
  );
}
