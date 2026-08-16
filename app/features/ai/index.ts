interface WorkerReply {
  id: number;
  direction?: number;
  error?: string;
}

interface PendingDecision {
  reject: (reason: Error) => void;
  resolve: (direction: number) => void;
}

let worker: Worker | undefined;
let nextRequestId = 0;
const pendingDecisions = new Map<number, PendingDecision>();

function getWorker(): Worker {
  if (worker) return worker;

  worker = new Worker(new URL("./ai.worker.ts", import.meta.url), { type: "module" });
  worker.onmessage = (event: MessageEvent<WorkerReply>) => {
    const { id, direction, error } = event.data;
    const pending = pendingDecisions.get(id);
    if (!pending) return;

    pendingDecisions.delete(id);
    if (error) pending.reject(new Error(error));
    else if (typeof direction === "number") pending.resolve(direction);
    else pending.reject(new Error("The AI worker returned an invalid response."));
  };
  worker.onerror = () => {
    for (const pending of pendingDecisions.values()) {
      pending.reject(new Error("The AI worker stopped unexpectedly."));
    }
    pendingDecisions.clear();
    worker?.terminate();
    worker = undefined;
  };

  return worker;
}

/** Runs the Wasm search away from the UI thread. */
export function nextMove(board: number[]): Promise<number> {
  const id = ++nextRequestId;
  return new Promise((resolve, reject) => {
    pendingDecisions.set(id, { resolve, reject });
    getWorker().postMessage({ id, board });
  });
}

/** Immediately aborts in-flight search so controls stay responsive. */
export function cancelAiWork(): void {
  if (!worker) return;

  worker.terminate();
  worker = undefined;
  for (const pending of pendingDecisions.values()) {
    pending.reject(new Error("AI run stopped."));
  }
  pendingDecisions.clear();
}
