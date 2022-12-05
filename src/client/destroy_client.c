#include <stdlib.h>
#include <unistd.h>
#include "client.h"

void destroy_client(t_client *client)
{
	if (client)
	{
		if (client->fd > 0)
			close(client->fd);
		free(client->addr);
		free(client->ip);
		free(client);
	}
}