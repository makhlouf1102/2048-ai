#include "CPUPlayer.h"
#include "Move.h"
#include "Board.h"
#include <vector>
#include <print>
#include <limits>

void CPUPlayer::play()
{   
    while (!board.isGameOver())
    {
        Move move = getNextMove();
        board.move(move);
    }
    board.printBoard();
    std::println("Game Over! Final Score: {}", board.score);
}

Move CPUPlayer::getNextMove()
{
    return monteCarloTreeSearch(board, 10);
}

Move CPUPlayer::getNextMoveMinMax()
{
    std::vector<Move> possibleMoves = { Move::UP, Move::DOWN, Move::LEFT, Move::RIGHT };
    int bestScore = -1;
    Move bestMove = Move::UP;
    for (Move move : possibleMoves)
    {
        // use minmaxAB to evaluate the move
        Board boardCopy = Board(board.getCopy());
        boardCopy.move(move);
        int alpha = std::numeric_limits<int>::min();
        int beta = std::numeric_limits<int>::max();
        if (boardCopy.hasChanged(board.getCopy()))
        {
            int evaluation = minmaxAB(boardCopy, 10, alpha, beta, false);
            if (evaluation > bestScore)
            {
                bestScore = evaluation;
                bestMove = move;
            }
        }
    }
    
    return bestMove;
}

int CPUPlayer::minmaxAB(Board board, int depth, int alpha, int beta, bool maximizingPlayer)
{
    if (depth == 0 || board.isGameOver())
    {
        return board.evaluate();
    }

    std::vector<Move> possibleMoves = { Move::UP, Move::DOWN, Move::LEFT, Move::RIGHT };
    if (maximizingPlayer)
    {
        int maxEval = std::numeric_limits<int>::min();
        for (Move move : possibleMoves)
        {
            Board boardCopy = Board(board.getCopy());
            boardCopy.move(move);
            if (boardCopy.hasChanged(board.getCopy()))
            {
                int eval = minmaxAB(boardCopy, depth - 1, alpha, beta, false);
                maxEval = std::max(maxEval, eval);
                alpha = std::max(alpha, eval);
                if (beta <= alpha)
                {
                    break;
                }
            }
        }
        return maxEval;
    }
    else
    {
        int minEval = std::numeric_limits<int>::max();
        for (Move move : possibleMoves)
        {
            Board boardCopy = Board(board.getCopy());z
            boardCopy.move(move);
            if (boardCopy.hasChanged(board.getCopy()))
            {
                int eval = minmaxAB(boardCopy, depth - 1, alpha, beta, true);
                minEval = std::min(minEval, eval);
                beta = std::min(beta, eval);
                if (beta <= alpha)
                {
                    break;
                }
            }
        }
        return minEval;
    }
}

Move CPUPlayer::monteCarloTreeSearch(Board board, int iterations)
{
    std::vector<Move> possibleMoves = { Move::UP, Move::DOWN, Move::LEFT, Move::RIGHT };
    Move bestMove = Move::UP;
    int bestScore = -1;

    for (Move move : possibleMoves)
    {
        int totalScore = 0;
        int simulations = 0;

        for (int i = 0; i < iterations; ++i)
        {
            Board boardCopy = Board(board.getCopy());
            boardCopy.move(move);
            if (boardCopy.hasChanged(board.getCopy()))
            {
                int score = simulateGame(boardCopy);
                totalScore += score;
                ++simulations;
            }
        }

        if (simulations > 0)
        {
            int averageScore = totalScore / simulations;
            if (averageScore > bestScore)
            {
                bestScore = averageScore;
                bestMove = move;
            }
        }
    }

    return bestMove;
}

int CPUPlayer::simulateGame(Board board)
{
    std::vector<Move> possibleMoves = { Move::UP, Move::DOWN, Move::LEFT, Move::RIGHT };
    while (!board.isGameOver())
    {
        Move bestMove = Move::UP;
        int bestEmptyTiles = -1;

        for (Move move : possibleMoves)
        {
            Board boardCopy = Board(board.getCopy());
            boardCopy.move(move);
            if (boardCopy.hasChanged(board.getCopy()))
            {
                int emptyTiles = boardCopy.getEmptyTiles().size();
                if (emptyTiles > bestEmptyTiles)
                {
                    bestEmptyTiles = emptyTiles;
                    bestMove = move;
                }
            }
        }

        board.move(bestMove);
    }

    return board.score; // Still using number of empty tiles for final evaluation
}

