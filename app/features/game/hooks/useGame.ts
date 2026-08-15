import { useEffect, useRef, useSyncExternalStore } from "react";
import { gameSession } from "../application/game-session";
import { installBrowserGameApi } from "../adapters/browser-game-api";
import type { Direction } from "../domain/game";

const BEST_SCORE_KEY = "twenty48-best-score";

export function useGame() {
  const hasLoadedBestScore = useRef(false);
  const game = useSyncExternalStore(
    gameSession.subscribe,
    gameSession.getSnapshot,
    gameSession.getSnapshot,
  );

  useEffect(() => installBrowserGameApi(gameSession), []);

  useEffect(() => {
    if (!hasLoadedBestScore.current) {
      const saved = Number.parseInt(window.localStorage.getItem(BEST_SCORE_KEY) ?? "0", 10) || 0;
      hasLoadedBestScore.current = true;
      gameSession.loadBestScore(saved);
      return;
    }
    window.localStorage.setItem(BEST_SCORE_KEY, String(game.bestScore));
  }, [game.bestScore]);

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
      gameSession.move(direction);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return {
    ...game,
    move: gameSession.move,
    restart: gameSession.restart,
    continueGame: gameSession.continueGame,
  };
}
