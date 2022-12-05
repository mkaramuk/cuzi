#include <stdlib.h>
#include <string.h>

#include "httpparser.h"

static int find_index(char *str, char chr)
{
	for (int i = 0; str[i]; i++)
		if (str[i] == chr)
			return (i);
	return (-1);
}

t_httpheader	*http_parse_header(char *data)
{
	t_httpheader	*header;

	int target_index = find_index(data, ' ');
	if (target_index == -1)
		return (NULL);

	header = calloc(sizeof(*header), 1);
	if (!strncmp(data, "GET", target_index))
		header->method = HTTP_GET;
	else if (!strncmp(data, "POST", target_index))
		header->method = HTTP_POST;
	else
	{
		free(header);
		return (NULL);
	}
	
	data += target_index + 1;
	target_index = find_index(data, ' ');

	if (target_index == -1)
	{
		free(header);
		return (NULL);
	}
	header->path = calloc(sizeof(char), target_index + 1);
	strncpy(header->path, data, target_index);

	data += target_index + 1;
	target_index = find_index(data, '\r');

	if (target_index == -1)
	{
		free(header->path);
		free(header);
		return (NULL);
	}

	header->version = http_nstr_to_version(data, target_index);
	
	if (header->version == -1)
	{
		free(header->path);
		free(header);
		return (NULL);
	}

	return (header);
}