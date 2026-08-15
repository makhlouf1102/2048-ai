import { useCallback, useEffect, useRef, useState } from "react";
import { nextMove } from "../../ai";
import { gameSession } from "../application/game-session";
import type { Direction } from "../domain/game";

const MOVE_DELAY_MS = 110;
const directions: Record<number, Direction> = {
  0: "down",
  1: "left",
  2: "right",
  3: "up",
};

export type AiPlayerStatus = "idle" | "running" | "error";

export interface AiMetrics {
  averageDecisionMs: number;
  lastDecisionMs: number;
  lastDirection: Direction | null;
  moves: number;
  wasmUtilization: number;
}

const emptyMetrics: AiMetrics = {
  averageDecisionMs: 0,
  lastDecisionMs: 0,
  lastDirection: null,
  moves: 0,
  wasmUtilization: 0,
};

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}

export function useAiPlayer() {
  const [status, setStatus] = useState<AiPlayerStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<AiMetrics>(emptyMetrics);
  const runId = useRef(0);
  const runStartedAt = useRef(0);
  const totalDecisionMs = useRef(0);
  const decisionCount = useRef(0);
  const moveCount = useRef(0);

  const stop = useCallback(() => {
    runId.current += 1;
    setStatus("idle");
  }, []);

  const start = useCallback(async () => {
    const currentRun = ++runId.current;
    runStartedAt.current = performance.now();
    totalDecisionMs.current = 0;
    decisionCount.current = 0;
    moveCount.current = 0;
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
        const directionCode = await nextMove(beforeMove.board.flat());
        const decisionDuration = performance.now() - decisionStartedAt;
        totalDecisionMs.current += decisionDuration;
        decisionCount.current += 1;
        if (currentRun !== runId.current) return;

        const updateMetrics = (lastDirection: Direction | null) => {
          const elapsed = Math.max(performance.now() - runStartedAt.current, 1);
          setMetrics({
            averageDecisionMs: totalDecisionMs.current / decisionCount.current,
            lastDecisionMs: decisionDuration,
            lastDirection,
            moves: moveCount.current,
            wasmUtilization: Math.min(100, (totalDecisionMs.current / elapsed) * 100),
          });
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
        updateMetrics(feedback.moved ? direction : null);

        await delay(MOVE_DELAY_MS);
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
  }, []);

  return { status, error, metrics, start, stop };
}
