import { useEffect, useRef, useState } from "react";
import { downloadGameScreenshot } from "../adapters/game-screenshot";
import type { Direction } from "../domain/game";
import { useAiPresentation } from "../hooks/useAiPresentation";
import { useAiPlayer } from "../hooks/useAiPlayer";
import { useGame } from "../hooks/useGame";
import { CookingScreen, RunViewControl } from "./AiRunDisplay";

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

function chartPoints(values: number[], width = 160, height = 42): string {
  const samples = values.length > 1 ? values : [values[0] ?? 0, values[0] ?? 0];
  const maximum = Math.max(...samples, 1);
  const minimum = Math.min(...samples);
  const range = Math.max(maximum - minimum, maximum * .2, 1);
  return samples.map((value, index) => {
    const x = (index / (samples.length - 1)) * width;
    const y = height - 4 - ((value - minimum) / range) * (height - 10);
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  }).join(" ");
}

function LineChart({ values, tone }: { values: number[]; tone: "green" | "blue" }) {
  const points = chartPoints(values);
  const areaPoints = `0,42 ${points} 160,42`;
  return (
    <svg className={`metric-chart metric-chart-${tone}`} viewBox="0 0 160 42" preserveAspectRatio="none" aria-hidden="true">
      <polygon className="metric-chart-area" points={areaPoints} />
      <polyline className="metric-chart-line" points={points} />
      {values.length > 0 && <circle cx={points.split(" ").at(-1)?.split(",")[0]} cy={points.split(" ").at(-1)?.split(",")[1]} r="2.5" />}
    </svg>
  );
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
  if (ai.metrics.lastScoreGain >= 256) return { mood: "proud", thought: "That changed everything. The board just opened up." };
  if (ai.metrics.lastScoreGain >= 64) return { mood: "pleased", thought: "There it is—a big merge and room to move." };
  if (highestTile >= 1024) return { mood: "focused", thought: "1024 is on the board. Keep the corner steady." };
  if (highestTile >= 512) return { mood: "pleased", thought: "Nice. The big tiles are starting to line up." };
  if (ai.metrics.lastDecisionMs >= 1000) return { mood: "thinking", thought: "This one needs a little more thought…" };

  const thoughts = [
    "I see a path. Let’s keep the corner tidy.",
    "Good merge. I’m making room for the next one.",
    "Reading the board… there’s still space to work.",
    "One move at a time. The pattern is taking shape.",
  ];
  return { mood: move > 0 ? "pleased" : "thinking", thought: thoughts[Math.floor(move / 8) % thoughts.length] };
}

function AiMonitor({
  ai,
  gameStatus,
  highestTile,
  liveUpdates,
}: {
  ai: ReturnType<typeof useAiPlayer>;
  gameStatus: "playing" | "won" | "lost";
  highestTile: number;
  liveUpdates: boolean;
}) {
  const [visibleMetrics, setVisibleMetrics] = useState(ai.metrics);
  const lastRevealAt = useRef(0);
  const holdBigMoveUntil = useRef(0);

  useEffect(() => {
    const now = performance.now();
    const isIncomingBigMove = ai.metrics.lastScoreGain >= 64;

    if (isIncomingBigMove) {
      setVisibleMetrics(ai.metrics);
      lastRevealAt.current = now;
      holdBigMoveUntil.current = now + 3200;
      return;
    }

    if (now < holdBigMoveUntil.current || now - lastRevealAt.current < 1600) return;
    setVisibleMetrics(ai.metrics);
    lastRevealAt.current = now;
  }, [ai.metrics]);

  const visibleAi = { ...ai, metrics: visibleMetrics };
  const persona = getAiPersona(visibleAi, gameStatus, highestTile);
  const isBigMove = visibleMetrics.lastScoreGain >= 64;
  const statusLabel = ai.status === "running"
    ? "Calculating next move"
    : ai.status === "error"
      ? "Run interrupted"
      : gameStatus === "lost"
        ? "Run complete"
        : ai.metrics.moves > 0
          ? "Run paused"
          : "Ready to play";
  const lastMoveLabel = visibleMetrics.lastDirection
    ? `Moved ${visibleMetrics.lastDirection}`
    : visibleMetrics.moves > 0
      ? "No legal move"
      : "No move yet";
  const directions = [
    { name: "up", path: "M12 19V5m-6 6 6-6 6 6" },
    { name: "left", path: "M19 12H5m6-6-6 6 6 6" },
    { name: "right", path: "M5 12h14m-6-6 6 6-6 6" },
    { name: "down", path: "M12 5v14m-6-6 6 6 6-6" },
  ] as const;
  const activeDirection = directions.find(({ name }) => name === visibleMetrics.lastDirection) ?? directions[0];

  return (
    <aside className={`ai-monitor ai-mood-${persona.mood}`} aria-label="Milo, AI player">
      <div className="studio-sun" aria-hidden="true"><span /><span /><span /></div>
      <header className="ai-profile">
        <div className="ai-avatar" aria-hidden="true">
          <img src={`${import.meta.env.BASE_URL}assets/milo-avatar-painted.png`} alt="" />
        </div>
        <div>
          <p className="ai-name">Milo <span>thinks deeper in tight spots</span></p>
          <p className={`ai-status ai-status-${ai.status}`}><span />{statusLabel}</p>
        </div>
      </header>

      <p className="ai-thought" key={persona.thought} aria-live="polite">
        <span aria-hidden="true" />{persona.thought}
      </p>

      <section className={`decision-console decision-console-${ai.status}${isBigMove ? " decision-console-big" : ""}`} aria-live="polite">
        <div className="decision-heading">
          <strong>Milo’s move {String(visibleMetrics.moves).padStart(2, "0")}</strong>
        </div>
        <div className="direction-compass" aria-label={lastMoveLabel}>
          <span className="direction-key is-active" key={`${activeDirection.name}-${visibleMetrics.moves}`} aria-hidden="true">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d={activeDirection.path} /></svg>
          </span>
          <div className="decision-core">
            {isBigMove && <span className="move-score" key={`${visibleMetrics.moves}-${visibleMetrics.lastScoreGain}`}>+{visibleMetrics.lastScoreGain}</span>}
            <strong>{isBigMove ? `Big merge — ${lastMoveLabel}` : ai.status === "running" ? `Choosing — ${lastMoveLabel}` : `Last move — ${lastMoveLabel}`}</strong>
            <span className="decision-time">{formatDuration(visibleMetrics.lastDecisionMs)}</span>
          </div>
        </div>
      </section>

      <dl className="metric-strip" aria-label="Milo's pace and activity">
        <div className="metric">
          <dt>Thinking pace</dt>
          <dd>{formatDuration(ai.metrics.averageDecisionMs)}</dd>
          <LineChart values={ai.metrics.decisionHistory} tone="green" />
        </div>
        <div className="metric">
          <dt>Steps taken</dt>
          <dd>{ai.metrics.moves}</dd>
        </div>
        <div className="metric">
          <dt>Busy time <span className="info-dot" title="Estimated share of autoplay time spent choosing moves. Browsers do not expose exact WebAssembly CPU usage.">i</span></dt>
          <dd>{ai.metrics.wasmUtilization.toFixed(1)}%</dd>
          <LineChart values={ai.metrics.utilizationHistory} tone="blue" />
        </div>
      </dl>
      <p className="monitor-note">
        <span />{liveUpdates ? "Milo’s notes refresh after every move" : "Notes wait until you request a snapshot"}
      </p>
    </aside>
  );
}

