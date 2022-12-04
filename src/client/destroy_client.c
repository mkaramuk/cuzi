#include <stdlib.h>
#include <unistd.h>
#include "client.h"

void destroy_client(t_client *client)
{
	free(client->addr);
	free(client->ip);
	close(client->fd);
	free(client);
}