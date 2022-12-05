#include <string.h>
#include "httpparser.h"

t_httpcode	http_str_to_response_code(char *str)
{
	if (!strcmp(str, HTTP_CODE_REASON_200))
		return HTTP_CODE_OK;
	return (-1);
}