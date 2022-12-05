#include <string.h>

#include "server.h"
#include "client.h"
#include "httpparser.h"
#include "string_utils.h"

int	http_send_response(t_client *client, t_httpresponse *response)
{
	char	*str = cuzi_itoa(response->code);

	send_to_client(client, http_version_to_str(response->version));
	send_to_client(client, " ");
	send_to_client(client, str);
	free(str);
	send_to_client(client, " ");
	send_to_client(client, http_response_reason(response->code));
	send_to_client(client, "\r\n");
	send_to_client(client, "Server: cuzi-server (Linux x64)\r\n");
	send_to_client(client, "Content-Length: ");
	str = cuzi_itoa(strlen(response->content));
	send_to_client(client, str);
	free(str);
	send_to_client(client, "\r\n");
	send_to_client(client, "Content-Type: ");
	send_to_client(client, response->contentType);
	send_to_client(client, "\r\n");
	send_to_client(client, "Connection: Closed\r\n\r\n");
	send_to_client(client, response->content);
	return (0);
}

/*


HTTP/1.1 200 OK
Server: cuzi-server (Linux x64)
Content-Length: 80
Content-Type: text/html
Conntection: Closed

<html><head><title>cuzi-server</title></head><body>Merhaba Dünya!</body></html>

*/
