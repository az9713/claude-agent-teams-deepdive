// A sample C++ file for testing TODO detection

#include <iostream>

int main() {
    // TODO: Use smart pointers
    int* ptr = new int(42);

    // XXX: Memory leak here
    std::cout << *ptr << std::endl;

    return 0;
}
