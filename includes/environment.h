#ifndef ENVIRONMENT_H
# define ENVIRONMENT_H

typedef struct s_args
{
	char	*ip;
	int		port;
} t_arguments;

t_arguments	*parse_arguments(int argc, char **argv, char **env);
void		destroy_arguments(t_arguments *args);
char		*find_env(char **env, char *name);

#endif