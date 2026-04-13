##
## EPITECH PROJECT, 2026
## my_teams
## File description:
## Makefile
##

TARGET_DIR = target
TARGETS = myteams_server myteams_cli
RELEASE_DIR = $(TARGET_DIR)/release
DEBUG_DIR = $(TARGET_DIR)/debug

.PHONY: all clean fclean re debug test tests_run

all:
	cargo build --release
	cp $(RELEASE_DIR)/myteams_server .
	cp $(RELEASE_DIR)/myteams_cli .

clean:
	cargo clean
	rm -f $(TARGETS)

fclean: clean

re: fclean all

debug:
	cargo build
	cp $(DEBUG_DIR)/myteams_server .
	cp $(DEBUG_DIR)/myteams_cli .

test: tests_run

tests_run:
	cargo test