// All literal C file templates. These are the exact same content the bash newc generated,
// minus the author/date interpolation which is done in scaffold.rs.

pub const MAKEFILE: &str = r#"CC = gcc

# =========================
# Warning levels
# =========================
BASE_WARNINGS = -Wall -Wextra -Wpedantic \
-Wshadow -Wconversion -Wsign-conversion \
-Wstrict-prototypes -Wmissing-prototypes \
-Wundef -Wpointer-arith -Wcast-align \
-Wformat=2 -Wswitch-enum -Wimplicit-fallthrough

STRICT_FLAGS = -Werror

# =========================
# Flags
# =========================
EXTRA_CFLAGS ?=
CFLAGS = -std=c11 -Iinclude -MMD -MP $(BASE_WARNINGS) $(EXTRA_CFLAGS)
LDFLAGS =

DEBUG_CFLAGS  = -g -O0 -fsanitize=address,undefined -fno-omit-frame-pointer
DEBUG_LDFLAGS = -fsanitize=address,undefined

RELEASE_CFLAGS  = -O2 -fstack-protector-strong -D_FORTIFY_SOURCE=2
RELEASE_LDFLAGS = -O2 -fstack-protector-strong

# =========================
# Project structure
# =========================
TARGET = main
ARGS ?=

SRC_DIR = src
BUILD_DIR = build

SRCS = $(wildcard $(SRC_DIR)/*.c)
OBJS = $(SRCS:$(SRC_DIR)/%.c=$(BUILD_DIR)/%.o)
DEPS = $(OBJS:.o=.d)

# =========================
# Default build
# =========================
all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(OBJS) -o $(TARGET) $(LDFLAGS)

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR):
	mkdir -p $@

# Include dependency files
-include $(DEPS)

# =========================
# Run
# =========================
run: all
	./$(TARGET) $(ARGS)

# =========================
# Build modes
# =========================

debug: CFLAGS  += $(DEBUG_CFLAGS)
debug: LDFLAGS += $(DEBUG_LDFLAGS)
debug: all

release: CFLAGS  += $(RELEASE_CFLAGS)
release: LDFLAGS += $(RELEASE_LDFLAGS)
release: all

# STRICT MODE (clean rebuild + warnings = errors)
strict: CFLAGS  += $(STRICT_FLAGS) $(RELEASE_CFLAGS)
strict: LDFLAGS += $(RELEASE_LDFLAGS)
strict: clean all
	@echo "Strict build complete (warnings treated as errors)"

# =========================
# Clean
# =========================
clean:
	rm -rf $(BUILD_DIR) $(TARGET)

# =========================
# Valgrind
# =========================
valgrind: CFLAGS += -g -O0
valgrind: all
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --error-exitcode=1 ./$(TARGET)

# =========================
# Valgrind (structured XML report for the GUI)
# =========================
valgrind-xml: CFLAGS += -g -O0
valgrind-xml: all
	valgrind --xml=yes --xml-file=vg.xml --leak-check=full --show-leak-kinds=all --track-origins=yes ./$(TARGET)

# =========================
# Clang static analysis
# =========================
analyse:
	clang -Wall -Wextra -Wshadow -Wconversion -fsyntax-only -Iinclude $(SRCS) 2>&1 || true

# =========================
# cppcheck static analysis
# =========================
cppcheck:
	cppcheck --enable=all --inconclusive --template=gcc -Iinclude $(SRCS) 2>&1 || true

# =========================
# Coverage (gcov)
# =========================
coverage: CFLAGS += -fprofile-arcs -ftest-coverage -g -O0
coverage: LDFLAGS += -fprofile-arcs -ftest-coverage
coverage: clean all
	./$(TARGET)
	gcov $(SRCS)

# =========================
# Help
# =========================
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all       Build the project (default)"
	@echo "  run       Build and run the executable"
	@echo "  debug     Build with debug symbols and sanitizers (ASan, UBSan)"
	@echo "  release   Build with optimisations and hardening flags"
	@echo "  strict    Clean rebuild with -Werror and release optimisations"
	@echo "  valgrind  Build with -g and run under Valgrind memory checker"
	@echo "  valgrind-xml  Same as valgrind, writes structured XML to vg.xml"
	@echo "  analyse   Run clang static analysis (syntax check + extra warnings)"
	@echo "  cppcheck  Run cppcheck static analysis (requires cppcheck installed)"
	@echo "  coverage  Build with gcov instrumentation, run, and report line coverage"
	@echo "  clean     Remove build artefacts and executable"
	@echo "  help      Show this help message"

.PHONY: all run clean debug release strict valgrind valgrind-xml analyse cppcheck coverage help
"#;

pub const GITIGNORE: &str = "build/\nmain\n";

/// CMake equivalent of [`MAKEFILE`] — same warning flags, build-type flags, and
/// custom targets (`run`, `valgrind`, `analyse`).
pub const CMAKE_LISTS: &str = r#"cmake_minimum_required(VERSION 3.10)
project(main C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

option(STRICT "Treat warnings as errors" OFF)
option(COVERAGE "Build with gcov instrumentation" OFF)
# Single string passed verbatim to `run` (not word-split like Makefile's $(ARGS)).
set(ARGS "" CACHE STRING "Arguments forwarded to the run target")

set(BASE_WARNINGS
    -Wall -Wextra -Wpedantic
    -Wshadow -Wconversion -Wsign-conversion
    -Wstrict-prototypes -Wmissing-prototypes
    -Wundef -Wpointer-arith -Wcast-align
    -Wformat=2 -Wswitch-enum -Wimplicit-fallthrough
)

file(GLOB SRCS ${CMAKE_SOURCE_DIR}/src/*.c)

add_executable(main ${SRCS})
target_include_directories(main PRIVATE ${CMAKE_SOURCE_DIR}/include)
target_compile_options(main PRIVATE ${BASE_WARNINGS})

target_compile_options(main PRIVATE
    $<$<CONFIG:Debug>:-g -O0 -fsanitize=address,undefined -fno-omit-frame-pointer>
    $<$<CONFIG:Release>:-O2 -fstack-protector-strong -D_FORTIFY_SOURCE=2>
)
target_link_options(main PRIVATE
    $<$<CONFIG:Debug>:-fsanitize=address,undefined>
)

if(STRICT)
    target_compile_options(main PRIVATE -Werror)
endif()

if(COVERAGE)
    target_compile_options(main PRIVATE --coverage -g -O0)
    target_link_options(main PRIVATE --coverage)
endif()

# =========================
# Run
# =========================
add_custom_target(run
    COMMAND $<TARGET_FILE:main> ${ARGS}
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Valgrind
# =========================
add_custom_target(valgrind
    COMMAND valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --error-exitcode=1 $<TARGET_FILE:main>
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

add_custom_target(valgrind-xml
    COMMAND valgrind --xml=yes --xml-file=${CMAKE_SOURCE_DIR}/vg.xml --leak-check=full --show-leak-kinds=all --track-origins=yes $<TARGET_FILE:main>
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Coverage (gcov) — configure with -DCOVERAGE=ON first
# =========================
add_custom_target(coverage
    COMMAND $<TARGET_FILE:main>
    COMMAND gcov ${SRCS}
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Clang static analysis
# =========================
add_custom_target(analyse
    COMMAND clang -Wall -Wextra -Wshadow -Wconversion -fsyntax-only -I${CMAKE_SOURCE_DIR}/include ${SRCS}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

add_custom_target(cppcheck
    COMMAND cppcheck --enable=all --inconclusive --template=gcc -I${CMAKE_SOURCE_DIR}/include ${SRCS}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)
"#;

/// CMake equivalent of [`MAKEFILE_WITH_TEST`] — adds a `test` target that runs
/// the built executable, plus links `-lm`.
/// `CMakePresets.json` (requires CMake 3.19+) mirroring the `-D` flag
/// combinations [`crate::build::cmake_configure_args`] would otherwise build
/// by hand, so `cmake --preset <name>` and the GUI/CLI build runner agree.
pub const CMAKE_PRESETS: &str = r#"{
    "version": 3,
    "cmakeMinimumRequired": { "major": 3, "minor": 19, "patch": 0 },
    "configurePresets": [
        {
            "name": "release",
            "binaryDir": "${sourceDir}/build",
            "cacheVariables": { "CMAKE_BUILD_TYPE": "Release", "STRICT": "OFF", "COVERAGE": "OFF" }
        },
        {
            "name": "debug",
            "binaryDir": "${sourceDir}/build",
            "cacheVariables": { "CMAKE_BUILD_TYPE": "Debug", "STRICT": "OFF", "COVERAGE": "OFF" }
        },
        {
            "name": "strict",
            "binaryDir": "${sourceDir}/build",
            "cacheVariables": { "CMAKE_BUILD_TYPE": "Release", "STRICT": "ON", "COVERAGE": "OFF" }
        },
        {
            "name": "coverage",
            "binaryDir": "${sourceDir}/build",
            "cacheVariables": { "CMAKE_BUILD_TYPE": "Debug", "STRICT": "OFF", "COVERAGE": "ON" }
        }
    ]
}
"#;

pub const CMAKE_LISTS_WITH_TEST: &str = r#"cmake_minimum_required(VERSION 3.10)
project(main C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

option(STRICT "Treat warnings as errors" OFF)
option(COVERAGE "Build with gcov instrumentation" OFF)
# Single string passed verbatim to `run` (not word-split like Makefile's $(ARGS)).
set(ARGS "" CACHE STRING "Arguments forwarded to the run target")

set(BASE_WARNINGS
    -Wall -Wextra -Wpedantic
    -Wshadow -Wconversion -Wsign-conversion
    -Wstrict-prototypes -Wmissing-prototypes
    -Wundef -Wpointer-arith -Wcast-align
    -Wformat=2 -Wswitch-enum -Wimplicit-fallthrough
)

file(GLOB SRCS ${CMAKE_SOURCE_DIR}/src/*.c)

add_executable(main ${SRCS})
target_include_directories(main PRIVATE ${CMAKE_SOURCE_DIR}/include)
target_compile_options(main PRIVATE ${BASE_WARNINGS})
target_link_libraries(main PRIVATE m)

target_compile_options(main PRIVATE
    $<$<CONFIG:Debug>:-g -O0 -fsanitize=address,undefined -fno-omit-frame-pointer>
    $<$<CONFIG:Release>:-O2 -fstack-protector-strong -D_FORTIFY_SOURCE=2>
)
target_link_options(main PRIVATE
    $<$<CONFIG:Debug>:-fsanitize=address,undefined>
)

if(STRICT)
    target_compile_options(main PRIVATE -Werror)
endif()

if(COVERAGE)
    target_compile_options(main PRIVATE --coverage -g -O0)
    target_link_options(main PRIVATE --coverage)
endif()

# =========================
# Test
# =========================
add_custom_target(test
    COMMAND $<TARGET_FILE:main>
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Run
# =========================
add_custom_target(run
    COMMAND $<TARGET_FILE:main> ${ARGS}
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Valgrind
# =========================
add_custom_target(valgrind
    COMMAND valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --error-exitcode=1 $<TARGET_FILE:main>
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

add_custom_target(valgrind-xml
    COMMAND valgrind --xml=yes --xml-file=${CMAKE_SOURCE_DIR}/vg.xml --leak-check=full --show-leak-kinds=all --track-origins=yes $<TARGET_FILE:main>
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Coverage (gcov) — configure with -DCOVERAGE=ON first
# =========================
add_custom_target(coverage
    COMMAND $<TARGET_FILE:main>
    COMMAND gcov ${SRCS}
    DEPENDS main
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

# =========================
# Clang static analysis
# =========================
add_custom_target(analyse
    COMMAND clang -Wall -Wextra -Wshadow -Wconversion -fsyntax-only -I${CMAKE_SOURCE_DIR}/include ${SRCS}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)

add_custom_target(cppcheck
    COMMAND cppcheck --enable=all --inconclusive --template=gcc -I${CMAKE_SOURCE_DIR}/include ${SRCS}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
)
"#;

pub fn main_c(author: &str, date: &str, includes: &[&str]) -> String {
    let include_lines: String = includes
        .iter()
        .map(|m| format!("#include \"{m}.h\"\n"))
        .collect();
    format!(
        r#"/*
 * Author: {author}
 * Purpose: ...
 *
 * Date: {date}
 */

/* Header files */
#include <stdio.h>
{include_lines}
int main(void)
{{
	return 0;
}}
"#
    )
}

