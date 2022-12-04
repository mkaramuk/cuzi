#include <stdlib.h>
#include <arpa/inet.h>
#include <sys/socket.h>

#include "logging.h"
#include "client.h"
#include "server.h"

t_client *accept_client(t_serverdata *data)
{
	t_client *client = calloc(sizeof(*client), 1);

	client->addr = calloc(sizeof(*client->addr), 1);
	socklen_t addrlen = sizeof(client->addr);

	log_info("Waiting for client...");
	client->fd = accept(data->fd, (struct sockaddr *)client->addr, &addrlen);
	
	client->ip = calloc(sizeof(char), INET_ADDRSTRLEN);
	inet_ntop(AF_INET, &(client->addr->sin_addr), client->ip, INET_ADDRSTRLEN);
	client->port = (int)ntohs(client->addr->sin_port);
	client->server = data;

	log_info("Client is connected from %s:%d", client->ip, client->port);
	return (client);
}