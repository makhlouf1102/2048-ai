/**
 * Quiet mode publishes lightweight score and telemetry snapshots at a human
 * reading pace while withholding the expensive board view.
 */
export const QUIET_TELEMETRY_INTERVAL_MS = 750;
export const LIVE_MOVE_DELAY_MS = 110;

export function moveDelay(liveUpdates: boolean): number {
  return liveUpdates ? LIVE_MOVE_DELAY_MS : 0;
}