pub fn input_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for user input functions — reading and validating
 *          integers and yes/no responses from the user
 *
 * Date: {date}
 */

#ifndef INPUT_H
#define INPUT_H

/* Prompts user with a Y/N question, returns 1 for yes, 0 for no */
int user_Yy_Nn_choice(const char *question);
/* Prompts user and returns a valid integer */
int prompt_for_int(const char *question);
/* Prompts user and returns an integer within [min, max] */
int prompt_for_int_in_range(const char *question, int min, int max);
/* Prompts user and returns a positive integer (> 0) */
int prompt_for_positive_int(const char *question);
/* Prompts user and returns a valid double */
double prompt_for_double(const char *question);
/* Prompts user and returns a double within [min, max] */
double prompt_for_double_in_range(const char *question, double min, double max);
/* Prompts user and returns a single non-whitespace character */
char prompt_for_char(const char *question);
/* Prompts user and stores a non-empty string into buffer (max_len includes null terminator) */
void prompt_for_string(const char *question, char *buffer, int max_len);
/* Waits for the user to press enter then exits */
void press_enter_to_exit(void);
/* Discards remaining characters in the input buffer */
void clear_input_buffer(void);

#endif
"#
    )
}

pub fn input_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: User input functions — reading and validating integers and
 *          yes/no responses from the user
 *
 * Date: {date}
 */

/* Standard I/O library for printf, scanf, getchar, puts, etc. */
#include <stdio.h>
/* String library for strlen */
#include <string.h>
/* Function declarations for this module */
#include "input.h"
/* Math utilities: is_int_in_range, is_double_in_range */
#include "math.h"

/*
 * Function:
 *     user_Yy_Nn_choice
 * Input:
 *     question - a string which is the question to ask the user
 * Output:
 *     1 if user inputs yes, 0 if user inputs no
 * Algorithm:
 *     while true
 *         clear input buffer
 *         print question and (Y/N)
 *         read char from user
 *         if char is Y/y/N/n break loop
 *         else print invalid input
 *     if char is Y/y return 1
 *     else return 0
 */
int user_Yy_Nn_choice(const char *question)
{{
	char response;
	while (1) {{
		clear_input_buffer();
		printf("%s (Y/N): ", question);
		int scan_result = scanf("%c", &response);
		if (scan_result == 1 && (response == 'N' || response == 'Y' ||
		    response == 'n' || response == 'y')) {{
			break;
		}} else {{
			puts("Invalid input.");
		}}
	}}
	printf("\n");
	int result = response == 'Y' || response == 'y' ? 1 : 0;
	return result;
}}

/*
 * Function:
 *     prompt_for_int
 * Input:
 *     question - a string which is the question to ask the user
 * Output:
 *     i - the integer value input by the user
 * Algorithm:
 *     while true
 *         print question
 *         read integer from user and store in i
 *         if read was successful return i
 *         else clear input buffer and print invalid input
 */
int prompt_for_int(const char *question)
{{
	int i;
	while (1) {{
		printf("%s", question);
		int scan_result = scanf("%i", &i);
		if (scan_result == 1) {{
			return i;
		}} else {{
			clear_input_buffer();
			puts("Invalid input, please enter an integer.");
		}}
	}}
}}

/*
 * Function:
 *     prompt_for_int_in_range
 * Input:
 *     question - a string which is the question to ask the user
 *     min - the minimum value of the range
 *     max - the maximum value of the range
 * Output:
 *     i - the integer value input by the user which is between min and max
 * Algorithm:
 *     while true
 *         call prompt_for_int to get an integer from user
 *         if i is between min and max return i
 *         else print invalid input message
 */
int prompt_for_int_in_range(const char *question, int min, int max)
{{
	while (1) {{
		int i = prompt_for_int(question);
		if (is_int_in_range(i, min, max) == 1) {{
			return i;
		}} else {{
			printf("Invalid input, please input an integer between "
			       "%i and %i.\n",
			       min, max);
		}}
	}}
}}

/*
 * Function:
 *     press_enter_to_exit
 * Input:
 *     None
 * Output:
 *     None
 * Algorithm:
 *     clear input buffer
 *     print message asking user to press enter
 *     wait for user to press enter
 */
void press_enter_to_exit(void)
{{
	clear_input_buffer();
	printf("Press enter to exit the program.");
	getchar();
}}

