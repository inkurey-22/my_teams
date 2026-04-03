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

.PHONY: all clean fclean re debug

all:
	cargo build --release
	cp $(RELEASE_DIR)/myteams_server .
	cp $(RELEASE_DIR)/myteams_cli .

clean:
	cargo clean
	rm -f $(TARGETS)

fclean: clean

re: fclean all

run_server: all
	LD_LIBRARY_PATH=./libs/myteams ./myteams_server 4242

run_client: all
	LD_LIBRARY_PATH=./libs/myteams ./myteams_cli localhost 4242

debug:
	cargo build
	cp $(DEBUG_DIR)/myteams_server .
	cp $(DEBUG_DIR)/myteams_cli .
