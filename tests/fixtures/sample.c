/* A sample C file for testing TODO detection */

#include <stdio.h>

int main() {
    // TODO: Add command line argument parsing
    printf("Hello\n");

    /* HACK: Buffer overflow workaround
     * This needs a proper fix
     */
    char buf[256];

    return 0;
}
