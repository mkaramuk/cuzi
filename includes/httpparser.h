#ifndef HTTPPARSER_H
# define HTTPPARSER_H
# include <stdlib.h>

# include "client.h"

// HTTP Methods
# define HTTP_GET 1
# define HTTP_POST 2

// String representations of HTTP Versions
# define HTTP_VER_STR_0_9 "HTTP/0.9"
# define HTTP_VER_STR_1_0 "HTTP/1.0"
# define HTTP_VER_STR_1_1 "HTTP/1.1"
# define HTTP_VER_STR_2_0 "HTTP/2.0"

// Integer representations of HTTP Versions
# define HTTP_VER_0_9 1
# define HTTP_VER_1_0 2
# define HTTP_VER_1_1 3
# define HTTP_VER_2_0 4

// Reasons of HTTP responses
# define HTTP_CODE_REASON_200 "OK"

// Integer representations of HTTP responses
# define HTTP_CODE_OK 200

typedef int t_httpversion;
typedef int t_httpmethod;
typedef int t_httpcode;

typedef struct s_httpheader {
	char			*path;
	t_httpversion	version;
	t_httpmethod	method;
} t_httpheader;

typedef struct s_httpresponse {
	t_httpversion		version;
	t_httpcode			code;
	size_t				contentLength;
	char				*contentType;
	char				*connection;
	char				*content;
} t_httpresponse;

// Constructor
t_httpheader	*http_parse_header(char *data);

int				http_send_response(t_client *client, t_httpresponse *response);

// String to value, value to string converters.
char			*http_response_reason(t_httpcode code);
char			*http_version_to_str(t_httpversion version);
t_httpversion	http_str_to_version(char *str);
t_httpversion	http_nstr_to_version(char *str, int n);
t_httpcode		http_str_to_response_code(char *str);

// Destructor
void			destroy_http_header(t_httpheader *header);

#endif