import { useRef } from "react";
import { downloadGameScreenshot } from "../adapters/game-screenshot";
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

function formatDuration(milliseconds: number): string {
  if (milliseconds === 0) return "—";
  return milliseconds < 1000
    ? `${milliseconds.toFixed(1)} ms`
    : `${(milliseconds / 1000).toFixed(2)} s`;
}

type AiMood = "ready" | "thinking" | "pleased" | "focused" | "tired" | "proud" | "error";

function getAiPersona(
  ai: ReturnType<typeof useAiPlayer>,
  gameStatus: "playing" | "won" | "lost",
  highestTile: number,
): { mood: AiMood; thought: string } {
  if (ai.status === "error") return { mood: "error", thought: "I lost my train of thought. Give me another try?" };
  if (gameStatus === "lost") return { mood: "tired", thought: "That board fought back. Ready for another run?" };
  if (gameStatus === "won" || highestTile >= 2048) return { mood: "proud", thought: "We made 2048. That felt good." };
  if (ai.status === "idle" && ai.metrics.moves === 0) return { mood: "ready", thought: "I’m ready. Let me study the board." };
  if (ai.status === "idle") return { mood: "focused", thought: "Paused—but I haven’t lost the pattern." };

  const move = ai.metrics.moves;
  if (highestTile >= 1024) return { mood: "focused", thought: "1024 is on the board. Keep the corner steady." };
  if (highestTile >= 512) return { mood: "pleased", thought: "Nice. The big tiles are starting to line up." };
  if (ai.metrics.lastDecisionMs >= 1000) return { mood: "thinking", thought: "This one needs a little more thought…" };

  const thoughts = [
    "I see a path. Let’s keep the corner tidy.",
    "Good merge. I’m making room for the next one.",
    "Reading the board… there’s still space to work.",
    "One move at a time. The pattern is taking shape.",
  ];
  return { mood: move > 0 ? "pleased" : "thinking", thought: thoughts[move % thoughts.length] };
}

function AiMonitor({
  ai,
  gameStatus,
  highestTile,
}: {
  ai: ReturnType<typeof useAiPlayer>;
  gameStatus: "playing" | "won" | "lost";
  highestTile: number;
}) {
  const persona = getAiPersona(ai, gameStatus, highestTile);
  const statusLabel = ai.status === "running"
    ? "Calculating next move"
    : ai.status === "error"
      ? "Run interrupted"
      : gameStatus === "lost"
        ? "Run complete"
        : ai.metrics.moves > 0
          ? "Run paused"
          : "Ready to play";
  const lastMoveLabel = ai.metrics.lastDirection
    ? `Moved ${ai.metrics.lastDirection}`
    : ai.metrics.moves > 0
      ? "No legal move"
      : "No move yet";
  const directions = [
    { name: "up", path: "M12 19V5m-6 6 6-6 6 6" },
    { name: "left", path: "M19 12H5m6-6-6 6 6 6" },
    { name: "right", path: "M5 12h14m-6-6 6 6-6 6" },
    { name: "down", path: "M12 5v14m-6-6 6 6 6-6" },
  ] as const;

  return (
    <aside className={`ai-monitor ai-mood-${persona.mood}`} aria-label="Milo, AI player">
      <header className="ai-profile">
        <div className="ai-avatar" aria-hidden="true">
          <span className="ai-eye ai-eye-left" />
          <span className="ai-eye ai-eye-right" />
          <span className="ai-mouth" />
          <i /><i /><i />
        </div>
        <div>
          <p className="ai-name">Milo <span>Depth 5</span></p>
          <p className={`ai-status ai-status-${ai.status}`}><span />{statusLabel}</p>
        </div>
      </header>

      <p className="ai-thought" key={`${persona.mood}-${ai.metrics.moves}`} aria-live="polite">
        <span aria-hidden="true" />{persona.thought}
      </p>

      <section className={`decision-console decision-console-${ai.status}`} aria-live="polite">
        <div className="decision-heading">
          <span>Current state</span>
          <strong>Move {String(ai.metrics.moves).padStart(3, "0")}</strong>
        </div>
        <div className="direction-compass" aria-label={lastMoveLabel}>
          {directions.map(({ name, path }) => (
            <span
              className={`direction-key direction-${name}${ai.metrics.lastDirection === name ? " is-active" : ""}`}
              key={`${name}-${ai.metrics.moves}`}
              aria-hidden="true"
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d={path} />
              </svg>
            </span>
          ))}
          <div className="decision-core">
            <small>{ai.status === "running" ? "Live" : "Last"}</small>
            <strong>{lastMoveLabel}</strong>
            <span>{formatDuration(ai.metrics.lastDecisionMs)}</span>
          </div>
        </div>
      </section>

      <dl className="metric-strip">
        <div className="metric">
          <dt>Average</dt>
          <dd>{formatDuration(ai.metrics.averageDecisionMs)}</dd>
        </div>
        <div className="metric">
          <dt>Moves</dt>
          <dd>{ai.metrics.moves}</dd>
        </div>
        <div className="metric">
          <dt>Compute <span className="info-dot" title="Estimated share of autoplay time spent inside next_move. Browsers do not expose per-Wasm CPU usage.">i</span></dt>
          <dd>{ai.metrics.wasmUtilization.toFixed(1)}%</dd>
        </div>
      </dl>

      <div className="compute-track" aria-hidden="true">
        <span style={{ transform: `scaleX(${ai.metrics.wasmUtilization / 100})` }} />
      </div>
      <p className="monitor-note"><span />Telemetry refreshes after every move</p>
    </aside>
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
      <div className="game-layout">
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
                {status === "lost" && (
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => downloadGameScreenshot(board, score, bestScore)}
                  >
                    Save screenshot
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
        <AiMonitor ai={ai} gameStatus={status} highestTile={Math.max(...board.flat())} />
      </div>
    </main>
  );
}
