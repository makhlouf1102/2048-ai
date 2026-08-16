import initWasm, { next_move } from "../wasm";

let initialization: Promise<unknown> | undefined;

self.onmessage = async (event: MessageEvent<{ id: number; board: number[] }>) => {
  const { id, board } = event.data;

  try {
    initialization ??= initWasm();
    await initialization;
    const direction = next_move(Uint32Array.from(board));
    self.postMessage({ id, direction });
  } catch (cause) {
    self.postMessage({
      id,
      error: cause instanceof Error ? cause.message : "The AI worker failed.",
    });
  }
};
