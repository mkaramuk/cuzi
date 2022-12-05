#include <stdlib.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <string.h>

#include "logging.h"
#include "client.h"
#include "server.h"
#include "io.h"

t_client *accept_client(t_serverdata *data)
{
	int					fd;
	t_client			*client;
	struct sockaddr_in	addr;
	int					addrlen = sizeof(addr);

	if (check_fd_in(data->fd, 100) == -1 ||
		(fd = accept(data->fd, (struct sockaddr *)&addr, (socklen_t *)&addrlen)) == -1)
		return (NULL);

	client = calloc(sizeof(*client), 1);
	client->addr = calloc(sizeof(*client->addr), 1);
	memcpy(client->addr, &addr, addrlen);

	client->fd = fd;

	client->ip = calloc(sizeof(char), INET_ADDRSTRLEN);
	inet_ntop(AF_INET, &(client->addr->sin_addr), client->ip, INET_ADDRSTRLEN);

	client->port = (int)ntohs(client->addr->sin_port);
	client->server = data;

	log_info("Client is connected from %s:%d", client->ip, client->port);
	return (client);
}