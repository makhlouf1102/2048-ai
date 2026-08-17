import { useCallback, useEffect, useRef, useState } from "react";
import { cancelAiWork, nextMove, type AiStrategy } from "../../ai";
import { moveDelay, QUIET_TELEMETRY_INTERVAL_MS } from "../application/ai-playback-policy";
import { gameSession } from "../application/game-session";
import type { Direction } from "../domain/game";

const directions: Record<number, Direction> = {
  0: "down",
  1: "left",
  2: "right",
  3: "up",
};

export type AiPlayerStatus = "idle" | "running" | "error";

export interface AiMetrics {
  averageDecisionMs: number;
  decisionHistory: number[];
  lastDecisionMs: number;
  lastDirection: Direction | null;
  lastScoreGain: number;
  moves: number;
  utilizationHistory: number[];
  wasmUtilization: number;
}

const emptyMetrics: AiMetrics = {
  averageDecisionMs: 0,
  decisionHistory: [],
  lastDecisionMs: 0,
  lastDirection: null,
  lastScoreGain: 0,
  moves: 0,
  utilizationHistory: [],
  wasmUtilization: 0,
};

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}

export function useAiPlayer({ liveUpdates = true }: { liveUpdates?: boolean } = {}) {
  const [status, setStatus] = useState<AiPlayerStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<AiMetrics>(emptyMetrics);
  const runId = useRef(0);
  const runStartedAt = useRef(0);
  const totalDecisionMs = useRef(0);
  const decisionCount = useRef(0);
  const moveCount = useRef(0);
  const metricsRef = useRef(emptyMetrics);
  const liveUpdatesRef = useRef(liveUpdates);

  useEffect(() => {
    liveUpdatesRef.current = liveUpdates;
    if (liveUpdates) setMetrics(metricsRef.current);
  }, [liveUpdates]);

  useEffect(() => {
    if (liveUpdates || status !== "running") return;
    const interval = window.setInterval(
      () => setMetrics(metricsRef.current),
      QUIET_TELEMETRY_INTERVAL_MS,
    );
    return () => window.clearInterval(interval);
  }, [liveUpdates, status]);

  const stop = useCallback(() => {
    runId.current += 1;
    cancelAiWork();
    setStatus("idle");
  }, []);

  const start = useCallback(async (strategy: AiStrategy) => {
    const currentRun = ++runId.current;
    runStartedAt.current = performance.now();
    totalDecisionMs.current = 0;
    decisionCount.current = 0;
    moveCount.current = 0;
    metricsRef.current = emptyMetrics;
    setMetrics(emptyMetrics);
    setError(null);
    setStatus("running");

    try {
      if (gameSession.getSnapshot().status === "lost") gameSession.restart();

      while (currentRun === runId.current) {
        const beforeMove = gameSession.getSnapshot();
        if (beforeMove.status === "lost") break;
        if (beforeMove.status === "won") {
          gameSession.continueGame();
          continue;
        }

        // Wasm receives a copy of the 4×4 board as a flat, row-major array.
        const decisionStartedAt = performance.now();
        const directionCode = await nextMove(beforeMove.board.flat(), strategy);
        const decisionDuration = performance.now() - decisionStartedAt;
        totalDecisionMs.current += decisionDuration;
        decisionCount.current += 1;
        if (currentRun !== runId.current) return;

        const updateMetrics = (lastDirection: Direction | null, lastScoreGain = 0) => {
          const elapsed = Math.max(performance.now() - runStartedAt.current, 1);
          const wasmUtilization = Math.min(100, (totalDecisionMs.current / elapsed) * 100);
          const previous = metricsRef.current;
          const nextMetrics = {
            averageDecisionMs: totalDecisionMs.current / decisionCount.current,
            decisionHistory: [...previous.decisionHistory, decisionDuration].slice(-18),
            lastDecisionMs: decisionDuration,
            lastDirection,
            lastScoreGain,
            moves: moveCount.current,
            utilizationHistory: [...previous.utilizationHistory, wasmUtilization].slice(-18),
            wasmUtilization,
          };
          metricsRef.current = nextMetrics;
          if (liveUpdatesRef.current) setMetrics(nextMetrics);
        };

        if (directionCode === -1) {
          updateMetrics(null);
          break;
        }

        const direction = directions[directionCode];
        if (!direction) {
          throw new Error(`nextMove returned ${directionCode}; expected -1, 0, 1, 2, or 3.`);
        }

        const feedback = gameSession.move(direction);
        if (!feedback.moved && feedback.status === "playing") {
          throw new Error(`nextMove selected a blocked direction (${directionCode}).`);
        }

        if (feedback.moved) moveCount.current += 1;
        updateMetrics(
          feedback.moved ? direction : null,
          feedback.moved ? feedback.score - beforeMove.score : 0,
        );

        await delay(moveDelay(liveUpdatesRef.current));
      }

      if (currentRun === runId.current) setStatus("idle");
    } catch (cause) {
      if (currentRun !== runId.current) return;
      setError(cause instanceof Error ? cause.message : "The AI player failed.");
      setStatus("error");
    }
  }, []);

  useEffect(() => () => {
    runId.current += 1;
    cancelAiWork();
  }, []);

  const refresh = useCallback(() => setMetrics(metricsRef.current), []);

  return { status, error, metrics, start, stop, refresh };
}
