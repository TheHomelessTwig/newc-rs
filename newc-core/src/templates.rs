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
CFLAGS = -std=c11 -Iinclude -MMD -MP $(BASE_WARNINGS)
LDFLAGS =

DEBUG_CFLAGS  = -g -O0 -fsanitize=address,undefined -fno-omit-frame-pointer
DEBUG_LDFLAGS = -fsanitize=address,undefined

RELEASE_CFLAGS  = -O2 -fstack-protector-strong -D_FORTIFY_SOURCE=2
RELEASE_LDFLAGS = -O2 -fstack-protector-strong

# =========================
# Project structure
# =========================
TARGET = main

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
	./$(TARGET)

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
	@echo "  clean     Remove build artefacts and executable"
	@echo "  help      Show this help message"

.PHONY: all run clean debug release strict help
"#;

pub const GITIGNORE: &str = "build/\nmain\n";

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
