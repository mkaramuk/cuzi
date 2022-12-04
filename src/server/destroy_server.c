#include <stdlib.h>
#include <unistd.h>
#include "server.h"
#include "environment.h"

void destroy_server(t_serverdata *data)
{
	close(data->fd);
	destroy_arguments(data->args);
	free(data->addr);
	free(data);
}