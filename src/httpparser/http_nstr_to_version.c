#include <string.h>
#include "httpparser.h"

t_httpversion	http_nstr_to_version(char *str, int n)
{
	if (!strncmp(str, HTTP_VER_STR_0_9, n)) return HTTP_VER_0_9;
	else if (!strncmp(str, HTTP_VER_STR_1_0, n)) return HTTP_VER_1_0;
	else if (!strncmp(str, HTTP_VER_STR_1_1, n)) return HTTP_VER_1_1;
	else if (!strncmp(str, HTTP_VER_STR_2_0, n)) return HTTP_VER_2_0;
	
	return (-1);
}