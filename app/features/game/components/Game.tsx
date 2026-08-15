import { useRef } from "react";
import type { Direction } from "../domain/game";
import { useAiPlayer } from "../hooks/useAiPlayer";
import { useGame } from "../hooks/useGame";

const tileNames: Record<number, string> = {
  2: "two", 4: "four", 8: "eight", 16: "sixteen", 32: "thirty-two",
  64: "sixty-four", 128: "one-twenty-eight", 256: "two-fifty-six",
  512: "five-twelve", 1024: "one-thousand", 2048: "two-thousand",
};

function ScoreCard({ label, value }: { label: string; value: number }) {
  return (
    <div className="score-card" aria-label={`${label}: ${value}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

export function Game() {
  const { board, score, bestScore, status, move, restart, continueGame } = useGame();
  const ai = useAiPlayer();
  const touchStart = useRef<{ x: number; y: number } | null>(null);

  const finishSwipe = (x: number, y: number) => {
    if (!touchStart.current) return;
    const deltaX = x - touchStart.current.x;
    const deltaY = y - touchStart.current.y;
    touchStart.current = null;
    if (Math.max(Math.abs(deltaX), Math.abs(deltaY)) < 30) return;
    const direction: Direction = Math.abs(deltaX) > Math.abs(deltaY)
      ? deltaX > 0 ? "right" : "left"
      : deltaY > 0 ? "down" : "up";
    move(direction);
  };

  return (
    <main className="game-shell">
      <section className="game" aria-labelledby="game-title">
        <header className="game-header">
          <div className="brand-block">
            <p className="eyebrow">The classic number puzzle</p>
            <h1 id="game-title">2048<span>.</span></h1>
          </div>
          <div className="score-area">
            <ScoreCard label="Score" value={score} />
            <ScoreCard label="Best" value={bestScore} />
          </div>
        </header>

        <div className="game-toolbar">
          <p>Join matching tiles and reach <strong>2048.</strong></p>
          <div className="toolbar-actions">
            <button
              className="ai-button"
              type="button"
              onClick={ai.status === "running" ? ai.stop : ai.start}
              aria-pressed={ai.status === "running"}
            >
              <span className="ai-indicator" aria-hidden="true" />
              {ai.status === "running" ? "Stop AI" : "Let AI play"}
            </button>
            <button className="new-game" type="button" onClick={() => { ai.stop(); restart(); }}>
              New game
            </button>
          </div>
        </div>

        {ai.error && <p className="ai-error" role="alert">{ai.error}</p>}

        <div
          className="board-wrap"
          onTouchStart={(event) => {
            const touch = event.touches[0];
            touchStart.current = { x: touch.clientX, y: touch.clientY };
          }}
          onTouchEnd={(event) => {
            const touch = event.changedTouches[0];
            finishSwipe(touch.clientX, touch.clientY);
          }}
        >
          <div className="board" role="grid" aria-label="2048 game board">
            {board.flatMap((row, rowIndex) =>
              row.map((value, columnIndex) => (
                <div
                  className={`tile ${value ? `tile-${tileNames[value] ?? "super"}` : "tile-empty"}`}
                  key={`${rowIndex}-${columnIndex}`}
                  role="gridcell"
                  aria-label={value ? String(value) : "Empty"}
                >
                  {value || ""}
                </div>
              )),
            )}
          </div>

          {status !== "playing" && (
            <div className="game-overlay" role="dialog" aria-live="assertive">
              <p>{status === "won" ? "You made 2048!" : "Game over"}</p>
              <div className="overlay-actions">
                {status === "won" && (
                  <button type="button" className="secondary-button" onClick={continueGame}>
                    Keep going
                  </button>
                )}
                <button type="button" className="new-game" onClick={restart}>Try again</button>
              </div>
            </div>
          )}
        </div>

        <footer className="game-footer">
          <div className="key-hint" aria-hidden="true">
            <span>↑</span><span>←</span><span>↓</span><span>→</span>
          </div>
          <p><strong>How to play</strong><br />Use your arrow keys, WASD, or swipe to move the tiles.</p>
        </footer>
      </section>
    </main>
  );
}
