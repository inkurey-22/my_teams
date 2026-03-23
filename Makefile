##
## EPITECH PROJECT, 2026
## cpp_template
## File description:
## Makefile
##

BUILD_DIR = build
TARGET = cpp_template

.PHONY: all clean fclean re debug

all:
	cmake -S . -B $(BUILD_DIR) -DCMAKE_BUILD_TYPE=Release
	cmake --build $(BUILD_DIR)

clean:
	rm -rf $(BUILD_DIR)

fclean: clean
	rm -f $(TARGET)

re: fclean all

debug:
	cmake -S . -B $(BUILD_DIR) -DCMAKE_BUILD_TYPE=Debug
	cmake --build $(BUILD_DIR)