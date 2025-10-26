import { useEffect, useRef, useCallback } from "react";

type Params = {
  onSingle?: (e: React.MouseEvent) => void;
  onDouble?: (e: React.MouseEvent) => void;
  delay?: number;
  tolerance?: number;
};

export default function usePaneClickCombo({
  onSingle,
  onDouble,
  delay = 250,
  tolerance = 6,
}: Params) {
  const timeoutRef = useRef<number | null>(null);
  const lastClickPosRef = useRef<{ x: number; y: number } | null>(null);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        window.clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    };
  }, []);

  const handlePaneClick = useCallback(
    (e: React.MouseEvent) => {
      const nowPos = { x: e.clientX, y: e.clientY };

      if (timeoutRef.current) {
        const prev = lastClickPosRef.current;
        const movedTooMuch =
          prev &&
          Math.hypot(nowPos.x - prev.x, nowPos.y - prev.y) > tolerance;

        if (!movedTooMuch) {
          window.clearTimeout(timeoutRef.current);
          timeoutRef.current = null;
          lastClickPosRef.current = null;
          onDouble?.(e);
          return;
        }

        window.clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }

      lastClickPosRef.current = nowPos;
      timeoutRef.current = window.setTimeout(() => {
        timeoutRef.current = null;
        lastClickPosRef.current = null;
        onSingle?.(e);
      }, delay);
    },
    [onSingle, onDouble, delay, tolerance]
  );

  return handlePaneClick;
}
