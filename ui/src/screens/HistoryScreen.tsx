import { ArrowLeft, FolderOpen, Images, Trash } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import {
  Button,
  Card,
  Chip,
  Spinner,
  toast,
} from "@heroui/react";

import { BeforeAfterCompare } from "../components/BeforeAfterCompare";
import { ErrorDetails } from "../components/ErrorDetails";
import { useLocale, useT } from "../i18n/react";
import {
  formatEv,
  formatSessionDate,
  sessionCardTitle,
  sessionStateColor,
  sessionStateLabel,
} from "../lib/history";
import {
  confirmDeleteSession,
  deleteSession,
  getSession,
  listSessions,
  openPath,
  parseAppError,
  previewSrc,
} from "../lib/tauri";
import type { AppError, SessionDetail, SessionSummary } from "../types";

export function HistoryScreen() {
  const t = useT();
  const locale = useLocale();
  const [loading, setLoading] = useState(true);
  const [summaries, setSummaries] = useState<SessionSummary[]>([]);
  const [listError, setListError] = useState<AppError | null>(null);
  const [selectedDir, setSelectedDir] = useState<string | null>(null);
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<AppError | null>(null);

  useEffect(() => {
    let cancelled = false;
    void listSessions()
      .then((next) => {
        if (cancelled) {
          return;
        }
        setSummaries(next);
        setListError(null);
      })
      .catch((cause) => {
        if (cancelled) {
          return;
        }
        const parsed = parseAppError(cause);
        setListError(parsed);
        setSummaries([]);
        toast.danger(t("history.loadFail"), { description: parsed.message });
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [t]);

  async function refresh() {
    setLoading(true);
    setListError(null);
    try {
      const next = await listSessions();
      setSummaries(next);
    } catch (cause) {
      const parsed = parseAppError(cause);
      setListError(parsed);
      setSummaries([]);
      toast.danger(t("history.loadFail"), { description: parsed.message });
    } finally {
      setLoading(false);
    }
  }

  async function openDetail(sessionDir: string) {
    setSelectedDir(sessionDir);
    setDetail(null);
    setDetailError(null);
    setDetailLoading(true);
    try {
      const next = await getSession(sessionDir);
      setDetail(next);
    } catch (cause) {
      const parsed = parseAppError(cause);
      setDetailError(parsed);
      toast.danger(t("history.detailFail"), { description: parsed.message });
    } finally {
      setDetailLoading(false);
    }
  }

  function closeDetail() {
    setSelectedDir(null);
    setDetail(null);
    setDetailError(null);
  }

  async function removeSession(sessionDir: string, sessionName: string) {
    const ok = await confirmDeleteSession(sessionName);
    if (!ok) {
      return;
    }
    try {
      await deleteSession(sessionDir);
      setSummaries((prev) =>
        prev.filter((item) => item.session_dir !== sessionDir),
      );
      if (selectedDir === sessionDir) {
        closeDetail();
      }
      toast.success(t("history.deleted"));
    } catch (cause) {
      const parsed = parseAppError(cause);
      toast.danger(t("history.deleteFail"), { description: parsed.message });
    }
  }

  if (selectedDir) {
    return (
      <div className="mx-auto flex w-full max-w-5xl flex-col gap-4">
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="secondary" size="sm" onPress={closeDetail}>
            <ArrowLeft size={16} aria-hidden />
            {t("history.back")}
          </Button>
          {detail ? (
            <>
              <Button
                variant="ghost"
                size="sm"
                onPress={() => void openPath(detail.session_dir)}
              >
                <FolderOpen size={16} aria-hidden />
                {t("history.openFolder")}
              </Button>
              <Button
                variant="danger"
                size="sm"
                onPress={() =>
                  void removeSession(detail.session_dir, detail.session_name)
                }
              >
                <Trash size={16} aria-hidden />
                {t("history.delete")}
              </Button>
            </>
          ) : null}
        </div>

        {detailLoading ? (
          <div className="flex items-center gap-3 py-12">
            <Spinner />
            <span>{t("common.loading")}</span>
          </div>
        ) : null}

        <ErrorDetails error={detailError} />

        {detail ? <SessionDetailView detail={detail} locale={locale} /> : null}
      </div>
    );
  }

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-4">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">
            {t("history.title")}
          </h1>
        </div>
        <Button
          variant="secondary"
          size="sm"
          isDisabled={loading}
          onPress={() => void refresh()}
        >
          {t("history.refresh")}
        </Button>
      </div>

      <ErrorDetails error={listError} />

      {loading ? (
        <div className="flex items-center gap-3 py-12">
          <Spinner />
          <span>{t("common.loading")}</span>
        </div>
      ) : null}

      {!loading && summaries.length === 0 ? (
        <Card>
          <Card.Header>
            <Card.Title>{t("history.title")}</Card.Title>
            <Card.Description>{t("history.empty")}</Card.Description>
          </Card.Header>
          <Card.Content>
            <Images size={40} className="text-muted" aria-hidden />
          </Card.Content>
        </Card>
      ) : null}

      {!loading && summaries.length > 0 ? (
        <div className="grid grid-cols-12 gap-4">
          {summaries.map((summary) => (
            <SessionCard
              key={summary.session_dir}
              summary={summary}
              locale={locale}
              onView={() => void openDetail(summary.session_dir)}
              onDelete={() =>
                void removeSession(
                  summary.session_dir,
                  sessionCardTitle(summary),
                )
              }
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function SessionCard({
  summary,
  locale,
  onView,
  onDelete,
}: {
  summary: SessionSummary;
  locale: string;
  onView: () => void;
  onDelete: () => void;
}) {
  const t = useT();
  const thumb = previewSrc(summary.thumb_jpeg);
  const color = sessionStateColor(summary.state_kind);
  const meta = [
    t("history.frames", { count: summary.frame_count }),
    t("history.step", { ev: formatEv(summary.step_ev).replace("+", "") }),
  ].join(" · ");

  return (
    <Card className="relative col-span-12 min-h-[220px] overflow-hidden rounded-3xl sm:col-span-6 xl:col-span-4">
      {thumb ? (
        <img
          src={thumb}
          alt=""
          aria-hidden
          className="absolute inset-0 h-full w-full object-cover"
        />
      ) : (
        <div
          aria-hidden
          className="absolute inset-0 bg-gradient-to-br from-foreground/15 to-foreground/5"
        />
      )}
      {/* Lisibilité du texte sur n’importe quelle photo */}
      <div
        aria-hidden
        className="absolute inset-0 bg-gradient-to-b from-black/55 via-black/15 to-black/65"
      />

      <Card.Header className="z-10 text-white">
        <div className="mb-2">
          <Chip size="sm" color={color} variant="primary">
            {sessionStateLabel(summary.state_kind)}
          </Chip>
        </div>
        <Card.Title className="truncate text-base font-semibold tracking-wide text-white">
          {sessionCardTitle(summary)}
        </Card.Title>
        <Card.Description className="text-sm leading-5 font-medium text-white/70">
          {formatSessionDate(summary.created_at, locale)}
        </Card.Description>
      </Card.Header>

      <Card.Footer className="z-10 mt-auto flex items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="truncate text-sm font-medium text-white">{meta}</div>
          <div className="truncate text-xs text-white/60">
            {summary.has_fused
              ? t("history.after")
              : t("history.frames", { count: summary.original_count })}
          </div>
        </div>
        <div className="flex shrink-0 gap-2">
          <Button
            aria-label={t("history.delete")}
            className="bg-white/15 text-white"
            size="sm"
            variant="tertiary"
            onPress={onDelete}
          >
            <Trash size={16} aria-hidden />
          </Button>
          <Button
            className="bg-white text-black"
            size="sm"
            variant="tertiary"
            onPress={onView}
          >
            {t("history.view")}
          </Button>
        </div>
      </Card.Footer>
    </Card>
  );
}

function SessionDetailView({
  detail,
  locale,
}: {
  detail: SessionDetail;
  locale: string;
}) {
  const t = useT();
  const before = previewSrc(detail.before_jpeg);
  const after = previewSrc(detail.fused_jpeg);
  const stateKind =
    detail.state.kind === "done" ||
    detail.state.kind === "aborted" ||
    detail.state.kind === "failed"
      ? detail.state.kind
      : "other";

  return (
    <div className="flex flex-col gap-4 lg:flex-row">
      <Card className="min-w-0 flex-1">
        <Card.Header>
          <Card.Title>{detail.session_name}</Card.Title>
          <Card.Description>
            {formatSessionDate(detail.created_at, locale)}
          </Card.Description>
        </Card.Header>
        <Card.Content>
          {before && after ? (
            <BeforeAfterCompare beforeSrc={before} afterSrc={after} />
          ) : after ? (
            <img
              src={after}
              alt={t("history.afterAlt")}
              className="w-full rounded-xl object-contain"
            />
          ) : (
            <p className="text-muted">{t("history.noCompare")}</p>
          )}
        </Card.Content>
      </Card>

      <Card className="w-full shrink-0 lg:w-80">
        <Card.Header>
          <Card.Title>{t("history.details")}</Card.Title>
        </Card.Header>
        <Card.Content className="flex flex-col gap-3 text-sm">
          <Chip
            size="sm"
            color={sessionStateColor(stateKind)}
            variant="primary"
          >
            {sessionStateLabel(stateKind)}
          </Chip>
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-2">
            <dt className="text-muted">{t("history.created")}</dt>
            <dd>{formatSessionDate(detail.created_at, locale)}</dd>
            <dt className="text-muted">{t("history.bracket")}</dt>
            <dd>
              {t("history.frames", { count: detail.frame_count })} ·{" "}
              {t("history.step", {
                ev: formatEv(detail.step_ev).replace("+", ""),
              })}
            </dd>
            <dt className="text-muted">{t("studio.centerEv")}</dt>
            <dd>{formatEv(detail.center_ev)} EV</dd>
            <dt className="text-muted">{t("studio.iso")}</dt>
            <dd>{detail.base_iso}</dd>
            <dt className="text-muted">{t("studio.aperture")}</dt>
            <dd>f/{detail.base_aperture}</dd>
            <dt className="text-muted">{t("studio.shutter")}</dt>
            <dd>{detail.base_shutter_label}</dd>
            <dt className="text-muted">{t("history.appVersion")}</dt>
            <dd>{detail.app_version}</dd>
          </dl>

          {detail.originals.length > 0 ? (
            <div className="flex flex-col gap-2">
              <p className="font-medium">{t("history.bracket")}</p>
              <ul className="flex flex-col gap-1 text-muted">
                {detail.originals.map((entry) => (
                  <li key={entry.name} className="flex justify-between gap-2">
                    <span>{entry.shutter_label}</span>
                    <span>{formatEv(entry.relative_ev)} EV</span>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}

          {detail.errors.length > 0 ? (
            <div className="flex flex-col gap-1">
              <p className="font-medium">{t("history.errors")}</p>
              <ul className="list-disc pl-4 text-muted">
                {detail.errors.map((error) => (
                  <li key={error}>{error}</li>
                ))}
              </ul>
            </div>
          ) : null}
        </Card.Content>
      </Card>
    </div>
  );
}
