#include <stdlib.h>
#include <sys/socket.h>
#include <string.h>

#include "client.h"

ssize_t send_to_client(t_client *client, char *data)
{
	return (send(client->fd, data, strlen(data), 0));
}