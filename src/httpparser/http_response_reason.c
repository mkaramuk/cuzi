#include <stdlib.h>
#include "httpparser.h"

char *http_response_reason(t_httpcode code)
{
	switch (code)
	{
		case HTTP_CODE_OK: return HTTP_CODE_REASON_200;
	}
	return (NULL);
}