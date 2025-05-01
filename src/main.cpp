#include "CPUPlayer.h"
#include <print>
#include <cstdlib>
#include <ctime> // Corrected header include

int main()
{
    std::srand(std::time(0)); // Seed the random number generator

    // for (int i{ 0 }; i < 5; ++i) {
    // std::println("Starting Game #{}", i + 1);
    CPUPlayer cpuPlayer;
    cpuPlayer.play();
    // std::println("Finished Game #{}\n", i + 1);
    // }

}
