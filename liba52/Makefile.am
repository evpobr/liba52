AM_CFLAGS = @CFLAGS@ @LIBA52_CFLAGS@

lib_LTLIBRARIES = liba52.la

liba52_la_SOURCES = bitstream.c imdct.c bit_allocate.c parse.c downmix.c \
					a52_internal.h bitstream.h tables.h
liba52_la_LIBADD = @LIBA52_LIBS@ @LIBM@
liba52_la_LDFLAGS = -no-undefined -export-symbols liba52.sym \
					-version-info @LIBA52_VERSION_INFO@

EXTRA_DIST = configure.incl
