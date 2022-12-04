#include <stdlib.h>
#include <string.h>

char *find_env(char **env, char *name)
{
	for (; *env; env++)
	{
		if (!strncmp(*env, name, strlen(name)))
		{
			while (**env != '=' && **env)
				*env += 1;

			char *value = calloc(sizeof(char), strlen(*env + 1) + 1);
			strcpy(value, *env + 1);
			return (value);
		}
	}
	return (NULL);
}