import initWasm, {
  next_move_depth,
  next_move_model,
  next_move_random_simulation,
} from "../wasm";

import type { AiStrategy } from ".";

const SEARCH_DEPTH = 3;

let initialization: Promise<unknown> | undefined;

self.onmessage = async (
  event: MessageEvent<{ id: number; board: number[]; strategy: AiStrategy }>,
) => {
  const { id, board, strategy } = event.data;

  try {
    if (!initialization) {
      console.debug("[Milo worker] Initializing WebAssembly module.");
      initialization = initWasm();
    }
    await initialization;
    const wasmBoard = Uint32Array.from(board);
    const direction = strategy === "depth"
      ? next_move_depth(wasmBoard, SEARCH_DEPTH)
      : strategy === "random-simulation"
        ? next_move_random_simulation(wasmBoard)
        : next_move_model(wasmBoard);
    self.postMessage({ id, direction });
  } catch (cause) {
    const error = cause instanceof Error ? cause : new Error(String(cause));

    console.error("[Milo worker] Failed to choose a move.", {
      requestId: id,
      board,
      errorName: error.name,
      errorMessage: error.message,
      stack: error.stack,
      cause,
    });

    self.postMessage({
      id,
      error: error.message || "The AI worker failed.",
    });
  }
};
