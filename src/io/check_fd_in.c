#include <sys/poll.h>

int check_fd_in(int fd, int timeout)
{
	int		poll_ret;
	struct 	pollfd pfd;

	pfd.fd = fd;
	pfd.events = POLLIN;

	poll_ret = poll(&pfd, 1, timeout);
	if (poll_ret >= 0 && pfd.revents & POLLIN)
		return (pfd.fd);
	return (-1);
}
