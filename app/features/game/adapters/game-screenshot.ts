import type { Board } from "../domain/game";

const TILE_COLORS: Record<number, { background: string; foreground: string }> = {
  2: { background: "#eee9de", foreground: "#504b43" },
  4: { background: "#e9dfc9", foreground: "#504b43" },
  8: { background: "#efa160", foreground: "#ffffff" },
  16: { background: "#f18450", foreground: "#ffffff" },
  32: { background: "#f0654e", foreground: "#ffffff" },
  64: { background: "#df4935", foreground: "#ffffff" },
  128: { background: "#edcd63", foreground: "#ffffff" },
  256: { background: "#e9c451", foreground: "#ffffff" },
  512: { background: "#dfb83b", foreground: "#ffffff" },
  1024: { background: "#dba927", foreground: "#ffffff" },
  2048: { background: "#292824", foreground: "#fff4bd" },
};

function roundedRect(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void {
  context.beginPath();
  context.roundRect(x, y, width, height, radius);
  context.fill();
}

function drawScore(
  context: CanvasRenderingContext2D,
  label: string,
  value: number,
  x: number,
): void {
  context.fillStyle = "#282825";
  roundedRect(context, x, 92, 170, 92, 18);
  context.textAlign = "center";
  context.fillStyle = "#bdb9b0";
  context.font = "700 16px Inter, Arial, sans-serif";
  context.fillText(label.toUpperCase(), x + 85, 122);
  context.fillStyle = "#ffffff";
  context.font = "800 34px Inter, Arial, sans-serif";
  context.fillText(String(value), x + 85, 164);
}

export function downloadGameScreenshot(board: Board, score: number, bestScore: number): void {
  const canvas = document.createElement("canvas");
  canvas.width = 1000;
  canvas.height = 1240;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Your browser does not support screenshot creation.");

  const background = context.createLinearGradient(0, 0, canvas.width, canvas.height);
  background.addColorStop(0, "#faf7f0");
  background.addColorStop(1, "#eee9df");
  context.fillStyle = background;
  context.fillRect(0, 0, canvas.width, canvas.height);

  context.textAlign = "left";
  context.fillStyle = "#282825";
  context.font = "900 112px Inter, Arial, sans-serif";
  context.fillText("2048", 72, 170);
  context.fillStyle = "#f1644b";
  context.fillText(".", 322, 170);

  drawScore(context, "Score", score, 560);
  drawScore(context, "Best", bestScore, 748);

  const boardX = 72;
  const boardY = 250;
  const boardSize = 856;
  const gap = 18;
  const padding = 18;
  const tileSize = (boardSize - padding * 2 - gap * 3) / 4;

  context.fillStyle = "#aaa398";
  roundedRect(context, boardX, boardY, boardSize, boardSize, 28);

  board.forEach((row, rowIndex) => {
    row.forEach((value, columnIndex) => {
      const x = boardX + padding + columnIndex * (tileSize + gap);
      const y = boardY + padding + rowIndex * (tileSize + gap);
      const colors = value
        ? TILE_COLORS[value] ?? { background: "#292824", foreground: "#fff4bd" }
        : { background: "#c5beb2", foreground: "#c5beb2" };
      context.fillStyle = colors.background;
      roundedRect(context, x, y, tileSize, tileSize, 18);

      if (value) {
        const digits = String(value).length;
        context.fillStyle = colors.foreground;
        context.textAlign = "center";
        context.textBaseline = "middle";
        context.font = `900 ${digits >= 4 ? 54 : digits === 3 ? 64 : 76}px Inter, Arial, sans-serif`;
        context.fillText(String(value), x + tileSize / 2, y + tileSize / 2 + 3);
      }
    });
  });

  context.textAlign = "center";
  context.textBaseline = "alphabetic";
  context.fillStyle = "#77736c";
  context.font = "700 23px Inter, Arial, sans-serif";
  context.fillText("GAME OVER  ·  Thanks for playing", canvas.width / 2, 1165);

  const link = document.createElement("a");
  link.download = `2048-score-${score}.png`;
  link.href = canvas.toDataURL("image/png");
  link.click();
}
