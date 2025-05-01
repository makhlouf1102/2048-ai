#ifndef BOARD_H
#define BOARD_H
#include <vector>
#include "Move.h"
#include <array>


class Board 
{
    public:
        static const int SIZE{ 4 };
        std::array<std::array<int, SIZE>, SIZE> board{};
        
        int score{ 0 }; 
        static const int PROBABILITY_ADD_TILE_TWO{ 90 };
        static const int PROBABILITY_ADD_TILE_FOUR{ 10 };

        Board();
        Board(std::array<std::array<int, SIZE>, SIZE> board);
        void printBoard();
        void addTile();
        void move(Move move);
        bool hasChanged(std::array<std::array<int, SIZE>, SIZE> oldBoard);
        void moveLeft();
        void moveRight();
        void moveUp();
        void moveDown();
        bool canMove();
        bool canMove(Move move);
        bool isGameOver();
        int evaluate();
        std::array<std::array<int, SIZE>, SIZE> getCopy();
        std::vector<std::pair<int, int>> getEmptyTiles();
        int HighestTile();
};

#endif

