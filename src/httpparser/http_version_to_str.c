#include <stdlib.h>

#include "httpparser.h"

char	*http_version_to_str(t_httpversion version)
{
	switch (version)
	{
		case HTTP_VER_0_9: return HTTP_VER_STR_0_9;
		case HTTP_VER_1_0: return HTTP_VER_STR_1_0;
		case HTTP_VER_1_1: return HTTP_VER_STR_1_1;
		case HTTP_VER_2_0: return HTTP_VER_STR_2_0;
	}

	return (NULL);
}