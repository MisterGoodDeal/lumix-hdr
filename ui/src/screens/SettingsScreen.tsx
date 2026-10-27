import {
  Card,
  Description,
  Label,
  ListBox,
  Select,
  Switch,
  Tooltip,
} from "@heroui/react";

import { ErrorDetails } from "../components/ErrorDetails";
import { useAppStore } from "../store/app-store";
import type { LogLevel } from "../types";

const LOG_LEVELS: { id: LogLevel; label: string }[] = [
  { id: "error", label: "Erreur" },
  { id: "warn", label: "Avertissement" },
  { id: "info", label: "Info" },
  { id: "debug", label: "Debug" },
  { id: "trace", label: "Trace" },
];

export function SettingsScreen() {
  const status = useAppStore((state) => state.status);
  const error = useAppStore((state) => state.error);
  const setDemoMode = useAppStore((state) => state.setDemoMode);
  const setLogLevel = useAppStore((state) => state.setLogLevel);

  if (!status) {
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>Réglages</Card.Title>
          <Card.Description>Chargement…</Card.Description>
        </Card.Header>
      </Card>
    );
  }

  return (
    <Card className="max-w-xl">
      <Card.Header>
        <Card.Title>Réglages</Card.Title>
        <Card.Description>
          Configuration persistée localement, version {status.version}.
        </Card.Description>
      </Card.Header>
      <Card.Content className="flex flex-col gap-6">
        <div className="flex flex-col gap-2">
          <Switch
            isSelected={status.demo_mode}
            isDisabled={status.demo_forced_by_env}
            onChange={(enabled) => {
              void setDemoMode(enabled);
            }}
          >
            <Switch.Content>
              <Switch.Control>
                <Switch.Thumb />
              </Switch.Control>
              Mode démo
            </Switch.Content>
          </Switch>
          {status.demo_forced_by_env ? (
            <Description>
              Imposé par la variable d'environnement LUMIX_HDR_DEMO=1.
            </Description>
          ) : (
            <Description>
              Active une caméra fictive pour tester l'application sans GH6.
            </Description>
          )}
        </div>

        <Select
          selectedKey={status.log_level}
          onSelectionChange={(key) => {
            if (typeof key === "string") {
              void setLogLevel(key as LogLevel);
            }
          }}
        >
          <Label>Niveau de journaux</Label>
          <Select.Trigger>
            <Select.Value />
            <Select.Indicator />
          </Select.Trigger>
          <Select.Popover>
            <ListBox>
              {LOG_LEVELS.map((level) => (
                <ListBox.Item id={level.id} key={level.id} textValue={level.label}>
                  {level.label}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              ))}
            </ListBox>
          </Select.Popover>
        </Select>

        <div className="flex flex-col gap-1">
          <Label>Dossier des journaux</Label>
          <Tooltip>
            <Tooltip.Trigger>
              <code className="break-all text-sm text-muted">{status.log_dir}</code>
            </Tooltip.Trigger>
            <Tooltip.Content>{status.log_dir}</Tooltip.Content>
          </Tooltip>
        </div>

        <ErrorDetails error={error} />
      </Card.Content>
    </Card>
  );
}
