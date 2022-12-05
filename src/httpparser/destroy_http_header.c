#include "httpparser.h"

void destroy_http_header(t_httpheader *header)
{
	if (header)
	{
		free(header->path);
		free(header);
	}
}