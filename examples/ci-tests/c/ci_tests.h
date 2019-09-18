#include <stdio.h>
#include <assert.h>

#include "ci_tests_api.h"
#include "ci_tests_gen.h"

#define ____ 0x00

void test_start(char *title) {
    char line[80+1];
    memset(line, '-', 80);
    line[80] = '\0';
    printf("%s\n", line);
    memset(line, ' ', 80);
    line[32] = '\0';
    printf("%s%s\n", line, title);
}