export function Game() {
  const presentation = useAiPresentation();
  const game = useGame(presentation.isLive);
  const ai = useAiPlayer({ liveUpdates: presentation.isLive });
  const { board, score, bestScore, status, move, restart, continueGame } = game;
  const touchStart = useRef<{ x: number; y: number } | null>(null);
  const isQuietRun = ai.status === "running" && !presentation.isLive;

  useEffect(() => {
    if (ai.status === "running") return;
    game.refresh();
    ai.refresh();
  }, [ai.status, ai.refresh, game.refresh]);

  const showSnapshot = () => {
    game.refresh();
    ai.refresh();
    presentation.revealSnapshot();
  };

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
            <h1 id="game-title"><span>20</span><span>48</span></h1>
            <p>Make matching numbers meet.</p>
          </div>
          <div className="score-area">
            <ScoreCard label="Score" value={score} />
            <ScoreCard label="Best" value={bestScore} />
          </div>
        </header>

        <div className="game-toolbar">
          <p>Use the board yourself, or let Milo find a path to <strong>2048.</strong></p>
          <div className="toolbar-actions">
            <button
              className="ai-button"
              type="button"
              onClick={ai.status === "running" ? ai.stop : () => {
                presentation.beginRun();
                void ai.start();
              }}
              aria-pressed={ai.status === "running"}
              aria-label={ai.status === "running" ? "Stop AI player" : "Start AI player"}
            >
              <span className="ai-indicator" aria-hidden="true" />
              {ai.status === "running" ? "Pause Milo" : "Let Milo play"}
            </button>
            <button className="new-game" type="button" onClick={() => { ai.stop(); restart(); }}>
              Mix a new board
            </button>
          </div>
        </div>

        {ai.error && <p className="ai-error" role="alert">{ai.error}</p>}

        {isQuietRun && !presentation.snapshotVisible ? (
          <CookingScreen onSnapshot={showSnapshot} onWatchLive={presentation.watchLive} />
        ) : (
          <>
            {ai.status === "running" && (
              <RunViewControl
                mode={presentation.mode}
                moveCount={ai.metrics.moves}
                onQuiet={presentation.enterQuietMode}
                onRefreshSnapshot={showSnapshot}
                onWatchLive={presentation.watchLive}
              />
            )}

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
              <div className="board" role="grid" aria-label={presentation.isLive ? "Live 2048 game board" : "2048 game board snapshot"}>
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
          </>
        )}

        <footer className="game-footer">
          <div className="key-hint" aria-hidden="true">
            <span>↑</span><span>←</span><span>↓</span><span>→</span>
          </div>
          <p>
            <strong>{ai.status === "running" ? "Milo’s turn" : "Your turn"}</strong><br />
            {ai.status === "running"
              ? presentation.isLive ? "You’re watching every move as it lands." : "Quiet mode saves the board work until you ask to see it."
              : "Use arrow keys, WASD, or swipe. Equal tiles join when they touch."}
          </p>
        </footer>
        </section>
        <AiMonitor
          ai={ai}
          gameStatus={status}
          highestTile={Math.max(...board.flat())}
          liveUpdates={presentation.isLive}
        />
      </div>
    </main>
  );
}
