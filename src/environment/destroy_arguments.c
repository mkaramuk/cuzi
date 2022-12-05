#include <stdlib.h>

#include "environment.h"

void destroy_arguments(t_arguments *args)
{
	if (args)
	{
		free(args->ip);
		free(args);
	}
}