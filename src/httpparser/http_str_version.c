#include <string.h>
#include "httpparser.h"

t_httpversion	http_str_to_version(char *str)
{
	return http_nstr_to_version(str, strlen(str));
}