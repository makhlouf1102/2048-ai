import { useCallback, useEffect, useRef, useState } from "react";
import { gameSession } from "../application/game-session";
import { installBrowserGameApi } from "../adapters/browser-game-api";
import type { Direction } from "../domain/game";

const BEST_SCORE_KEY = "twenty48-best-score";

export function useGame(liveUpdates = true) {
  const hasLoadedBestScore = useRef(false);
  const [game, setGame] = useState(gameSession.getSnapshot);

  const refresh = useCallback(() => {
    setGame(gameSession.getSnapshot());
  }, []);

  useEffect(() => {
    if (!liveUpdates) return;
    refresh();
    return gameSession.subscribe(refresh);
  }, [liveUpdates, refresh]);

  useEffect(() => installBrowserGameApi(gameSession), []);

  useEffect(() => {
    const saved = Number.parseInt(window.localStorage.getItem(BEST_SCORE_KEY) ?? "0", 10) || 0;
    hasLoadedBestScore.current = true;
    gameSession.loadBestScore(saved);
    refresh();

    return gameSession.subscribe(() => {
      if (!hasLoadedBestScore.current) return;
      window.localStorage.setItem(BEST_SCORE_KEY, String(gameSession.getSnapshot().bestScore));
    });
  }, [refresh]);

  useEffect(() => {
    const directions: Record<string, Direction> = {
      ArrowUp: "up",
      ArrowDown: "down",
      ArrowLeft: "left",
      ArrowRight: "right",
      w: "up",
      s: "down",
      a: "left",
      d: "right",
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      const direction = directions[event.key];
      if (!direction) return;
      event.preventDefault();
      setGame(gameSession.move(direction));
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return {
    ...game,
    move: (direction: Direction) => {
      const next = gameSession.move(direction);
      setGame(next);
      return next;
    },
    restart: () => {
      const next = gameSession.restart();
      setGame(next);
      return next;
    },
    continueGame: () => {
      const next = gameSession.continueGame();
      setGame(next);
      return next;
    },
    refresh,
  };
}
