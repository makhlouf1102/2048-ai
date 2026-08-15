import {
  addRandomTile,
  canMove,
  createInitialBoard,
  hasWinningTile,
  moveBoard,
  type Board,
  type Direction,
} from "../domain/game";

export type GameStatus = "playing" | "won" | "lost";

export interface GameSnapshot {
  board: Board;
  score: number;
  bestScore: number;
  status: GameStatus;
  keepPlaying: boolean;
  moved: boolean;
}

type Listener = () => void;

function createHydrationSafeBoard(): Board {
  const sequence = [0.08, 0.1, 0.72, 0.1];
  let index = 0;
  return createInitialBoard(() => sequence[index++]);
}

function initialSnapshot(board = createHydrationSafeBoard(), bestScore = 0): GameSnapshot {
  return {
    board,
    score: 0,
    bestScore,
    status: "playing",
    keepPlaying: false,
    moved: false,
  };
}

/** Owns one game session. UI and external players issue commands through this class. */
export class GameSession {
  private snapshot = initialSnapshot();
  private readonly listeners = new Set<Listener>();

  getSnapshot = (): GameSnapshot => this.snapshot;

  subscribe = (listener: Listener): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  move = (direction: Direction): GameSnapshot => {
    const current = this.snapshot;
    if (current.status === "lost" || (current.status === "won" && !current.keepPlaying)) {
      this.commit({ ...current, moved: false });
      return this.snapshot;
    }

    const result = moveBoard(current.board, direction);
    if (!result.moved) {
      this.commit({ ...current, moved: false });
      return this.snapshot;
    }

    const board = addRandomTile(result.board);
    const score = current.score + result.scoreGain;
    const bestScore = Math.max(current.bestScore, score);
    const status: GameStatus = !canMove(board)
      ? "lost"
      : hasWinningTile(board) && !current.keepPlaying
        ? "won"
        : "playing";

    this.commit({ ...current, board, score, bestScore, status, moved: true });
    return this.snapshot;
  };

  restart = (): GameSnapshot => {
    this.commit(initialSnapshot(createInitialBoard(), this.snapshot.bestScore));
    return this.snapshot;
  };

  continueGame = (): GameSnapshot => {
    this.commit({ ...this.snapshot, status: "playing", keepPlaying: true, moved: false });
    return this.snapshot;
  };

  loadBestScore = (bestScore: number): void => {
    if (bestScore <= this.snapshot.bestScore) return;
    this.commit({ ...this.snapshot, bestScore, moved: false });
  };

  private commit(next: GameSnapshot): void {
    this.snapshot = next;
    this.listeners.forEach((listener) => listener());
  }
}

export const gameSession = new GameSession();
