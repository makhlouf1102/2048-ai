#ifndef CPUPLAYER_H
#define CPUPLAYER_H

#include "Board.h"

class CPUPlayer
{
    public:
        Board board;

        void play();
        Move getNextMove();
        Move getNextMoveMinMax();
        Move monteCarloTreeSearch(Board board, int iterations);
        Board getBoard() { return board; }
        int simulateGame(Board board);
        int minmaxAB(Board board, int depth, int alpha, int beta, bool maximizingPlayer);
};

#endif