/*
 * Function:
 *     clear_input_buffer
 * Input:
 *     None
 * Output:
 *     None
 * Algorithm:
 *     read and discard characters until newline or EOF is encountered
 */
void clear_input_buffer(void)
{{
	int c;
	while ((c = getchar()) != '\n' && c != EOF)
		;
}}

/*
 * Function:
 *     prompt_for_positive_int
 * Input:
 *     question - a string which is the question to ask the user
 * Output:
 *     i - a positive integer (> 0) input by the user
 * Algorithm:
 *     while true
 *         call prompt_for_int to get an integer from user
 *         if i is greater than 0 return i
 *         else print invalid input message
 */
int prompt_for_positive_int(const char *question)
{{
	while (1) {{
		int i = prompt_for_int(question);
		if (i > 0) {{
			return i;
		}} else {{
			puts("Invalid input, please enter a positive integer.");
		}}
	}}
}}

/*
 * Function:
 *     prompt_for_double
 * Input:
 *     question - a string which is the question to ask the user
 * Output:
 *     d - the double value input by the user
 * Algorithm:
 *     while true
 *         print question
 *         read double from user and store in d
 *         if read was successful return d
 *         else clear input buffer and print invalid input
 */
double prompt_for_double(const char *question)
{{
	double d;
	while (1) {{
		printf("%s", question);
		int scan_result = scanf("%lf", &d);
		if (scan_result == 1) {{
			clear_input_buffer();
			return d;
		}} else {{
			clear_input_buffer();
			puts("Invalid input, please enter a number.");
		}}
	}}
}}

/*
 * Function:
 *     prompt_for_double_in_range
 * Input:
 *     question - a string which is the question to ask the user
 *     min - the minimum value of the range
 *     max - the maximum value of the range
 * Output:
 *     d - the double value input by the user which is between min and max
 * Algorithm:
 *     while true
 *         call prompt_for_double to get a double from user
 *         if d is between min and max return d
 *         else print invalid input message
 */
double prompt_for_double_in_range(const char *question, double min, double max)
{{
	while (1) {{
		double d = prompt_for_double(question);
		if (is_double_in_range(d, min, max) == 1) {{
			return d;
		}} else {{
			printf("Invalid input, please input a number between "
			       "%.2f and %.2f.\n",
			       min, max);
		}}
	}}
}}

/*
 * Function:
 *     prompt_for_char
 * Input:
 *     question - a string which is the question to ask the user
 * Output:
 *     c - the character input by the user
 * Algorithm:
 *     while true
 *         print question
 *         read a non-whitespace character from user
 *         if read was successful clear buffer and return c
 *         else clear input buffer and print invalid input
 */
char prompt_for_char(const char *question)
{{
	char c;
	while (1) {{
		printf("%s", question);
		if (scanf(" %c", &c) == 1) {{
			clear_input_buffer();
			return c;
		}} else {{
			clear_input_buffer();
			puts("Invalid input.");
		}}
	}}
}}

/*
 * Function:
 *     prompt_for_string
 * Input:
 *     question - a string which is the question to ask the user
 *     buffer - char array to store the result in
 *     max_len - maximum length including null terminator
 * Output:
 *     void - result stored in buffer
 * Algorithm:
 *     while true
 *         print question
 *         read line into buffer with fgets
 *         strip trailing newline if present
 *         if buffer is non-empty return
 *         else print invalid input message
 */
void prompt_for_string(const char *question, char *buffer, int max_len)
{{
	while (1) {{
		printf("%s", question);
		if (fgets(buffer, max_len, stdin) != NULL) {{
			int len = (int)strlen(buffer);
			if (len > 0 && buffer[len - 1] == '\n') {{
				buffer[len - 1] = '\0';
			}}
			if (buffer[0] != '\0') {{
				return;
			}}
		}}
		puts("Invalid input, please enter a non-empty string.");
	}}
}}
"#
    )
}

pub fn math_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for mathematical utility functions — range checking,
 *          multiple checking, and integer division with remainder
 *
 * Date: {date}
 */

#ifndef MATH_H
#define MATH_H

/* Returns 1 if input is within [min, max], 0 otherwise */
int is_int_in_range(int input, int min, int max);
/* Returns 1 if input is a multiple of check, 0 otherwise */
int is_multiple_of(int input, int check);
/* Calculates integer division and remainder, storing results via pointers */
void divide_with_remainder(int above, int below, int *div, int *rem);
/* Returns 1 if input is within [min, max], 0 otherwise (double version) */
int is_double_in_range(double input, double min, double max);

#endif
"#
    )
}

pub fn math_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Mathematical utility functions — range checking, multiple checking,
 *          and integer division with remainder
 *
 * Date: {date}
 */

/* Function declarations for this module */
#include "math.h"

/*
 * Function:
 *     is_int_in_range
 * Input:
 *     input - the integer to check
 *     min - the minimum value of the range
 *     max - the maximum value of the range
 * Output:
 *     1 if input is within range, 0 otherwise
 * Algorithm:
 *     if input is greater than or equal to min and less than or equal to max
 *     return 1 else return 0
 */
int is_int_in_range(int input, int min, int max)
{{
	int result = input >= min && input <= max ? 1 : 0;
	return result;
}}

/*
 * Function:
 *     is_multiple_of
 * Input:
 *     input - the integer to check
 *     check - the divisor to check against
 * Output:
 *     1 if input is a multiple of check, 0 otherwise
 * Algorithm:
 *     if input modulo check equals 0 return 1
 *     else return 0
 */
int is_multiple_of(int input, int check)
{{
	int result = input % check == 0 ? 1 : 0;
	return result;
}}

/*
 * Function:
 *     divide_with_remainder
 * Input:
 *     above - the dividend (number to be divided)
 *     below - the divisor (number to divide by)
 *     div - pointer to store the quotient (result of division)
 *     rem - pointer to store the remainder (leftover after division)
 * Output:
 *     void - modifies div and rem through pointers
 * Algorithm:
 *     calculate quotient using integer division (above / below)
 *     calculate remainder using modulo operator (above % below)
 *     store results in the provided pointers
 */
void divide_with_remainder(int above, int below, int *div, int *rem)
{{
	*div = above / below;
	*rem = above % below;
}}

/*
 * Function:
 *     is_double_in_range
 * Input:
 *     input - the double to check
 *     min - the minimum value of the range
 *     max - the maximum value of the range
 * Output:
 *     1 if input is within range, 0 otherwise
 * Algorithm:
 *     if input is greater than or equal to min and less than or equal to max
 *     return 1 else return 0
 */
int is_double_in_range(double input, double min, double max)
{{
	int result = input >= min && input <= max ? 1 : 0;
	return result;
}}
"#
    )
}

pub fn display_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for display formatting functions
 *
 * Date: {date}
 */

#ifndef DISPLAY_H
#define DISPLAY_H

/* Prints a horizontal divider line */
void print_divider(void);
/* Prints a title between two dividers */
void print_header(const char *title);

#endif
"#
    )
}

pub fn display_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Display formatting functions — dividers and section headers
 *
 * Date: {date}
 */

/* Standard I/O library */
#include <stdio.h>
/* Function declarations for this module */
#include "display.h"

/*
 * Function:
 *     print_divider
 * Input:
 *     None
 * Output:
 *     None
 * Algorithm:
 *     print a line of = characters to stdout
 */
void print_divider(void)
{{
	printf("========================================\n");
}}

/*
 * Function:
 *     print_header
 * Input:
 *     title - a string to display as the header title
 * Output:
 *     None
 * Algorithm:
 *     print a divider, then the title, then another divider
 */
void print_header(const char *title)
{{
	print_divider();
	printf(" %s\n", title);
	print_divider();
}}
"#
    )
}

pub fn array_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for integer array utility functions
 *
 * Date: {date}
 */

#ifndef ARRAY_H
#define ARRAY_H

