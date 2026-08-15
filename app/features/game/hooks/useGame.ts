import { useCallback, useEffect, useRef, useState } from "react";
import {
  addRandomTile,
  canMove,
  createInitialBoard,
  hasWinningTile,
  moveBoard,
  type Board,
  type Direction,
} from "../domain/game";

const BEST_SCORE_KEY = "twenty48-best-score";

interface GameState {
  board: Board;
  score: number;
  bestScore: number;
  status: "playing" | "won" | "lost";
  keepPlaying: boolean;
}

function newGameState(bestScore = 0, board = createInitialBoard()): GameState {
  return {
    board,
    score: 0,
    bestScore,
    status: "playing",
    keepPlaying: false,
  };
}

export function useGame() {
  const hasLoadedBestScore = useRef(false);
  // A stable first board keeps SSR output identical to the hydration render.
  const [game, setGame] = useState<GameState>(() => {
    const sequence = [0.08, 0.1, 0.72, 0.1];
    let index = 0;
    return newGameState(0, createInitialBoard(() => sequence[index++]));
  });

  const move = useCallback((direction: Direction) => {
    setGame((current) => {
      if (current.status === "lost" || (current.status === "won" && !current.keepPlaying)) {
        return current;
      }

      const result = moveBoard(current.board, direction);
      if (!result.moved) return current;

      const board = addRandomTile(result.board);
      const score = current.score + result.scoreGain;
      const bestScore = Math.max(current.bestScore, score);
      const status = !canMove(board)
        ? "lost"
        : hasWinningTile(board) && !current.keepPlaying
          ? "won"
          : "playing";

      return { ...current, board, score, bestScore, status };
    });
  }, []);

  const restart = useCallback(
    () => setGame((current) => newGameState(current.bestScore)),
    [],
  );
  const continueGame = useCallback(
    () => setGame((current) => ({ ...current, status: "playing", keepPlaying: true })),
    [],
  );

  useEffect(() => {
    if (!hasLoadedBestScore.current) {
      const saved = Number.parseInt(window.localStorage.getItem(BEST_SCORE_KEY) ?? "0", 10) || 0;
      hasLoadedBestScore.current = true;
      setGame((current) => ({ ...current, bestScore: Math.max(current.bestScore, saved) }));
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
      move(direction);
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [move]);

  return { ...game, move, restart, continueGame };
}
