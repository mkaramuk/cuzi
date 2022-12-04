#ifndef SERVER_H
# define SERVER_H
# include "environment.h"

typedef struct s_serverdata
{
	t_arguments			*args;
	struct sockaddr_in	*addr;
	int					run;
	int					fd;
} t_serverdata;

void			destroy_server(t_serverdata *data);
t_serverdata	*init_server(t_arguments *args);

#endif