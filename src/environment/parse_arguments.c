#include <stdlib.h>
#include <string.h>

#include "constants.h"
#include "environment.h"
#include "logging.h"

t_arguments *parse_arguments(int argc, char **argv, char **env)
{
	(void)argc;
	(void)argv;

	t_arguments *args = calloc(sizeof(*args), 1);
	args->ip = find_env(env, "IP");

	char *port_number = find_env(env, "PORT");
	if (!port_number)
		args->port = DEFAULT_PORT;
	else
		args->port = atoi(port_number);

	if (!args->ip)
	{
		args->ip = calloc(sizeof(char), strlen(DEFAULT_IP) + 1);
		strcpy(args->ip, DEFAULT_IP);
	}
	log_info("The address will listen is: %s:%d", args->ip, args->port);

	return (args);
}