/* Fills an int array with the given value */
void array_fill_int(int arr[], int size, int value);
/* Fills a double array with the given value */
void array_fill_double(double arr[], int size, double value);
/* Fills a char array with the given value */
void array_fill_char(char arr[], int size, char value);
/* Returns the sum of all elements in arr */
int array_sum(int arr[], int size);
/* Returns the average of all elements in arr */
double array_average(int arr[], int size);
/* Returns the largest element in arr */
int array_max(int arr[], int size);
/* Returns the smallest element in arr */
int array_min(int arr[], int size);
/* Prints all elements in arr as [a, b, c, ...] */
void array_print(int arr[], int size);
/* Prompts user to fill each element of an int array individually */
void array_input_int(int arr[], int size, const char *question, int show_count);
/* Prompts user to fill each element of a double array individually */
void array_input_double(double arr[], int size, const char *question, int show_count);
/* Prompts user to fill each element of a char array individually */
void array_input_char(char arr[], int size, const char *question, int show_count);
/* Sorts arr in ascending order */
void array_sort_asc(int arr[], int size);
/* Sorts arr in descending order */
void array_sort_desc(int arr[], int size);
/* Linear search — returns index of value in arr, or -1 if not found */
int array_find(int arr[], int size, int value);
/* Binary search — returns index of value in sorted arr, or -1 if not found */
int array_find_sorted(int arr[], int size, int value);
/* Returns the sum of all elements in a double arr */
double array_sum_double(double arr[], int size);
/* Returns the average of all elements in a double arr */
double array_average_double(double arr[], int size);
/* Returns the largest element in a double arr */
double array_max_double(double arr[], int size);
/* Returns the smallest element in a double arr */
double array_min_double(double arr[], int size);
/* Prints all elements in a double arr as [a, b, c, ...] */
void array_print_double(double arr[], int size);
/* Sorts a double arr in ascending order */
void array_sort_asc_double(double arr[], int size);
/* Sorts a double arr in descending order */
void array_sort_desc_double(double arr[], int size);
/* Linear search — returns index of value in double arr, or -1 if not found */
int array_find_double(double arr[], int size, double value);
/* Binary search — returns index of value in sorted double arr, or -1 if not found */
int array_find_sorted_double(double arr[], int size, double value);

#endif
"#
    )
}

pub fn array_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Integer array utility functions — sum, average, min, max, print
 *
 * Date: {date}
 */

/* Standard I/O library */
#include <stdio.h>
/* Function declarations for this module */
#include "array.h"
/* Input functions: prompt_for_int, prompt_for_double, prompt_for_char */
#include "input.h"

/*
 * Function:
 *     array_fill_int
 * Input:
 *     arr - the int array to fill
 *     size - the number of elements in arr
 *     value - the value to assign to each element
 * Output:
 *     None
 * Algorithm:
 *     for each element set it to value
 */
void array_fill_int(int arr[], int size, int value)
{{
	for (int i = 0; i < size; i++) {{
		arr[i] = value;
	}}
}}

/*
 * Function:
 *     array_fill_double
 * Input:
 *     arr - the double array to fill
 *     size - the number of elements in arr
 *     value - the value to assign to each element
 * Output:
 *     None
 * Algorithm:
 *     for each element set it to value
 */
void array_fill_double(double arr[], int size, double value)
{{
	for (int i = 0; i < size; i++) {{
		arr[i] = value;
	}}
}}

/*
 * Function:
 *     array_fill_char
 * Input:
 *     arr - the char array to fill
 *     size - the number of elements in arr
 *     value - the value to assign to each element
 * Output:
 *     None
 * Algorithm:
 *     for each element set it to value
 */
void array_fill_char(char arr[], int size, char value)
{{
	for (int i = 0; i < size; i++) {{
		arr[i] = value;
	}}
}}

/*
 * Function:
 *     array_sum
 * Input:
 *     arr - the array of integers
 *     size - the number of elements in arr
 * Output:
 *     sum of all elements
 * Algorithm:
 *     initialise sum to 0
 *     for each element add it to sum
 *     return sum
 */
int array_sum(int arr[], int size)
{{
	int sum = 0;
	for (int i = 0; i < size; i++) {{
		sum += arr[i];
	}}
	return sum;
}}

/*
 * Function:
 *     array_average
 * Input:
 *     arr - the array of integers
 *     size - the number of elements in arr
 * Output:
 *     average of all elements as a double
 * Algorithm:
 *     return sum of arr divided by size cast to double
 */
double array_average(int arr[], int size)
{{
	return (double)array_sum(arr, size) / size;
}}

/*
 * Function:
 *     array_max
 * Input:
 *     arr - the array of integers
 *     size - the number of elements in arr
 * Output:
 *     the largest element in arr
 * Algorithm:
 *     set max to first element
 *     for each remaining element if greater than max update max
 *     return max
 */
int array_max(int arr[], int size)
{{
	int max = arr[0];
	for (int i = 1; i < size; i++) {{
		if (arr[i] > max) {{
			max = arr[i];
		}}
	}}
	return max;
}}

/*
 * Function:
 *     array_min
 * Input:
 *     arr - the array of integers
 *     size - the number of elements in arr
 * Output:
 *     the smallest element in arr
 * Algorithm:
 *     set min to first element
 *     for each remaining element if less than min update min
 *     return min
 */
int array_min(int arr[], int size)
{{
	int min = arr[0];
	for (int i = 1; i < size; i++) {{
		if (arr[i] < min) {{
			min = arr[i];
		}}
	}}
	return min;
}}

/*
 * Function:
 *     array_print
 * Input:
 *     arr - the array of integers
 *     size - the number of elements in arr
 * Output:
 *     None
 * Algorithm:
 *     print opening bracket
 *     for each element print value and comma separator if not last
 *     print closing bracket and newline
 */
void array_print(int arr[], int size)
{{
	printf("[");
	for (int i = 0; i < size; i++) {{
		printf("%d", arr[i]);
		if (i < size - 1) {{
			printf(", ");
		}}
	}}
	printf("]\n");
}}

/*
 * Function:
 *     array_input_int
 * Input:
 *     arr - the int array to fill
 *     size - the number of elements in arr
 *     question - prompt to display before each input
 *     show_count - 1 to append the element number to the prompt, 0 otherwise
 * Output:
 *     None
 * Algorithm:
 *     for each element build prompt with count if show_count is 1
 *     call prompt_for_int and store the result
 */
void array_input_int(int arr[], int size, const char *question, int show_count)
{{
	char prompt[256];
	for (int i = 0; i < size; i++) {{
		if (show_count) {{
			snprintf(prompt, sizeof(prompt), "%s %d: ", question, i + 1);
			arr[i] = prompt_for_int(prompt);
		}} else {{
			arr[i] = prompt_for_int(question);
		}}
	}}
}}

/*
 * Function:
 *     array_input_double
 * Input:
 *     arr - the double array to fill
 *     size - the number of elements in arr
 *     question - prompt to display before each input
 *     show_count - 1 to append the element number to the prompt, 0 otherwise
 * Output:
 *     None
 * Algorithm:
 *     for each element build prompt with count if show_count is 1
 *     call prompt_for_double and store the result
 */
void array_input_double(double arr[], int size, const char *question, int show_count)
{{
	char prompt[256];
	for (int i = 0; i < size; i++) {{
		if (show_count) {{
			snprintf(prompt, sizeof(prompt), "%s %d: ", question, i + 1);
			arr[i] = prompt_for_double(prompt);
		}} else {{
			arr[i] = prompt_for_double(question);
		}}
	}}
}}

/*
 * Function:
 *     array_input_char
 * Input:
 *     arr - the char array to fill
 *     size - the number of elements in arr
 *     question - prompt to display before each input
 *     show_count - 1 to append the element number to the prompt, 0 otherwise
 * Output:
 *     None
 * Algorithm:
 *     for each element build prompt with count if show_count is 1
 *     call prompt_for_char and store the result
 */
void array_input_char(char arr[], int size, const char *question, int show_count)
{{
	char prompt[256];
	for (int i = 0; i < size; i++) {{
		if (show_count) {{
			snprintf(prompt, sizeof(prompt), "%s %d: ", question, i + 1);
			arr[i] = prompt_for_char(prompt);
		}} else {{
			arr[i] = prompt_for_char(question);
		}}
	}}
}}

/*
 * Function:
 *     array_sort_asc
 * Input:
 *     arr - the array to sort
 *     size - the number of elements in arr
 * Output:
 *     None
 * Algorithm:
 *     bubble sort: repeatedly step through arr comparing adjacent elements
 *     swap them if they are in the wrong order until no swaps are needed
 */
void array_sort_asc(int arr[], int size)
{{
	for (int i = 0; i < size - 1; i++) {{
		for (int j = 0; j < size - i - 1; j++) {{
			if (arr[j] > arr[j + 1]) {{
				int temp = arr[j];
				arr[j] = arr[j + 1];
				arr[j + 1] = temp;
			}}
		}}
	}}
}}

