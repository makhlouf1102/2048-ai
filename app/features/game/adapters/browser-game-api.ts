import type { GameSnapshot } from "../application/game-session";
import type { GameSession } from "../application/game-session";
import type { Direction } from "../domain/game";

export interface ExternalGameState {
  matrix: number[][];
  score: number;
  bestScore: number;
  status: "playing" | "won" | "lost";
  moved: boolean;
}

export interface Game2048Api {
  move(direction: Direction): ExternalGameState;
  /** Numeric directions are convenient for Wasm: 0=up, 1=right, 2=down, 3=left. */
  moveCode(direction: number): ExternalGameState;
  getState(): ExternalGameState;
  restart(): ExternalGameState;
  subscribe(listener: (state: ExternalGameState) => void): () => void;
}

declare global {
  interface Window {
    game2048: Game2048Api;
  }
}

const directionByCode: Record<0 | 1 | 2 | 3, Direction> = {
  0: "up",
  1: "right",
  2: "down",
  3: "left",
};

function externalState(snapshot: GameSnapshot): ExternalGameState {
  return {
    // Never expose the session's mutable board reference to an external player.
    matrix: snapshot.board.map((row) => [...row]),
    score: snapshot.score,
    bestScore: snapshot.bestScore,
    status: snapshot.status,
    moved: snapshot.moved,
  };
}

export function installBrowserGameApi(session: GameSession): () => void {
  const api: Game2048Api = {
    move: (direction) => externalState(session.move(direction)),
    moveCode: (direction) => {
      const mappedDirection = directionByCode[direction as 0 | 1 | 2 | 3];
      if (!mappedDirection) throw new RangeError("Direction code must be 0, 1, 2, or 3.");
      return externalState(session.move(mappedDirection));
    },
    getState: () => externalState(session.getSnapshot()),
    restart: () => externalState(session.restart()),
    subscribe: (listener) => session.subscribe(() => listener(externalState(session.getSnapshot()))),
  };

  window.game2048 = api;
  return () => {
    if (window.game2048 === api) delete (window as Partial<Window>).game2048;
  };
}
