import {
  useCallback,
  useId,
  useRef,
  useState,
  type PointerEvent,
} from "react";

import { useT } from "../i18n/react";

type Props = {
  beforeSrc: string;
  afterSrc: string;
  beforeAlt?: string;
  afterAlt?: string;
};

/** Comparaison glissable avant / après (clip-path). */
export function BeforeAfterCompare({
  beforeSrc,
  afterSrc,
  beforeAlt,
  afterAlt,
}: Props) {
  const t = useT();
  const labelId = useId();
  const trackRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState(50);
  const dragging = useRef(false);

  const setFromClientX = useCallback((clientX: number) => {
    const el = trackRef.current;
    if (!el) {
      return;
    }
    const rect = el.getBoundingClientRect();
    if (rect.width <= 0) {
      return;
    }
    const next = ((clientX - rect.left) / rect.width) * 100;
    setPosition(Math.min(100, Math.max(0, next)));
  }, []);

  function onPointerDown(event: PointerEvent<HTMLDivElement>) {
    dragging.current = true;
    event.currentTarget.setPointerCapture(event.pointerId);
    setFromClientX(event.clientX);
  }

  function onPointerMove(event: PointerEvent<HTMLDivElement>) {
    if (!dragging.current) {
      return;
    }
    setFromClientX(event.clientX);
  }

  function onPointerUp(event: PointerEvent<HTMLDivElement>) {
    dragging.current = false;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }

  return (
    <div className="flex flex-col gap-2">
      <div
        ref={trackRef}
        role="slider"
        tabIndex={0}
        aria-labelledby={labelId}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(position)}
        className="relative aspect-[4/3] w-full cursor-ew-resize select-none overflow-hidden rounded-xl bg-black/40 touch-none"
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        onKeyDown={(event) => {
          if (event.key === "ArrowLeft") {
            event.preventDefault();
            setPosition((p) => Math.max(0, p - 2));
          } else if (event.key === "ArrowRight") {
            event.preventDefault();
            setPosition((p) => Math.min(100, p + 2));
          }
        }}
      >
        <img
          src={afterSrc}
          alt={afterAlt ?? t("history.afterAlt")}
          className="absolute inset-0 h-full w-full object-contain"
          draggable={false}
        />
        <div
          className="absolute inset-0 overflow-hidden"
          style={{ clipPath: `inset(0 ${100 - position}% 0 0)` }}
        >
          <img
            src={beforeSrc}
            alt={beforeAlt ?? t("history.beforeAlt")}
            className="absolute inset-0 h-full w-full object-contain"
            draggable={false}
          />
        </div>
        <div
          className="pointer-events-none absolute inset-y-0 w-0.5 bg-white shadow-[0_0_0_1px_rgba(0,0,0,0.35)]"
          style={{ left: `${position}%` }}
        >
          <div className="absolute top-1/2 left-1/2 flex h-9 w-9 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full border border-white/80 bg-black/55 text-xs font-medium text-white">
            ↔
          </div>
        </div>
        <span className="pointer-events-none absolute top-3 left-3 rounded-md bg-black/55 px-2 py-0.5 text-xs text-white">
          {t("history.before")}
        </span>
        <span className="pointer-events-none absolute top-3 right-3 rounded-md bg-black/55 px-2 py-0.5 text-xs text-white">
          {t("history.after")}
        </span>
      </div>
      <p id={labelId} className="text-sm text-muted">
        {t("history.compareHint")}
      </p>
    </div>
  );
}