/*
 * Function:
 *     array_sort_desc
 * Input:
 *     arr - the array to sort
 *     size - the number of elements in arr
 * Output:
 *     None
 * Algorithm:
 *     bubble sort: repeatedly step through arr comparing adjacent elements
 *     swap them if they are in the wrong order until no swaps are needed
 */
void array_sort_desc(int arr[], int size)
{{
	for (int i = 0; i < size - 1; i++) {{
		for (int j = 0; j < size - i - 1; j++) {{
			if (arr[j] < arr[j + 1]) {{
				int temp = arr[j];
				arr[j] = arr[j + 1];
				arr[j + 1] = temp;
			}}
		}}
	}}
}}

/*
 * Function:
 *     array_find
 * Input:
 *     arr - the array to search
 *     size - the number of elements in arr
 *     value - the value to search for
 * Output:
 *     index of value if found, -1 otherwise
 * Algorithm:
 *     for each element if it equals value return its index
 *     if loop completes without finding value return -1
 */
int array_find(int arr[], int size, int value)
{{
	for (int i = 0; i < size; i++) {{
		if (arr[i] == value) {{
			return i;
		}}
	}}
	return -1;
}}

/*
 * Function:
 *     array_find_sorted
 * Input:
 *     arr - a sorted array to search
 *     size - the number of elements in arr
 *     value - the value to search for
 * Output:
 *     index of value if found, -1 otherwise
 * Algorithm:
 *     binary search: set low and high bounds
 *     while low <= high calculate mid
 *         if arr[mid] equals value return mid
 *         if arr[mid] < value search upper half
 *         else search lower half
 *     return -1 if not found
 */
int array_find_sorted(int arr[], int size, int value)
{{
	int low = 0;
	int high = size - 1;
	while (low <= high) {{
		int mid = low + (high - low) / 2;
		if (arr[mid] == value) {{
			return mid;
		}} else if (arr[mid] < value) {{
			low = mid + 1;
		}} else {{
			high = mid - 1;
		}}
	}}
	return -1;
}}

/*
 * Function:
 *     array_sum_double
 * Input:
 *     arr - the array of doubles
 *     size - the number of elements in arr
 * Output:
 *     sum of all elements
 * Algorithm:
 *     initialise sum to 0
 *     for each element add it to sum
 *     return sum
 */
double array_sum_double(double arr[], int size)
{{
	double sum = 0.0;
	for (int i = 0; i < size; i++) {{
		sum += arr[i];
	}}
	return sum;
}}

/*
 * Function:
 *     array_average_double
 * Input:
 *     arr - the array of doubles
 *     size - the number of elements in arr
 * Output:
 *     average of all elements as a double
 * Algorithm:
 *     return sum of arr divided by size
 */
double array_average_double(double arr[], int size)
{{
	return array_sum_double(arr, size) / size;
}}

/*
 * Function:
 *     array_max_double
 * Input:
 *     arr - the array of doubles
 *     size - the number of elements in arr
 * Output:
 *     the largest element in arr
 * Algorithm:
 *     set max to first element
 *     for each remaining element if greater than max update max
 *     return max
 */
double array_max_double(double arr[], int size)
{{
	double max = arr[0];
	for (int i = 1; i < size; i++) {{
		if (arr[i] > max) {{
			max = arr[i];
		}}
	}}
	return max;
}}

/*
 * Function:
 *     array_min_double
 * Input:
 *     arr - the array of doubles
 *     size - the number of elements in arr
 * Output:
 *     the smallest element in arr
 * Algorithm:
 *     set min to first element
 *     for each remaining element if less than min update min
 *     return min
 */
double array_min_double(double arr[], int size)
{{
	double min = arr[0];
	for (int i = 1; i < size; i++) {{
		if (arr[i] < min) {{
			min = arr[i];
		}}
	}}
	return min;
}}

/*
 * Function:
 *     array_print_double
 * Input:
 *     arr - the array of doubles
 *     size - the number of elements in arr
 * Output:
 *     None
 * Algorithm:
 *     print opening bracket
 *     for each element print value and comma separator if not last
 *     print closing bracket and newline
 */
void array_print_double(double arr[], int size)
{{
	printf("[");
	for (int i = 0; i < size; i++) {{
		printf("%g", arr[i]);
		if (i < size - 1) {{
			printf(", ");
		}}
	}}
	printf("]\n");
}}

/*
 * Function:
 *     array_sort_asc_double
 * Input:
 *     arr - the double array to sort
 *     size - the number of elements in arr
 * Output:
 *     None
 * Algorithm:
 *     bubble sort: repeatedly step through arr comparing adjacent elements
 *     swap them if they are in the wrong order until no swaps are needed
 */
void array_sort_asc_double(double arr[], int size)
{{
	for (int i = 0; i < size - 1; i++) {{
		for (int j = 0; j < size - i - 1; j++) {{
			if (arr[j] > arr[j + 1]) {{
				double temp = arr[j];
				arr[j] = arr[j + 1];
				arr[j + 1] = temp;
			}}
		}}
	}}
}}

/*
 * Function:
 *     array_sort_desc_double
 * Input:
 *     arr - the double array to sort
 *     size - the number of elements in arr
 * Output:
 *     None
 * Algorithm:
 *     bubble sort: repeatedly step through arr comparing adjacent elements
 *     swap them if they are in the wrong order until no swaps are needed
 */
void array_sort_desc_double(double arr[], int size)
{{
	for (int i = 0; i < size - 1; i++) {{
		for (int j = 0; j < size - i - 1; j++) {{
			if (arr[j] < arr[j + 1]) {{
				double temp = arr[j];
				arr[j] = arr[j + 1];
				arr[j + 1] = temp;
			}}
		}}
	}}
}}

/*
 * Function:
 *     array_find_double
 * Input:
 *     arr - the double array to search
 *     size - the number of elements in arr
 *     value - the value to search for
 * Output:
 *     index of value if found, -1 otherwise
 * Algorithm:
 *     for each element if it equals value return its index
 *     if loop completes without finding value return -1
 */
int array_find_double(double arr[], int size, double value)
{{
	for (int i = 0; i < size; i++) {{
		if (arr[i] == value) {{
			return i;
		}}
	}}
	return -1;
}}

/*
 * Function:
 *     array_find_sorted_double
 * Input:
 *     arr - a sorted double array to search
 *     size - the number of elements in arr
 *     value - the value to search for
 * Output:
 *     index of value if found, -1 otherwise
 * Algorithm:
 *     binary search: set low and high bounds
 *     while low <= high calculate mid
 *         if arr[mid] equals value return mid
 *         if arr[mid] < value search upper half
 *         else search lower half
 *     return -1 if not found
 */
int array_find_sorted_double(double arr[], int size, double value)
{{
	int low = 0;
	int high = size - 1;
	while (low <= high) {{
		int mid = low + (high - low) / 2;
		if (arr[mid] == value) {{
			return mid;
		}} else if (arr[mid] < value) {{
			low = mid + 1;
		}} else {{
			high = mid - 1;
		}}
	}}
	return -1;
}}
"#
    )
}

// ── strings module ────────────────────────────────────────────────────────────

pub fn strings_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for string utility functions — length, copy, case, reverse
 *
 * Date: {date}
 */

#ifndef STRINGS_H
#define STRINGS_H

/* Returns the length of str (not counting null terminator) */
int str_length(const char *str);
/* Copies src into dst safely; dst_size is total buffer size including null terminator */
void str_copy_safe(char *dst, const char *src, int dst_size);
/* Converts all characters in str to uppercase in-place */
void str_to_upper(char *str);
/* Converts all characters in str to lowercase in-place */
void str_to_lower(char *str);
/* Reverses str in-place */
void str_reverse(char *str);
/* Returns 0 if a == b, negative if a < b, positive if a > b */
int str_compare(const char *a, const char *b);
/* Returns 1 if str contains the character c, 0 otherwise */
int str_contains_char(const char *str, char c);
/* Returns 1 if str starts with prefix, 0 otherwise */
int str_starts_with(const char *str, const char *prefix);
/* Strips leading and trailing whitespace from str in-place */
void str_trim(char *str);

#endif
"#
    )
}

pub fn strings_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: String utility functions — length, copy, case conversion, reverse
 *
 * Date: {date}
 */

