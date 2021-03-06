dnl Process this file with autoconf to produce a configure script.
AC_PREREQ([2.71])
AC_INIT([a52dec],[0.7.4])
AC_CONFIG_SRCDIR([src/a52dec.c])
AC_CONFIG_AUX_DIR(autotools)
AC_CONFIG_MACRO_DIRS([m4])
AC_CONFIG_FILES([Makefile autotools/Makefile include/Makefile test/Makefile
    doc/Makefile src/Makefile liba52/Makefile libao/Makefile vc++/Makefile])
AM_INIT_AUTOMAKE
LT_INIT
AC_CONFIG_HEADERS(include/config.h)
AM_MAINTAINER_MODE
AC_CANONICAL_HOST

dnl ------------------------------------------------------------------------------------
dnl  Rules for library version information:
dnl
dnl   1. Start with version information of `0:0:0' for each libtool library.
dnl   2. Update the version information only immediately before a public release of
dnl      your software. More frequent updates are unnecessary, and only guarantee
dnl      that the current interface number gets larger faster.
dnl   3. If the library source code has changed at all since the last update, then
dnl      increment revision (`c:r:a' becomes `c:r+1:a').
dnl   4. If any interfaces have been added, removed, or changed since the last update,
dnl      increment current, and set revision to 0.
dnl   5. If any interfaces have been added since the last public release, then increment
dnl      age.
dnl   6. If any interfaces have been removed since the last public release, then set age
dnl      to 0.

dnl   This is libtool version of library, we add it to `--version-info` property.

m4_define([lt_current], [76])
m4_define([lt_revision], [4])
m4_define([lt_age], [76])

AC_SUBST([LIBA52_VERSION_INFO], [lt_current:lt_revision:lt_age])

dnl Checks for compiler
AC_PROG_CC
AC_PROG_GCC_TRADITIONAL

if test x"$CC" = x"tcc" -a x"`$CC -version 2>&1 | grep TenDRA`" != x""; then
    dnl TenDRA portability checking compiler
    TENDRA=yes
    CFLAGS="-Xp -Yansi -f`pwd`/include/tendra.h -DELIDE_CODE"
    enable_mlib=no
    enable_oss=no
    enable_solaris_audio=no
fi

dnl Checks for libraries.

LT_LIB_M

dnl Checks for header files.
AC_CHECK_HEADERS([sys/time.h sys/timeb.h io.h])
AC_CHECK_GENERATE_INTTYPES([include])

dnl Checks for typedefs, structures, and compiler characteristics.
AC_C_CONST
AC_C_RESTRICT
AC_TYPE_SIZE_T
AC_C_BIGENDIAN

dnl Checks for library functions.
AC_CHECK_FUNCS([_aligned_malloc aligned_alloc memalign gettimeofday ftime])

builtin([include],[liba52/configure.incl])
builtin([include],[libao/configure.incl])

AC_ARG_ENABLE([warnings],
    AS_HELP_STRING([--enable-warnings], [treat warnings as errors]))
if test x"$enable_warnings" = x"yes" -a x"$GCC" = x"yes"; then
    dnl compiler warnings
    OPT_CFLAGS="$CFLAGS -Wall -Werror"
    AC_TRY_CFLAGS([$OPT_CFLAGS],[CFLAGS=$OPT_CFLAGS])
elif test x"$TENDRA" = x"yes"; then
    dnl TenDRA portability checking compiler
    CFLAGS="$CFLAGS -DTenDRA_check"
fi

builtin([include],[src/configure.incl])

AC_C_ATTRIBUTE_ALIGNED

AC_OUTPUT
