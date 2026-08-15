export const BOARD_SIZE = 4;
export const WINNING_TILE = 2048;

export type Direction = "up" | "down" | "left" | "right";
export type Board = number[][];

export interface MoveResult {
  board: Board;
  scoreGain: number;
  moved: boolean;
}

export function createEmptyBoard(): Board {
  return Array.from({ length: BOARD_SIZE }, () =>
    Array<number>(BOARD_SIZE).fill(0),
  );
}

export function addRandomTile(board: Board, random = Math.random): Board {
  const emptyCells: Array<[number, number]> = [];

  board.forEach((row, rowIndex) =>
    row.forEach((value, columnIndex) => {
      if (value === 0) emptyCells.push([rowIndex, columnIndex]);
    }),
  );

  if (emptyCells.length === 0) return board.map((row) => [...row]);

  const [row, column] = emptyCells[Math.floor(random() * emptyCells.length)];
  const nextBoard = board.map((line) => [...line]);
  nextBoard[row][column] = random() < 0.9 ? 2 : 4;
  return nextBoard;
}

export function createInitialBoard(random = Math.random): Board {
  return addRandomTile(addRandomTile(createEmptyBoard(), random), random);
}

function collapseLine(line: number[]): { line: number[]; scoreGain: number } {
  const values = line.filter(Boolean);
  const collapsed: number[] = [];
  let scoreGain = 0;

  for (let index = 0; index < values.length; index += 1) {
    if (values[index] === values[index + 1]) {
      const merged = values[index] * 2;
      collapsed.push(merged);
      scoreGain += merged;
      index += 1;
    } else {
      collapsed.push(values[index]);
    }
  }

  return {
    line: [...collapsed, ...Array(BOARD_SIZE - collapsed.length).fill(0)],
    scoreGain,
  };
}

function boardsMatch(first: Board, second: Board): boolean {
  return first.every((row, rowIndex) =>
    row.every((value, columnIndex) => value === second[rowIndex][columnIndex]),
  );
}

export function moveBoard(board: Board, direction: Direction): MoveResult {
  const vertical = direction === "up" || direction === "down";
  const reverse = direction === "right" || direction === "down";
  const nextBoard = createEmptyBoard();
  let scoreGain = 0;

  for (let lineIndex = 0; lineIndex < BOARD_SIZE; lineIndex += 1) {
    const original = Array.from({ length: BOARD_SIZE }, (_, cellIndex) =>
      vertical ? board[cellIndex][lineIndex] : board[lineIndex][cellIndex],
    );
    const oriented = reverse ? [...original].reverse() : original;
    const collapsed = collapseLine(oriented);
    const result = reverse ? collapsed.line.reverse() : collapsed.line;
    scoreGain += collapsed.scoreGain;

    result.forEach((value, cellIndex) => {
      if (vertical) nextBoard[cellIndex][lineIndex] = value;
      else nextBoard[lineIndex][cellIndex] = value;
    });
  }

  return { board: nextBoard, scoreGain, moved: !boardsMatch(board, nextBoard) };
}

export function hasWinningTile(board: Board): boolean {
  return board.some((row) => row.some((value) => value >= WINNING_TILE));
}

export function canMove(board: Board): boolean {
  if (board.some((row) => row.includes(0))) return true;

  return board.some((row, rowIndex) =>
    row.some(
      (value, columnIndex) =>
        value === row[columnIndex + 1] ||
        value === board[rowIndex + 1]?.[columnIndex],
    ),
  );
}