/* Standard I/O for printf */
#include <stdio.h>
/* ctype for toupper, tolower, isspace */
#include <ctype.h>
/* Function declarations for this module */
#include "strings.h"

/*
 * Function:
 *     str_length
 * Input:
 *     str - the string to measure
 * Output:
 *     number of characters before the null terminator
 * Algorithm:
 *     walk pointer until null terminator, count steps
 */
int str_length(const char *str)
{{
	int len = 0;
	while (str[len] != '\0') {{
		len++;
	}}
	return len;
}}

/*
 * Function:
 *     str_copy_safe
 * Input:
 *     dst      - destination buffer
 *     src      - source string
 *     dst_size - total size of dst including null terminator
 * Output:
 *     None — result written into dst, always null-terminated
 * Algorithm:
 *     copy at most dst_size - 1 characters, then append null terminator
 */
void str_copy_safe(char *dst, const char *src, int dst_size)
{{
	int i;
	for (i = 0; i < dst_size - 1 && src[i] != '\0'; i++) {{
		dst[i] = src[i];
	}}
	dst[i] = '\0';
}}

/*
 * Function:
 *     str_to_upper
 * Input:
 *     str - string to convert in-place
 * Output:
 *     None — str modified directly
 * Algorithm:
 *     for each character apply toupper
 */
void str_to_upper(char *str)
{{
	for (int i = 0; str[i] != '\0'; i++) {{
		str[i] = (char)toupper((unsigned char)str[i]);
	}}
}}

/*
 * Function:
 *     str_to_lower
 * Input:
 *     str - string to convert in-place
 * Output:
 *     None — str modified directly
 * Algorithm:
 *     for each character apply tolower
 */
void str_to_lower(char *str)
{{
	for (int i = 0; str[i] != '\0'; i++) {{
		str[i] = (char)tolower((unsigned char)str[i]);
	}}
}}

/*
 * Function:
 *     str_reverse
 * Input:
 *     str - string to reverse in-place
 * Output:
 *     None — str modified directly
 * Algorithm:
 *     swap characters from both ends walking inward
 */
void str_reverse(char *str)
{{
	int left = 0;
	int right = str_length(str) - 1;
	while (left < right) {{
		char tmp = str[left];
		str[left]  = str[right];
		str[right] = tmp;
		left++;
		right--;
	}}
}}

/*
 * Function:
 *     str_compare
 * Input:
 *     a, b - strings to compare
 * Output:
 *     0 if equal, negative if a < b, positive if a > b
 * Algorithm:
 *     compare characters until difference or null terminator
 */
int str_compare(const char *a, const char *b)
{{
	while (*a != '\0' && *a == *b) {{
		a++;
		b++;
	}}
	return (unsigned char)*a - (unsigned char)*b;
}}

/*
 * Function:
 *     str_contains_char
 * Input:
 *     str - the string to search
 *     c   - the character to look for
 * Output:
 *     1 if c is found in str, 0 otherwise
 * Algorithm:
 *     scan each character; return 1 on match
 */
int str_contains_char(const char *str, char c)
{{
	for (int i = 0; str[i] != '\0'; i++) {{
		if (str[i] == c) {{
			return 1;
		}}
	}}
	return 0;
}}

/*
 * Function:
 *     str_starts_with
 * Input:
 *     str    - the string to test
 *     prefix - the prefix to look for
 * Output:
 *     1 if str begins with prefix, 0 otherwise
 * Algorithm:
 *     compare prefix characters against str; fail on first mismatch
 */
int str_starts_with(const char *str, const char *prefix)
{{
	for (int i = 0; prefix[i] != '\0'; i++) {{
		if (str[i] != prefix[i]) {{
			return 0;
		}}
	}}
	return 1;
}}

/*
 * Function:
 *     str_trim
 * Input:
 *     str - string to trim in-place
 * Output:
 *     None — leading and trailing whitespace removed
 * Algorithm:
 *     find first and last non-whitespace; shift and null-terminate
 */
void str_trim(char *str)
{{
	int start = 0;
	while (str[start] != '\0' && isspace((unsigned char)str[start])) {{
		start++;
	}}
	int end = str_length(str) - 1;
	while (end >= start && isspace((unsigned char)str[end])) {{
		end--;
	}}
	int len = end - start + 1;
	for (int i = 0; i < len; i++) {{
		str[i] = str[start + i];
	}}
	str[len] = '\0';
}}
"#
    )
}

// ── linked_list module ────────────────────────────────────────────────────────

pub fn linked_list_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for singly linked list with integer data
 *
 * Date: {date}
 */

#ifndef LINKED_LIST_H
#define LINKED_LIST_H

/* Node of the linked list */
typedef struct Node {{
	int data;
	struct Node *next;
}} Node;

/* Inserts a new node with data at the front of the list; updates *head */
void list_insert(Node **head, int data);
/* Removes the first node with matching data; returns 1 if removed, 0 if not found */
int list_delete_value(Node **head, int data);
/* Prints all elements as [a -> b -> c -> NULL] */
void list_print(Node *head);
/* Frees all nodes in the list and sets *head to NULL */
void list_free(Node **head);
/* Returns pointer to first node with data, or NULL if not found */
Node *list_find(Node *head, int data);
/* Returns the number of nodes in the list */
int list_length(Node *head);

#endif
"#
    )
}

pub fn linked_list_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Singly linked list — insert, delete, find, print, free
 *
 * Date: {date}
 */

/* Standard I/O for printf */
#include <stdio.h>
/* malloc, free */
#include <stdlib.h>
/* Function declarations for this module */
#include "linked_list.h"

/*
 * Function:
 *     list_insert
 * Input:
 *     head - pointer to the head pointer of the list
 *     data - integer to store in the new node
 * Output:
 *     None — new node prepended to the list
 * Algorithm:
 *     allocate new node, set data and next, update *head
 */
void list_insert(Node **head, int data)
{{
	Node *node = malloc(sizeof(Node));
	if (node == NULL) {{
		fprintf(stderr, "list_insert: malloc failed\n");
		return;
	}}
	node->data = data;
	node->next = *head;
	*head = node;
}}

/*
 * Function:
 *     list_delete_value
 * Input:
 *     head - pointer to the head pointer of the list
 *     data - value to remove
 * Output:
 *     1 if node was found and removed, 0 otherwise
 * Algorithm:
 *     traverse with prev/curr pointers; unlink matching node
 */
int list_delete_value(Node **head, int data)
{{
	Node *prev = NULL;
	Node *curr = *head;
	while (curr != NULL) {{
		if (curr->data == data) {{
			if (prev == NULL) {{
				*head = curr->next;
			}} else {{
				prev->next = curr->next;
			}}
			free(curr);
			curr = NULL;
			return 1;
		}}
		prev = curr;
		curr = curr->next;
	}}
	return 0;
}}

/*
 * Function:
 *     list_print
 * Input:
 *     head - first node of the list (may be NULL)
 * Output:
 *     None — prints list in format [a -> b -> c -> NULL]
 * Algorithm:
 *     traverse nodes printing each data value
 */
void list_print(Node *head)
{{
	printf("[");
	Node *curr = head;
	while (curr != NULL) {{
		printf("%d", curr->data);
		if (curr->next != NULL) {{
			printf(" -> ");
		}}
		curr = curr->next;
	}}
	printf(" -> NULL]\n");
}}

/*
 * Function:
 *     list_free
 * Input:
 *     head - pointer to the head pointer of the list
 * Output:
 *     None — all nodes freed, *head set to NULL
 * Algorithm:
 *     traverse freeing each node; advance before freeing
 */
void list_free(Node **head)
{{
	Node *curr = *head;
	while (curr != NULL) {{
		Node *next = curr->next;
		free(curr);
		curr = next;
	}}
	*head = NULL;
}}

/*
 * Function:
 *     list_find
 * Input:
 *     head - first node of the list
 *     data - value to search for
 * Output:
 *     pointer to first matching node, or NULL if not found
 * Algorithm:
 *     linear scan comparing each node's data
 */
Node *list_find(Node *head, int data)
{{
	Node *curr = head;
	while (curr != NULL) {{
		if (curr->data == data) {{
			return curr;
		}}
		curr = curr->next;
	}}
	return NULL;
}}

/*
 * Function:
 *     list_length
 * Input:
 *     head - first node of the list (may be NULL)
 * Output:
 *     number of nodes in the list
 * Algorithm:
 *     count each node traversed
 */
