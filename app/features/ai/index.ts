import initWasm, { next_move } from "../wasm";

let initialization: Promise<unknown> | undefined;

/** Calls the compiled Wasm AI with the board flattened in row-major order. */
export async function nextMove(matrix: number[]): Promise<number> {
  initialization ??= initWasm();
  await initialization;
  return next_move(Uint32Array.from(matrix));
}
