import { useCallback, useState } from "react";

export type AiViewMode = "quiet" | "live";

/** Owns how an AI run is presented; it never reads or mutates game state. */
export function useAiPresentation() {
  const [mode, setMode] = useState<AiViewMode>("quiet");
  const [snapshotVisible, setSnapshotVisible] = useState(false);

  const beginRun = useCallback(() => setSnapshotVisible(false), []);
  const revealSnapshot = useCallback(() => setSnapshotVisible(true), []);
  const watchLive = useCallback(() => {
    setMode("live");
    setSnapshotVisible(false);
  }, []);
  const enterQuietMode = useCallback(() => {
    setMode("quiet");
    setSnapshotVisible(false);
  }, []);

  return {
    beginRun,
    enterQuietMode,
    isLive: mode === "live",
    mode,
    revealSnapshot,
    snapshotVisible,
    watchLive,
  };
}