int list_length(Node *head)
{{
	int count = 0;
	Node *curr = head;
	while (curr != NULL) {{
		count++;
		curr = curr->next;
	}}
	return count;
}}
"#
    )
}

// ── files module ──────────────────────────────────────────────────────────────

pub fn files_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Declarations for safe file I/O helper functions
 *
 * Date: {date}
 */

#ifndef FILES_H
#define FILES_H

/* Opens a file safely; prints error and returns NULL on failure */
FILE *file_open_safe(const char *path, const char *mode);
/* Counts the number of newline-terminated lines in an open file; rewinds when done */
int file_count_lines(FILE *fp);
/* Reads the next line from fp into buf (max_len includes null terminator); strips newline */
int file_read_line_into(FILE *fp, char *buf, int max_len);
/* Writes a line to fp followed by a newline; returns 1 on success, 0 on error */
int file_write_line(FILE *fp, const char *line);

#endif
"#
    )
}

pub fn files_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Safe file I/O helpers — open, line count, read line, write line
 *
 * Date: {date}
 */

/* Standard I/O for FILE, fopen, fgets, fprintf, rewind, etc. */
#include <stdio.h>
/* String functions for strlen */
#include <string.h>
/* Function declarations for this module */
#include "files.h"

/*
 * Function:
 *     file_open_safe
 * Input:
 *     path - path to the file
 *     mode - open mode string (e.g. "r", "w", "a")
 * Output:
 *     FILE pointer on success, NULL on failure (error printed to stderr)
 * Algorithm:
 *     call fopen; if NULL print error and return NULL
 */
FILE *file_open_safe(const char *path, const char *mode)
{{
	FILE *fp = fopen(path, mode);
	if (fp == NULL) {{
		fprintf(stderr, "Error: cannot open '%s' in mode '%s'\n", path, mode);
	}}
	return fp;
}}

/*
 * Function:
 *     file_count_lines
 * Input:
 *     fp - open file pointer (must not be NULL)
 * Output:
 *     number of newline-terminated lines; file position rewound to start
 * Algorithm:
 *     read characters counting newlines, then rewind
 */
int file_count_lines(FILE *fp)
{{
	int count = 0;
	int c;
	while ((c = fgetc(fp)) != EOF) {{
		if (c == '\n') {{
			count++;
		}}
	}}
	rewind(fp);
	return count;
}}

/*
 * Function:
 *     file_read_line_into
 * Input:
 *     fp      - open file pointer
 *     buf     - destination buffer
 *     max_len - buffer size including null terminator
 * Output:
 *     1 if a line was read, 0 on EOF or error
 *     Trailing newline stripped from buf
 * Algorithm:
 *     use fgets; strip trailing newline if present
 */
int file_read_line_into(FILE *fp, char *buf, int max_len)
{{
	if (fgets(buf, max_len, fp) == NULL) {{
		return 0;
	}}
	int len = (int)strlen(buf);
	if (len > 0 && buf[len - 1] == '\n') {{
		buf[len - 1] = '\0';
	}}
	return 1;
}}

/*
 * Function:
 *     file_write_line
 * Input:
 *     fp   - open file pointer (write mode)
 *     line - string to write
 * Output:
 *     1 on success, 0 on write error
 * Algorithm:
 *     fprintf with newline; check return value
 */
int file_write_line(FILE *fp, const char *line)
{{
	return fprintf(fp, "%s\n", line) > 0 ? 1 : 0;
}}
"#
    )
}

// ── test_utils module ─────────────────────────────────────────────────────────

pub fn test_utils_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Minimal unit test harness — assert macros, test runner, summary
 *
 * Date: {date}
 */

#ifndef TEST_UTILS_H
#define TEST_UTILS_H

/* Assert that two integers are equal; prints PASS/FAIL with values */
void assert_int_eq(int expected, int actual, const char *label);
/* Assert that two strings are equal; prints PASS/FAIL */
void assert_str_eq(const char *expected, const char *actual, const char *label);
/* Assert that pointer is NULL */
void assert_null(const void *ptr, const char *label);
/* Assert that pointer is not NULL */
void assert_not_null(const void *ptr, const char *label);
/* Assert that two doubles are equal within tolerance */
void assert_double_near(double expected, double actual, double tolerance, const char *label);
/* Run a named test function; updates pass/fail counters */
void test_run(const char *name, void (*test_fn)(void));
/* Print final summary: passed/total */
void print_test_summary(void);

#endif
"#
    )
}

pub fn test_utils_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Minimal unit test harness implementation
 *
 * Date: {date}
 */

/* printf, fprintf */
#include <stdio.h>
/* strcmp */
#include <string.h>
/* fabs */
#include <math.h>
/* Function declarations for this module */
#include "test_utils.h"

static int g_passed = 0;
static int g_failed = 0;

/*
 * Function:
 *     assert_int_eq
 * Input:
 *     expected - the expected integer value
 *     actual   - the actual integer value
 *     label    - description of the assertion
 * Output:
 *     None — prints PASS or FAIL to stdout; updates counters
 * Algorithm:
 *     compare expected and actual; print result
 */
void assert_int_eq(int expected, int actual, const char *label)
{{
	if (expected == actual) {{
		printf("  [PASS] %s\n", label);
		g_passed++;
	}} else {{
		printf("  [FAIL] %s — expected %d, got %d\n", label, expected, actual);
		g_failed++;
	}}
}}

/*
 * Function:
 *     assert_str_eq
 * Input:
 *     expected - expected string
 *     actual   - actual string
 *     label    - description of the assertion
 * Output:
 *     None — prints PASS or FAIL
 * Algorithm:
 *     strcmp; print match or mismatch
 */
void assert_str_eq(const char *expected, const char *actual, const char *label)
{{
	if (strcmp(expected, actual) == 0) {{
		printf("  [PASS] %s\n", label);
		g_passed++;
	}} else {{
		printf("  [FAIL] %s — expected \"%s\", got \"%s\"\n", label, expected, actual);
		g_failed++;
	}}
}}

/*
 * Function:
 *     assert_null
 * Input:
 *     ptr   - pointer to check
 *     label - description
 * Output:
 *     None — prints PASS if ptr is NULL
 */
void assert_null(const void *ptr, const char *label)
{{
	if (ptr == NULL) {{
		printf("  [PASS] %s\n", label);
		g_passed++;
	}} else {{
		printf("  [FAIL] %s — expected NULL, got non-NULL\n", label);
		g_failed++;
	}}
}}

/*
 * Function:
 *     assert_not_null
 * Input:
 *     ptr   - pointer to check
 *     label - description
 * Output:
 *     None — prints PASS if ptr is non-NULL
 */
void assert_not_null(const void *ptr, const char *label)
{{
	if (ptr != NULL) {{
		printf("  [PASS] %s\n", label);
		g_passed++;
	}} else {{
		printf("  [FAIL] %s — expected non-NULL, got NULL\n", label);
		g_failed++;
	}}
}}

/*
 * Function:
 *     assert_double_near
 * Input:
 *     expected  - expected double value
 *     actual    - actual double value
 *     tolerance - acceptable difference (e.g. 0.0001)
 *     label     - description
 * Output:
 *     None — prints PASS if |expected - actual| <= tolerance
 */
void assert_double_near(double expected, double actual, double tolerance, const char *label)
{{
	if (fabs(expected - actual) <= tolerance) {{
		printf("  [PASS] %s\n", label);
		g_passed++;
	}} else {{
		printf("  [FAIL] %s — expected %.6f, got %.6f (tolerance %.6f)\n",
		       label, expected, actual, tolerance);
		g_failed++;
	}}
}}

/*
 * Function:
 *     test_run
 * Input:
 *     name    - human-readable test name printed as header
 *     test_fn - function pointer to the test body (takes no args, returns void)
 * Output:
 *     None — runs test_fn and prints header
 */
void test_run(const char *name, void (*test_fn)(void))
{{
	printf("[TEST] %s\n", name);
	test_fn();
}}

/*
 * Function:
 *     print_test_summary
 * Input:
 *     None
 * Output:
 *     None — prints final pass/fail summary and exits with 1 if any failures
 */
void print_test_summary(void)
{{
	int total = g_passed + g_failed;
	printf("\n========================================\n");
	printf("Results: %d / %d passed", g_passed, total);
	if (g_failed == 0) {{
		printf(" — ALL PASS\n");
	}} else {{
		printf(" — %d FAILED\n", g_failed);
	}}
	printf("========================================\n");
}}
"#
    )
}

