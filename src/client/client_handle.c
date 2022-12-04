#include <string.h>
#include <sys/socket.h> 

#include "logging.h"
#include "client.h"

int client_handle(void *p)
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