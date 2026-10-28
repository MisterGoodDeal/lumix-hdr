import {
  Button,
  Card,
  Description,
  Label,
  ListBox,
  Select,
  Switch,
  Tooltip,
} from "@heroui/react";

import { ErrorDetails } from "../components/ErrorDetails";
import { isLocalePreference } from "../i18n";
import { useT } from "../i18n/react";
import { fusionMethodLabel } from "../lib/fusion";
import { isTauri, openPath, pickSessionsFolder } from "../lib/tauri";
import { useAppStore } from "../store/app-store";
import { isThemePreference } from "../theme";
import type {
  FusionMethod,
  LocalePreference,
  LogLevel,
  ThemePreference,
} from "../types";

const LOG_LEVELS: LogLevel[] = ["error", "warn", "info", "debug", "trace"];
const LOCALES: LocalePreference[] = ["system", "en", "fr"];
const THEMES: ThemePreference[] = ["system", "dark", "light"];

export function SettingsScreen() {
  const t = useT();
  const status = useAppStore((state) => state.status);
  const error = useAppStore((state) => state.error);
  const setDemoMode = useAppStore((state) => state.setDemoMode);
  const setLogLevel = useAppStore((state) => state.setLogLevel);
  const setFusionOptions = useAppStore((state) => state.setFusionOptions);
  const setLocale = useAppStore((state) => state.setLocale);
  const setTheme = useAppStore((state) => state.setTheme);
  const setSessionsDir = useAppStore((state) => state.setSessionsDir);

  if (!status) {
    return (
      <Card className="max-w-xl">
        <Card.Header>
          <Card.Title>{t("settings.title")}</Card.Title>
          <Card.Description>{t("common.loading")}</Card.Description>
        </Card.Header>
      </Card>
    );
  }

  function logLabel(level: LogLevel): string {
    switch (level) {
      case "error":
        return t("settings.log.error");
      case "warn":
        return t("settings.log.warn");
      case "info":
        return t("settings.log.info");
      case "debug":
        return t("settings.log.debug");
      case "trace":
        return t("settings.log.trace");
    }
  }

  function localeLabel(locale: LocalePreference): string {
    switch (locale) {
      case "system":
        return t("locale.system");
      case "en":
        return t("locale.en");
      case "fr":
        return t("locale.fr");
    }
  }

  function themeLabel(theme: ThemePreference): string {
    switch (theme) {
      case "system":
        return t("theme.system");
      case "dark":
        return t("theme.dark");
      case "light":
        return t("theme.light");
    }
  }

  return (
    <Card className="max-w-xl">
      <Card.Header>
        <Card.Title>{t("settings.title")}</Card.Title>
        <Card.Description>
          {t("settings.version", { version: status.version })}
        </Card.Description>
      </Card.Header>
      <Card.Content className="flex flex-col gap-6">
        <Select
          selectedKey={status.locale}
          onSelectionChange={(key) => {
            if (typeof key === "string" && isLocalePreference(key)) {
              void setLocale(key);
            }
          }}
        >
          <Label>{t("locale.label")}</Label>
          <Select.Trigger>
            <Select.Value />
            <Select.Indicator />
          </Select.Trigger>
          <Select.Popover>
            <ListBox>
              {LOCALES.map((locale) => (
                <ListBox.Item
                  id={locale}
                  key={locale}
                  textValue={localeLabel(locale)}
                >
                  {localeLabel(locale)}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              ))}
            </ListBox>
          </Select.Popover>
        </Select>
        <Description>{t("locale.hint")}</Description>

        <Select
          selectedKey={status.theme}
          onSelectionChange={(key) => {
            if (typeof key === "string" && isThemePreference(key)) {
              void setTheme(key);
            }
          }}
        >
          <Label>{t("theme.label")}</Label>
          <Select.Trigger>
            <Select.Value />
            <Select.Indicator />
          </Select.Trigger>
          <Select.Popover>
            <ListBox>
              {THEMES.map((theme) => (
                <ListBox.Item
                  id={theme}
                  key={theme}
                  textValue={themeLabel(theme)}
                >
                  {themeLabel(theme)}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              ))}
            </ListBox>
          </Select.Popover>
        </Select>
        <Description>{t("theme.hint")}</Description>

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
              {t("settings.demo")}
            </Switch.Content>
          </Switch>
          {status.demo_forced_by_env ? (
            <Description>{t("settings.demoForced")}</Description>
          ) : (
            <Description>{t("settings.demoHint")}</Description>
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
          <Label>{t("settings.logLevel")}</Label>
          <Select.Trigger>
            <Select.Value />
            <Select.Indicator />
          </Select.Trigger>
          <Select.Popover>
            <ListBox>
              {LOG_LEVELS.map((level) => (
                <ListBox.Item
                  id={level}
                  key={level}
                  textValue={logLabel(level)}
                >
                  {logLabel(level)}
                  <ListBox.ItemIndicator />
                </ListBox.Item>
              ))}
            </ListBox>
          </Select.Popover>
        </Select>

        <Switch
          isSelected={status.fusion.align}
          onChange={(align) => {
            void setFusionOptions({ ...status.fusion, align });
          }}
        >
          <Switch.Content>
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            {t("settings.align")}
          </Switch.Content>
        </Switch>
        <Description>{t("settings.alignHint")}</Description>

        <Select
          selectedKey={status.fusion.method}
          onSelectionChange={(key) => {
            if (key === "mertens" || key === "debevec") {
              void setFusionOptions({
                ...status.fusion,
                method: key as FusionMethod,
              });
            }
          }}
        >
          <Label>{t("settings.fusionMethod")}</Label>
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

        <Switch
          isSelected={status.fusion.output_jpeg}
          onChange={(output_jpeg) => {
            void setFusionOptions({ ...status.fusion, output_jpeg });
          }}
        >
          <Switch.Content>
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            {t("settings.exportJpeg")}
          </Switch.Content>
        </Switch>

        <Switch
          isSelected={status.fusion.output_tiff}
          onChange={(output_tiff) => {
            void setFusionOptions({ ...status.fusion, output_tiff });
          }}
        >
          <Switch.Content>
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            {t("settings.exportTiff")}
          </Switch.Content>
        </Switch>

        <div className="flex flex-col gap-2">
          <Label>{t("settings.sessionsDir")}</Label>
          <Tooltip>
            <Tooltip.Trigger>
              <code className="break-all text-sm text-muted">
                {status.sessions_dir}
              </code>
            </Tooltip.Trigger>
            <Tooltip.Content>{status.sessions_dir}</Tooltip.Content>
          </Tooltip>
          <Description>
            {status.sessions_dir_custom
              ? t("settings.sessionsCustom")
              : t("settings.sessionsDefault")}
          </Description>
          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant="secondary"
              isDisabled={!isTauri()}
              onPress={() => {
                void (async () => {
                  const folder = await pickSessionsFolder();
                  if (folder) {
                    await setSessionsDir(folder);
                  }
                })();
              }}
            >
              {t("common.change")}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              isDisabled={!isTauri() || !status.sessions_dir_custom}
              onPress={() => {
                void setSessionsDir(null);
              }}
            >
              {t("common.reset")}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              isDisabled={!isTauri()}
              onPress={() => {
                void openPath(status.sessions_dir);
              }}
            >
              {t("common.open")}
            </Button>
          </div>
        </div>

        <div className="flex flex-col gap-1">
          <Label>{t("settings.logDir")}</Label>
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