/// Header for the Unity-style test harness — a small, self-contained
/// alternative to [`test_utils_h`] using assertion macro names familiar from
/// the ThrowTheSwitch/Unity testing framework (`TEST_ASSERT_*`, `RUN_TEST`,
/// `UNITY_BEGIN`/`UNITY_END`). This is an original minimal implementation
/// providing that API surface, not a vendored copy of the Unity library.
pub fn unity_h(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Minimal Unity-style unit test harness (assert macros + runner)
 *
 * Date: {date}
 */

#ifndef UNITY_H
#define UNITY_H

void unity_begin(void);
int unity_end(void);
void unity_run_test(const char *name, void (*test_fn)(void));
void unity_assert_true(int condition, const char *expr, const char *file, int line);
void unity_assert_equal_int(int expected, int actual, const char *expr, const char *file, int line);
void unity_assert_equal_string(const char *expected, const char *actual, const char *expr, const char *file, int line);

#define UNITY_BEGIN() unity_begin()
#define UNITY_END() unity_end()
#define RUN_TEST(fn) unity_run_test(#fn, fn)
#define TEST_ASSERT_TRUE(cond) unity_assert_true((cond), #cond, __FILE__, __LINE__)
#define TEST_ASSERT_FALSE(cond) unity_assert_true(!(cond), "!(" #cond ")", __FILE__, __LINE__)
#define TEST_ASSERT_EQUAL_INT(expected, actual) unity_assert_equal_int((expected), (actual), #actual, __FILE__, __LINE__)
#define TEST_ASSERT_EQUAL_STRING(expected, actual) unity_assert_equal_string((expected), (actual), #actual, __FILE__, __LINE__)

#endif
"#
    )
}

/// Implementation for [`unity_h`].
pub fn unity_c(author: &str, date: &str) -> String {
    format!(
        r#"/*
 * Author: {author}
 * Purpose: Minimal Unity-style unit test harness implementation
 *
 * Date: {date}
 */

#include <stdio.h>
#include <string.h>
#include "unity.h"

static int g_passed = 0;
static int g_failed = 0;

void unity_begin(void)
{{
	g_passed = 0;
	g_failed = 0;
}}

int unity_end(void)
{{
	int total = g_passed + g_failed;
	printf("\n========================================\n");
	printf("Results: %d / %d passed", g_passed, total);
	if (g_failed == 0) {{
		printf(" — ALL PASS\n");
	}} else {{
		printf(" — %d FAILED\n", g_failed);
	}}
	printf("========================================\n");
	return g_failed == 0 ? 0 : 1;
}}

void unity_run_test(const char *name, void (*test_fn)(void))
{{
	printf("RUN  %s\n", name);
	test_fn();
}}

void unity_assert_true(int condition, const char *expr, const char *file, int line)
{{
	if (condition) {{
		g_passed++;
	}} else {{
		printf("  [FAIL] %s:%d — TEST_ASSERT_TRUE(%s)\n", file, line, expr);
		g_failed++;
	}}
}}

void unity_assert_equal_int(int expected, int actual, const char *expr, const char *file, int line)
{{
	if (expected == actual) {{
		g_passed++;
	}} else {{
		printf("  [FAIL] %s:%d — %s: expected %d, got %d\n", file, line, expr, expected, actual);
		g_failed++;
	}}
}}

void unity_assert_equal_string(const char *expected, const char *actual, const char *expr, const char *file, int line)
{{
	if (strcmp(expected, actual) == 0) {{
		g_passed++;
	}} else {{
		printf("  [FAIL] %s:%d — %s: expected \"%s\", got \"%s\"\n", file, line, expr, expected, actual);
		g_failed++;
	}}
}}
"#
    )
}

/// Makefile with an added `test` target that runs ./main and checks exit code.
pub const MAKEFILE_WITH_TEST: &str = r#"CC = gcc

# =========================
# Warning levels
# =========================
BASE_WARNINGS = -Wall -Wextra -Wpedantic \
-Wshadow -Wconversion -Wsign-conversion \
-Wstrict-prototypes -Wmissing-prototypes \
-Wundef -Wpointer-arith -Wcast-align \
-Wformat=2 -Wswitch-enum -Wimplicit-fallthrough

STRICT_FLAGS = -Werror

# =========================
# Flags
# =========================
EXTRA_CFLAGS ?=
CFLAGS = -std=c11 -Iinclude -MMD -MP $(BASE_WARNINGS) $(EXTRA_CFLAGS)
LDFLAGS = -lm

DEBUG_CFLAGS  = -g -O0 -fsanitize=address,undefined -fno-omit-frame-pointer
DEBUG_LDFLAGS = -fsanitize=address,undefined

RELEASE_CFLAGS  = -O2 -fstack-protector-strong -D_FORTIFY_SOURCE=2
RELEASE_LDFLAGS = -O2 -fstack-protector-strong

# =========================
# Project structure
# =========================
TARGET = main
ARGS ?=

SRC_DIR = src
BUILD_DIR = build

SRCS = $(wildcard $(SRC_DIR)/*.c)
OBJS = $(SRCS:$(SRC_DIR)/%.c=$(BUILD_DIR)/%.o)
DEPS = $(OBJS:.o=.d)

# =========================
# Default build
# =========================
all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(OBJS) -o $(TARGET) $(LDFLAGS)

$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR):
	mkdir -p $@

# Include dependency files
-include $(DEPS)

# =========================
# Test
# =========================
test: all
	./$(TARGET)

# =========================
# Run
# =========================
run: all
	./$(TARGET) $(ARGS)

# =========================
# Build modes
# =========================

debug: CFLAGS  += $(DEBUG_CFLAGS)
debug: LDFLAGS += $(DEBUG_LDFLAGS)
debug: all

release: CFLAGS  += $(RELEASE_CFLAGS)
release: LDFLAGS += $(RELEASE_LDFLAGS)
release: all

# STRICT MODE (clean rebuild + warnings = errors)
strict: CFLAGS  += $(STRICT_FLAGS) $(RELEASE_CFLAGS)
strict: LDFLAGS += $(RELEASE_LDFLAGS)
strict: clean all
	@echo "Strict build complete (warnings treated as errors)"

# =========================
# Clean
# =========================
clean:
	rm -rf $(BUILD_DIR) $(TARGET)

# =========================
# Valgrind
# =========================
valgrind: CFLAGS += -g -O0
valgrind: all
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --error-exitcode=1 ./$(TARGET)

# =========================
# Valgrind (structured XML report for the GUI)
# =========================
valgrind-xml: CFLAGS += -g -O0
valgrind-xml: all
	valgrind --xml=yes --xml-file=vg.xml --leak-check=full --show-leak-kinds=all --track-origins=yes ./$(TARGET)

# =========================
# Clang static analysis
# =========================
analyse:
	clang -Wall -Wextra -Wshadow -Wconversion -fsyntax-only -Iinclude $(SRCS) 2>&1 || true

# =========================
# cppcheck static analysis
# =========================
cppcheck:
	cppcheck --enable=all --inconclusive --template=gcc -Iinclude $(SRCS) 2>&1 || true

# =========================
# Coverage (gcov)
# =========================
coverage: CFLAGS += -fprofile-arcs -ftest-coverage -g -O0
coverage: LDFLAGS += -fprofile-arcs -ftest-coverage
coverage: clean all
	./$(TARGET)
	gcov $(SRCS)

# =========================
# Help
# =========================
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all       Build the project (default)"
	@echo "  test      Build and run tests"
	@echo "  run       Build and run the executable"
	@echo "  debug     Build with debug symbols and sanitizers (ASan, UBSan)"
	@echo "  release   Build with optimisations and hardening flags"
	@echo "  strict    Clean rebuild with -Werror and release optimisations"
	@echo "  valgrind  Build with -g and run under Valgrind memory checker"
	@echo "  valgrind-xml  Same as valgrind, writes structured XML to vg.xml"
	@echo "  analyse   Run clang static analysis (syntax check + extra warnings)"
	@echo "  cppcheck  Run cppcheck static analysis (requires cppcheck installed)"
	@echo "  coverage  Build with gcov instrumentation, run, and report line coverage"
	@echo "  clean     Remove build artefacts and executable"
	@echo "  help      Show this help message"

.PHONY: all test run clean debug release strict valgrind valgrind-xml analyse cppcheck coverage help
"#;
