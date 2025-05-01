#include "Board.h"
#include <array>
#include <vector>
#include <ctime>
#include <print>
#include "Move.h"
#include <cstdlib>

Board::Board()
{
    addTile();
    addTile();
}

Board::Board(std::array<std::array<int, SIZE>, SIZE> board) : board(board){}

void Board::printBoard() 
{
    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ 0 }; j < SIZE; j++) 
        {
            std::print("{} ", board[i][j]);
        }
        std::println();
    }
    std::println("=========================");
    return;
}

void Board::move(Move move) 
{
    std::array<std::array<int, SIZE>, SIZE> oldBoard{ getCopy() } ;

    switch (move) 
    {
        case UP:
            moveUp();
            break;
        case DOWN:
            moveDown();
            break;
        case LEFT:
            moveLeft();
            break;
        case RIGHT:
            moveRight();
            break;
    }
    if (hasChanged(oldBoard)) 
    {
        addTile();
    }
    // else 
    // {
    //     std::println("No changes made to the board for move {}", static_cast<int>(move));
    // }
}

bool Board::hasChanged(std::array<std::array<int, SIZE>, SIZE> oldBoard)
{
    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ 0 }; j < SIZE; j++) 
        {
            if (board[i][j] != oldBoard[i][j]) 
            {
                return true;
            }
        }
    }
    return false;
}

std::vector<std::pair<int, int>> Board::getEmptyTiles()
{
    std::vector<std::pair<int, int>> emptyTiles;
    for (int i = 0; i < SIZE; i++) 
    {
        for (int j = 0; j < SIZE; j++) 
        {
            if (board[i][j] == 0) 
            {
                emptyTiles.push_back(std::make_pair(i, j));
            }
        }
    }
    return emptyTiles;
}

void Board::addTile()
{
    std::vector<std::pair<int, int>> emptyTiles = getEmptyTiles();
    if (emptyTiles.empty()) 
    {
        return;
    }

    int randomIndex = std::rand() % emptyTiles.size();
    int randomValue = (std::rand() % 100 < PROBABILITY_ADD_TILE_FOUR) ? 4 : 2;
    board[emptyTiles[randomIndex].first][emptyTiles[randomIndex].second] = randomValue;
    return;
}

void Board::moveLeft()
{
    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ 0 }; j < SIZE - 1; j++) 
        {
            if (board[i][j] == 0) 
            {
                for (int k { j + 1 }; k < SIZE; k++) 
                {
                    if (board[i][k] != 0) 
                    {
                        board[i][j] = board[i][k];
                        board[i][k] = 0;
                        break;
                    }
                }
            }
        }
    }

    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ 0 }; j < SIZE - 1; j++) 
        {
            if (board[i][j] == board[i][j + 1]) 
            {
                board[i][j] *= 2;
                score += board[i][j];
                board[i][j + 1] = 0;
            }
        }
    }
}

void Board::moveRight()
{
    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ SIZE - 1 }; j > 0; j--) 
        {
            if (board[i][j] == 0) 
            {
                for (int k{ j - 1 }; k >= 0; k--) 
                {
                    if (board[i][k] != 0) 
                    {
                        board[i][j] = board[i][k];
                        board[i][k] = 0;
                        break;
                    }
                }
            }
        }
    }

    for (int i = 0; i < SIZE; i++) 
    {
        for (int j = SIZE - 1; j > 0; j--) 
        {
            if (board[i][j] == board[i][j - 1]) 
            {
                board[i][j] *= 2;
                score += board[i][j];
                board[i][j - 1] = 0;
            }
        }
    }
}

void Board::moveUp()
{
    for (int j{ 0 }; j < SIZE; j++) 
    {
        for (int i{ 0 }; i < SIZE - 1; i++) 
        {
            if (board[i][j] == 0) 
            {
                for (int k{ i + 1 }; k < SIZE; k++) 
                {
                    if (board[k][j] != 0) 
                    {
                        board[i][j] = board[k][j];
                        board[k][j] = 0;
                        break;
                    }
                }
            }
        }
    }

    for (int j = 0; j < SIZE; j++) 
    {
        for (int i = 0; i < SIZE - 1; i++) 
        {
            if (board[i][j] == board[i + 1][j]) 
            {
                board[i][j] *= 2;
                score += board[i][j];
                board[i + 1][j] = 0;
            }
        }
    }
}

void Board::moveDown()
{
    for (int j{ 0 }; j < SIZE; j++) 
    {
        for (int i{ SIZE - 1 }; i > 0; i--) 
        {
            if (board[i][j] == 0) 
            {
                for (int k{ i - 1 }; k >= 0; k--) 
                {
                    if (board[k][j] != 0) 
                    {
                        board[i][j] = board[k][j];
                        board[k][j] = 0;
                        break;
                    }
                }
            }
        }
    }

    for (int j = 0; j < SIZE; j++) 
    {
        for (int i = SIZE - 1; i > 0; i--) 
        {
            if (board[i][j] == board[i - 1][j]) 
            {
                board[i][j] *= 2;
                score += board[i][j];
                board[i - 1][j] = 0;
            }
        }
    }
}

bool Board::canMove()
{
    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ 0 }; j < SIZE; j++) 
        {
            if (board[i][j] == 0) 
            {
                return true;
            }
            if (i < SIZE - 1 && board[i][j] == board[i + 1][j]) 
            {
                return true;
            }
            if (j < SIZE - 1 && board[i][j] == board[i][j + 1]) 
            {
                return true;
            }
        }
    }
    return false;
}

bool Board::canMove(Move move)
{
    std::array<std::array<int, SIZE>, SIZE> oldBoard{ getCopy() };
    switch (move) 
    {
        case UP:
            moveUp();
            break;
        case DOWN:
            moveDown();
            break;
        case LEFT:
            moveLeft();
            break;
        case RIGHT:
            moveRight();
            break;
    }
    board = oldBoard; // revert the board to its original state
    return hasChanged(oldBoard);
}

bool Board::isGameOver()
{
    return !canMove();
}

int Board::HighestTile()
{
    int highestTile{ 0 };
    for (int i{ 0 }; i < SIZE; i++) 
    {
        for (int j{ 0 }; j < SIZE; j++) 
        {
            if (board[i][j] > highestTile) 
            {
                highestTile = board[i][j];
            }
        }
    }
    return highestTile;
}
int Board::evaluate()
{
    int emptyTiles = getEmptyTiles().size();
    int highestTile = HighestTile();
    int gameScore = score;
    int evaluation = emptyTiles * 10 + highestTile * 2 + gameScore;
    return evaluation;
}

std::array<std::array<int, Board::SIZE>, Board::SIZE> Board::getCopy()
{
    std::array<std::array<int, SIZE>, SIZE> copy;
    for (int i{ 0 }; i < SIZE; ++i) 
    {
        for (int j{ 0 }; j < SIZE; ++j) 
        {
            copy[i][j] = board[i][j];
        }
    }
    return copy;
}


