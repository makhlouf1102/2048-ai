import initWasm, { next_move } from "../wasm";

let initialization: Promise<unknown> | undefined;

self.onmessage = async (event: MessageEvent<{ id: number; board: number[] }>) => {
  const { id, board } = event.data;

  try {
    if (!initialization) {
      console.debug("[Milo worker] Initializing WebAssembly module.");
      initialization = initWasm();
    }
    await initialization;
    const direction = next_move(Uint32Array.from(board));
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
