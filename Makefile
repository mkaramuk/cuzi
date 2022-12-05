SRC		= $(shell find src -type f -name "*.c")
OBJ		= $(SRC:src/%.c=objs/%.o)

EXLIB  = ./argv_lib/argv.a
EXLIBDIR = ./argv_lib
EXLIBDIR2 = ./template_vector

NAME	= cuzi-server
CC		= gcc
LFLAGS	= 
CFLAGS	= -Wall -Wextra -Werror -I./includes -I$(EXLIBDIR2) -I$(EXLIBDIR) -g

all: $(EXLIB) $(NAME)

objs/%.o: src/%.c
	@mkdir -p $(shell dirname $@)
	$(CC) $(CFLAGS) -c $< -o $@

$(NAME): $(OBJ)
	$(CC) $(LFLAGS) $(OBJ) $(EXLIB) -o $(NAME) 

$(EXLIB):
	make -C ./argv_lib

clean:
	rm -rf $(OBJ)

fclean: clean
	rm -rf $(NAME)
	rm -rf objs

re: fclean all

run: all
	@echo Running...
	@./$(NAME)

.PHONY: all clean fclean re run