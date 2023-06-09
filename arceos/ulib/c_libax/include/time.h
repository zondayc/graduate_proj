#ifndef __TIME_H__
#define __TIME_H__

#include <stddef.h>

typedef long time_t;

struct tm {
    int tm_sec;          /* seconds of minute */
    int tm_min;          /* minutes of hour */
    int tm_hour;         /* hours of day */
    int tm_mday;         /* day of month */
    int tm_mon;          /* month of year, 0 is first month(January) */
    int tm_year;         /* years, whose value equals the actual year minus 1900 */
    int tm_wday;         /* day of week, 0 is sunday, 1 is monday, and so on*/
    int tm_yday;         /* day of year */
    int tm_isdst;        /*  */
    long int tm_gmtoff;  /* */
    const char *tm_zone; /* timezone */
};

size_t strftime(char *__restrict__ _Buf, size_t _SizeInBytes, const char *__restrict__ _Format,
                const struct tm *__restrict__ _Tm);

struct tm *gmtime(const time_t *timer);

struct tm *localtime(const time_t *timep);
time_t time(time_t *t);

#endif
