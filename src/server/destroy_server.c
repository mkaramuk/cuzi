#include <stdlib.h>
#include <unistd.h>

#include "logging.h"
#include "server.h"
#include "environment.h"

void destroy_server(t_serverdata *data)
{
	if (data)
	{
		log_info("Server socket closing on fd %d", data->fd);
		if (data->fd > 0)
			close(data->fd);
		destroy_arguments(data->args);
		free(data->addr);
		free(data);
	}
}