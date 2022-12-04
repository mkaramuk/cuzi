#include <arpa/inet.h>
#include <netinet/in.h>
#include <stdlib.h>
#include <stdio.h>
#include <fcntl.h>
#include <string.h>
#include <errno.h>

#include "logging.h"
#include "server.h"

t_serverdata *init_server(t_arguments *args)
{
	t_serverdata *data = calloc(sizeof(*data), 1);

	int sock_fd = socket(AF_INET, SOCK_STREAM, 0);
	struct sockaddr_in *server_addr = calloc(sizeof(*server_addr), 1);

	if (sock_fd < 0)
	{
		fprintf(stderr, "Socket couldn't created\n");
		return (NULL);
	}
	log_info("Socket descriptor created %d", sock_fd);

	int opt = 1;
	setsockopt(sock_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

	memset(server_addr, 0, sizeof(*server_addr));
	server_addr->sin_family = AF_INET;
	server_addr->sin_port = htons(args->port);
	server_addr->sin_addr.s_addr = inet_addr(args->ip);

	data->addr = server_addr;
	data->args = args;
	data->run = 0;
	data->fd = sock_fd;

	if(fcntl(sock_fd, F_SETFL, O_NONBLOCK) < 0) {
		log_error("fcntl(): %s", strerror(errno));
		destroy_server(data);
		return (NULL);

	}
	
	if (bind(sock_fd, (struct sockaddr *)data->addr, sizeof(*data->addr)) < 0) {
		log_error("bind(): %s", strerror(errno));
		destroy_server(data);
		return (NULL);
	}
	
	if (listen(data->fd, 10) < 0) {
		log_error("listen(): %s", strerror(errno));
		destroy_server(data);
		return (NULL);
	}

	return (data);
}