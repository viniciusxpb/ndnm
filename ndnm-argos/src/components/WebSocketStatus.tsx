import React from "react";

type Props = {
  status: "idle" | "connecting" | "open" | "closing" | "closed";
};

export function WebSocketStatus({ status }: Props) {
  const color =
    status === "open"
      ? "#00ff95"
      : status === "connecting"
      ? "#ffee00"
      : status === "closing"
      ? "#ff9f00"
      : status === "closed"
      ? "#ff4d4d"
      : "#7fdbff";

  return (
    <div
      style={{
        position: "fixed",
        top: 8,
        right: 8,
        zIndex: 9999,
        background: "#0b0f0c",
        border: `1px solid ${color}`,
        color,
        fontFamily:
          "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
        fontSize: 12,
        padding: "6px 10px",
        borderRadius: 10,
        boxShadow: "0 0 10px rgba(0,255,149,0.15)",
        userSelect: "none",
      }}
      aria-live="polite"
    >
      WS: {status.toUpperCase()}
    </div>
  );
}
