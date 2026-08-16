import type { AiViewMode } from "../hooks/useAiPresentation";

export function CookingScreen({
  onSnapshot,
  onWatchLive,
}: {
  onSnapshot: () => void;
  onWatchLive: () => void;
}) {
  return (
    <section className="cooking-screen" aria-live="polite" aria-label="Milo is playing without live board updates">
      <div className="cooking-orbit" aria-hidden="true">
        <span /><span /><span />
        <img src={`${import.meta.env.BASE_URL}assets/milo-avatar-painted.png`} alt="" />
      </div>
      <div className="cooking-copy">
        <h2>Milo is cooking.</h2>
        <p>He’s playing off-screen so the board doesn’t repaint after every move. Score and studio notes update in calm batches.</p>
        <div className="cooking-actions">
          <button className="snapshot-button" type="button" onClick={onSnapshot}>
            Show me a snapshot
          </button>
          <button className="secondary-button" type="button" onClick={onWatchLive}>
            Watch moves live
          </button>
        </div>
        <p className="cooking-note">A snapshot freezes the exact board you request. Milo keeps playing behind it.</p>
      </div>
    </section>
  );
}

export function RunViewControl({
  mode,
  moveCount,
  onQuiet,
  onRefreshSnapshot,
  onWatchLive,
}: {
  mode: AiViewMode;
  moveCount: number;
  onQuiet: () => void;
  onRefreshSnapshot: () => void;
  onWatchLive: () => void;
}) {
  const isLive = mode === "live";

  return (
    <div className={`view-control view-control-${mode}`} role="status">
      <div>
        <strong>{isLive ? "Watching Milo live" : `Snapshot · move ${moveCount}`}</strong>
        <span>{isLive ? "The board refreshes after every move." : "Milo is still playing off-screen."}</span>
      </div>
      <div className="view-control-actions">
        {!isLive && (
          <button type="button" className="secondary-button" onClick={onRefreshSnapshot}>
            Refresh snapshot
          </button>
        )}
        <button type="button" className="secondary-button" onClick={isLive ? onQuiet : onWatchLive}>
          {isLive ? "Hide live board" : "Watch live"}
        </button>
      </div>
    </div>
  );
}
