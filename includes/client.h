#ifndef CLIENT_H
# define CLIENT_H
# include <netinet/in.h>
# include "server.h"

typedef struct s_client
{
	t_serverdata		*server;
	struct sockaddr_in	*addr;
	char				*ip;
	int 				port;
	int					fd;
} t_client;


t_client	*accept_client(t_serverdata *data);
ssize_t		send_to_client(t_client *client, char *data);
int			client_handle(void *p);
void		destroy_client(t_client *client);

#endif