# 2048

A responsive 2048 game built with React Router and a clean, framework-independent game engine. It supports keyboard, WASD, touch, and programmatic players hosted by JavaScript or WebAssembly.

## Development

```bash
npm install
npm run dev
```

The app is served under `/2048-ai/` to match its GitHub Pages project URL.

## Programmatic and WebAssembly API

Once the page has mounted, the game exposes `window.game2048`. Every move is synchronous and returns feedback containing the resulting 4×4 matrix.

```js
const feedback = window.game2048.move("left");

console.log(feedback);
// {
//   matrix: [[0, 0, 2, 4], [0, 0, 0, 2], ...],
//   score: 4,
//   bestScore: 128,
//   status: "playing",
//   moved: true
// }
```

Numeric directions are easier to pass from a WebAssembly host:

```js
// 0 = up, 1 = right, 2 = down, 3 = left
const feedback = window.game2048.moveCode(0);
const matrix = feedback.matrix;
```

The remaining API is:

```js
window.game2048.getState();
window.game2048.restart();

const unsubscribe = window.game2048.subscribe((state) => {
  // Called after UI or programmatic moves.
  console.log(state.matrix);
});
```

Raw WebAssembly functions can only exchange numeric values, so the JavaScript host should call `moveCode` and then copy `feedback.matrix.flat()` into Wasm memory when needed. The UI and external API share the same `GameSession`; a programmatic move is rendered immediately in the browser.

## AI autoplay contract

The **Let AI play** button repeatedly calls:

```ts
nextMove(matrix: number[]): Promise<number>
```

`matrix` contains all 16 cells in row-major order. The returned direction is `0=up`, `1=right`, `2=down`, or `3=left`; `-1` means no move is available. Autoplay stops cleanly on `-1` or game over, when the user presses **Stop AI**, or when the AI returns an invalid or blocked move.

## Build

```bash
npm run typecheck
npm run build
```

Deploy `build/client` to GitHub Pages. Runtime server-side rendering is disabled.
