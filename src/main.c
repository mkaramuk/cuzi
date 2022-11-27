#include <stdio.h>
#include <stdlib.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <unistd.h>
#include <pthread.h>
#include <signal.h>
#include <poll.h>
#include <logging.h>
#include <string.h>
#include <errno.h>
#include <argv_lib.h>
#include <fcntl.h>
#include <sys/poll.h>
#include <tempv_lib.h>

#define DEFAULT_IP "127.0.0.1"
#define DEFAULT_PORT 2002

typedef enum enum_cmdtype
{
	CMD_EXIT,
	CMD_UNDEFINED,
	CMD_START,
	CMD_STOP,
	CMD_NOTHING
} e_cmdtype;

typedef struct s_args
{
	char	*ip;
	int		port;
} t_args;

typedef struct s_serverdata
{
	t_args				*args;
	struct sockaddr_in	*addr;
	int					run;
	int					fd;
} t_serverdata;

typedef struct s_client
{
	t_serverdata		*server;
	pthread_t			th_id;
	struct sockaddr_in	*addr;
	char				*ip;
	int 				port;
	int					fd;
} t_client;


void destroy_server(t_serverdata *data);
t_serverdata *init_server(t_args *args);

e_cmdtype parse_command(char *cmd)
{
	if (!strcmp(cmd, "exit"))
		return CMD_EXIT;
	else if (!strcmp(cmd, "start"))
		return CMD_START;
	else if (!strcmp(cmd, "stop"))
		return CMD_STOP;
	else if (!*cmd)
		return CMD_NOTHING;
	return (CMD_UNDEFINED);
}

char *find_env(char **env, char *name)
{
	for (; *env; env++)
	{
		if (!strncmp(*env, name, strlen(name)))
		{
			while (**env != '=' && **env)
				*env += 1;

			char *value = calloc(sizeof(char), strlen(*env + 1) + 1);
			strcpy(value, *env + 1);
			return (value);
		}
	}
	return (NULL);
}

t_args *parse_arguments(int argc, char **argv, char **env)
{
	(void)argc;
	(void)argv;

	t_args *args = calloc(sizeof(*args), 1);
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

void destroy_args(t_args *args)
{
	free(args->ip);
	free(args);
}

t_serverdata *init_server(t_args *args)
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

void destroy_server(t_serverdata *data)
{
	close(data->fd);
	destroy_args(data->args);
	free(data->addr);
	free(data);
}

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

void destroy_client(t_client *client)
{
	free(client->addr);
	free(client->ip);
	close(client->fd);
	free(client);
}

ssize_t send_to_client(t_client *client, char *data)
{
	return (send(client->fd, data, strlen(data), 0));
}

 int client_stuff(void *p)
{
	t_client *client = p;

	log_info("Sending welcome message...");
	if (send_to_client(client, "Hello, world!\n") == -1) {
		log_error("Message couldn't send. Connection terminating...");
		return (-1);
	}

	while (1)
	{
		char str[2083];
		ssize_t recieved = recv(client->fd, str, 2083, 0);
		if (recieved == 0)
			return (-1);
		else if (recieved == -1)
			return (0);
		str[recieved - 2] = 0;
		if (*str)
			log_info("Recieved from [%s:%d]: %s", client->ip, client->port, str);
		
		if (*str == '/')
		{
			if (!strcmp(str + 1, "exit"))
				break ;
			else if (!strcmp(str + 1, "selamla"))
			{
				log_info("Greeting %s:%d", client->ip, client->port);
				send_to_client(client, "Hi.\n");
			}
		}
	}
	return (0);
}

typedef struct pollfd pfd_t; 



GENERATE_TEMP_VECTOR_HEADER(pfd_t, PFD)
GENERATE_TEMP_VECTOR(pfd_t, PFD)


int main(int argc, char **argv, char **env)
{
	t_args			*args 			= parse_arguments(argc, argv, env);
	t_serverdata	*data 			= init_server(args);
	argv_t			*clients;
	pfd_t			pf;
	t_client		*client;
	int				stat;
	PFDtempv_t		*pvec;

	if (!data)
		return (1);	
	clients = argv_new(NULL, NULL);
	printf("%ld\n", clients->len);
	pf = (struct pollfd){0};
	pf.fd = data->fd;
	pf.events = POLLIN;
	pvec = PFDtempv_new(NULL, 0, NULL);	
	PFDtempv_push(pvec ,pf);
	while(1)
	{
		stat = poll(pvec->vector, pvec->len, -1);
		if (stat < 0) {
			dprintf(2, "poll error\n");
			exit(1);
		}
		if (stat == 0) {
			dprintf(2,"time out\n");
			exit(1);
		}
		int i = 0;
		while (i < (int)pvec->len)
		{
			pf = pvec->vector[i];
			if (pf.revents == 0) {
				++i;
				continue;
			}
			if (pf.revents != POLLIN)
			{
					argv_del_one(clients, i - 1, (void (*)(void *))destroy_client); // 
					PFDtempv_del_one(pvec, i, NULL);
					continue;
			}
			if (pf.fd == data->fd)
			{
				client = accept_client(data); 
				fcntl(client->fd, F_SETFL, O_NONBLOCK);
				if (argv_push(clients, client) < 0)
					continue;
				printf("push odu\n");
				pf = (struct pollfd){.fd = client->fd, .events = POLLIN};
				PFDtempv_push(pvec, pf);
				i++;
				continue;	
			}
			else
			{
				if (-1 == client_stuff(clients->vector[i - 1])){
					argv_del_one(clients, i - 1, (void (*)(void *))destroy_client);
					PFDtempv_del_one(pvec, i, NULL);
					--i;
				}
			}
			++i;
		}
	}
	
	return (0);
}