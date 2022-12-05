#include <stdio.h>
#include <stdlib.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <unistd.h>
#include <signal.h>
#include <poll.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/poll.h>

#include "io.h"
#include "argv_lib.h"
#include "logging.h"
#include "httpparser.h"
#include "tempv_lib.h"
#include "server.h"
#include "environment.h"
#include "client.h"

typedef struct pollfd pfd_t; 

GENERATE_TEMP_VECTOR_HEADER(pfd_t, PFD)
GENERATE_TEMP_VECTOR(pfd_t, PFD)

int main(int argc, char **argv, char **env)
{
	t_serverdata	*data	= init_server(parse_arguments(argc, argv, env));
	t_client		*client;

	if (!data)
		return (1);



	while (1)
	{
		if ((client = accept_client(data)))
		{
			char buf[1024];
			int count = read(client->fd, buf, 1024);
			if (count > 0)
			{
				buf[count] = 0;
				t_httpresponse *response = calloc(sizeof(*response), 1);
				response->content = "<html><head><meta charset=\"UTF-8\"><title>cuzi-server</title></head><body>Merhaba Dünya!</body></html>";
				response->connection = "Closed";
				response->contentType = "text/html";
				response->code = HTTP_CODE_OK;
				response->version = HTTP_VER_1_0;
				response->contentLength = 0;
				http_send_response(client, response);
				free(response);
			}
		}
	}

	destroy_server(data);
	/* clients = argv_new(NULL, NULL);
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
				if (-1 == client_handle(clients->vector[i - 1])){
					argv_del_one(clients, i - 1, (void (*)(void *))destroy_client);
					PFDtempv_del_one(pvec, i, NULL);
					--i;
				}
			}
			++i;
		}
	} */
	
	return (0);
}