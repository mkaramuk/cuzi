#include <string.h>
#include <stdlib.h>

static int	digit_count(int n)
{
	int	ret;

	ret = 0;
	if (n < 0)
	{
		ret++;
		n = -n;
	}
	while (n > 0)
	{
		ret++;
		n /= 10;
	}
	return (ret);
}

static char	*strtoint(char *str, int index, int nbr, int isneg)
{
	str[index] = '\0';
	while (--index)
	{
		str[index] = (nbr % 10) + '0';
		nbr /= 10;
	}
	if (isneg == 1)
		str[0] = '-';
	else
		str[0] = (nbr % 10) + '0';
	return (str);
}

char	*cuzi_itoa(int nbr)
{
	int		len;
	char	*rtn;
	int		isneg;

	len = digit_count(nbr);
	rtn = 0;
	isneg = 0;
	if (nbr == -2147483648)
		return (strdup("-2147483648"));
	if (nbr == 0)
		return (strdup("0"));
	else
		rtn = malloc(sizeof(char) * (len + 1));
	if (!rtn)
		return (0);
	isneg = 0;
	if (nbr < 0)
	{
		isneg++;
		nbr = -nbr;
	}
	return (strtoint(rtn, len, nbr, isneg));
}