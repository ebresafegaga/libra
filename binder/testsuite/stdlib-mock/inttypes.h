#ifndef MOCK_INTTYPES_H
#define MOCK_INTTYPES_H

typedef signed char int8_t;
typedef short int16_t;
typedef int int32_t;
typedef long long int64_t;

typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef int8_t int_least8_t;
typedef int16_t int_least16_t;
typedef int32_t int_least32_t;
typedef int64_t int_least64_t;
typedef uint8_t uint_least8_t;
typedef uint16_t uint_least16_t;
typedef uint32_t uint_least32_t;
typedef uint64_t uint_least64_t;

typedef int8_t int_fast8_t;
typedef int16_t int_fast16_t;
typedef int32_t int_fast32_t;
typedef int64_t int_fast64_t;
typedef uint8_t uint_fast8_t;
typedef uint16_t uint_fast16_t;
typedef uint32_t uint_fast32_t;
typedef uint64_t uint_fast64_t;

typedef long intptr_t;
typedef unsigned long uintptr_t;

typedef long long intmax_t;
typedef unsigned long long uintmax_t;

#define INT8_MAX 127
#define INT16_MAX 32767
#define INT32_MAX 2147483647
#define INT64_MAX 9223372036854775807LL

#define INT8_MIN -128
#define INT16_MIN -32768
#define INT32_MIN (-INT32_MAX - 1)
#define INT64_MIN (-INT64_MAX - 1)

#define UINT8_MAX 255
#define UINT16_MAX 65535
#define UINT32_MAX 4294967295U
#define UINT64_MAX 18446744073709551615ULL

#define INT_LEAST8_MIN INT8_MIN
#define INT_LEAST16_MIN INT16_MIN
#define INT_LEAST32_MIN INT32_MIN
#define INT_LEAST64_MIN INT64_MIN

#define INT_LEAST8_MAX INT8_MAX
#define INT_LEAST16_MAX INT16_MAX
#define INT_LEAST32_MAX INT32_MAX
#define INT_LEAST64_MAX INT64_MAX

#define UINT_LEAST8_MAX UINT8_MAX
#define UINT_LEAST16_MAX UINT16_MAX
#define UINT_LEAST32_MAX UINT32_MAX
#define UINT_LEAST64_MAX UINT64_MAX

#define INT_FAST8_MIN INT8_MIN
#define INT_FAST16_MIN INT16_MIN
#define INT_FAST32_MIN INT32_MIN
#define INT_FAST64_MIN INT64_MIN

#define INT_FAST8_MAX INT8_MAX
#define INT_FAST16_MAX INT16_MAX
#define INT_FAST32_MAX INT32_MAX
#define INT_FAST64_MAX INT64_MAX

#define UINT_FAST8_MAX UINT8_MAX
#define UINT_FAST16_MAX UINT16_MAX
#define UINT_FAST32_MAX UINT32_MAX
#define UINT_FAST64_MAX UINT64_MAX

#define INTPTR_MIN INT64_MIN
#define INTPTR_MAX INT64_MAX
#define UINTPTR_MAX UINT64_MAX

#define INTMAX_MIN INT64_MIN
#define INTMAX_MAX INT64_MAX
#define UINTMAX_MAX UINT64_MAX

#define PTRDIFF_MIN INT64_MIN
#define PTRDIFF_MAX INT64_MAX

/* fprintf macros for signed integers */
#define PRId8 "d"    /* int8_t */
#define PRId16 "d"   /* int16_t */
#define PRId32 "d"   /* int32_t */
#define PRId64 "lld" /* int64_t */

#define PRIdLEAST8 "d"    /* int_least8_t */
#define PRIdLEAST16 "d"   /* int_least16_t */
#define PRIdLEAST32 "d"   /* int_least32_t */
#define PRIdLEAST64 "lld" /* int_least64_t */

#define PRIdFAST8 "d"    /* int_fast8_t */
#define PRIdFAST16 "d"   /* int_fast16_t */
#define PRIdFAST32 "d"   /* int_fast32_t */
#define PRIdFAST64 "lld" /* int_fast64_t */

#define PRIdMAX "jd" /* intmax_t */
#define PRIdPTR "ld" /* intptr_t */

#define PRIi8 "i"    /* int8_t */
#define PRIi16 "i"   /* int16_t */
#define PRIi32 "i"   /* int32_t */
#define PRIi64 "lli" /* int64_t */

#define PRIiLEAST8 "i"    /* int_least8_t */
#define PRIiLEAST16 "i"   /* int_least16_t */
#define PRIiLEAST32 "i"   /* int_least32_t */
#define PRIiLEAST64 "lli" /* int_least64_t */

#define PRIiFAST8 "i"    /* int_fast8_t */
#define PRIiFAST16 "i"   /* int_fast16_t */
#define PRIiFAST32 "i"   /* int_fast32_t */
#define PRIiFAST64 "lli" /* int_fast64_t */

#define PRIiMAX "ji" /* intmax_t */
#define PRIiPTR "li" /* intptr_t */

/* fprintf macros for unsigned integers */
#define PRIo8 "o"    /* int8_t */
#define PRIo16 "o"   /* int16_t */
#define PRIo32 "o"   /* int32_t */
#define PRIo64 "llo" /* int64_t */

#define PRIoLEAST8 "o"    /* int_least8_t */
#define PRIoLEAST16 "o"   /* int_least16_t */
#define PRIoLEAST32 "o"   /* int_least32_t */
#define PRIoLEAST64 "llo" /* int_least64_t */

#define PRIoFAST8 "o"    /* int_fast8_t */
#define PRIoFAST16 "o"   /* int_fast16_t */
#define PRIoFAST32 "o"   /* int_fast32_t */
#define PRIoFAST64 "llo" /* int_fast64_t */

#define PRIoMAX "jo" /* intmax_t */
#define PRIoPTR "lo" /* intptr_t */

#define PRIu8 "u"    /* uint8_t */
#define PRIu16 "u"   /* uint16_t */
#define PRIu32 "u"   /* uint32_t */
#define PRIu64 "llu" /* uint64_t */

#define PRIuLEAST8 "u"    /* uint_least8_t */
#define PRIuLEAST16 "u"   /* uint_least16_t */
#define PRIuLEAST32 "u"   /* uint_least32_t */
#define PRIuLEAST64 "llu" /* uint_least64_t */

#define PRIuFAST8 "u"    /* uint_fast8_t */
#define PRIuFAST16 "u"   /* uint_fast16_t */
#define PRIuFAST32 "u"   /* uint_fast32_t */
#define PRIuFAST64 "llu" /* uint_fast64_t */

#define PRIuMAX "ju" /* uintmax_t */
#define PRIuPTR "lu" /* uintptr_t */

#define PRIx8 "x"    /* uint8_t */
#define PRIx16 "x"   /* uint16_t */
#define PRIx32 "x"   /* uint32_t */
#define PRIx64 "llx" /* uint64_t */

#define PRIxLEAST8 "x"    /* uint_least8_t */
#define PRIxLEAST16 "x"   /* uint_least16_t */
#define PRIxLEAST32 "x"   /* uint_least32_t */
#define PRIxLEAST64 "llx" /* uint_least64_t */

#define PRIxFAST8 "x"    /* uint_fast8_t */
#define PRIxFAST16 "x"   /* uint_fast16_t */
#define PRIxFAST32 "x"   /* uint_fast32_t */
#define PRIxFAST64 "llx" /* uint_fast64_t */

#define PRIxMAX "jx" /* uintmax_t */
#define PRIxPTR "lx" /* uintptr_t */

#define PRIX8 "X"    /* uint8_t */
#define PRIX16 "X"   /* uint16_t */
#define PRIX32 "X"   /* uint32_t */
#define PRIX64 "llX" /* uint64_t */

#define PRIXLEAST8 "X"    /* uint_least8_t */
#define PRIXLEAST16 "X"   /* uint_least16_t */
#define PRIXLEAST32 "X"   /* uint_least32_t */
#define PRIXLEAST64 "llX" /* uint_least64_t */

#define PRIXFAST8 "X"    /* uint_fast8_t */
#define PRIXFAST16 "X"   /* uint_fast16_t */
#define PRIXFAST32 "X"   /* uint_fast32_t */
#define PRIXFAST64 "llX" /* uint_fast64_t */

#define PRIXMAX "jX" /* uintmax_t */
#define PRIXPTR "lX" /* uintptr_t */

/* fscanf macros for signed integers */
#define SCNd8 "hhd"  /* int8_t */
#define SCNd16 "hd"  /* int16_t */
#define SCNd32 "d"   /* int32_t */
#define SCNd64 "lld" /* int64_t */

#define SCNdLEAST8 "hhd"  /* int_least8_t */
#define SCNdLEAST16 "hd"  /* int_least16_t */
#define SCNdLEAST32 "d"   /* int_least32_t */
#define SCNdLEAST64 "lld" /* int_least64_t */

#define SCNdFAST8 "d"    /* int_fast8_t */
#define SCNdFAST16 "d"   /* int_fast16_t */
#define SCNdFAST32 "d"   /* int_fast32_t */
#define SCNdFAST64 "lld" /* int_fast64_t */

#define SCNdMAX "jd" /* intmax_t */
#define SCNdPTR "ld" /* intptr_t */

#define SCNi8 "hhi"  /* int8_t */
#define SCNi16 "hi"  /* int16_t */
#define SCNi32 "i"   /* int32_t */
#define SCNi64 "lli" /* int64_t */

#define SCNiLEAST8 "hhi"  /* int_least8_t */
#define SCNiLEAST16 "hi"  /* int_least16_t */
#define SCNiLEAST32 "i"   /* int_least32_t */
#define SCNiLEAST64 "lli" /* int_least64_t */

#define SCNiFAST8 "i"    /* int_fast8_t */
#define SCNiFAST16 "i"   /* int_fast16_t */
#define SCNiFAST32 "i"   /* int_fast32_t */
#define SCNiFAST64 "lli" /* int_fast64_t */

#define SCNiMAX "ji" /* intmax_t */
#define SCNiPTR "li" /* intptr_t */

/* fscanf macros for unsigned integers */
#define SCNo8 "hho"  /* uint8_t */
#define SCNo16 "ho"  /* uint16_t */
#define SCNo32 "o"   /* uint32_t */
#define SCNo64 "llo" /* uint64_t */

#define SCNoLEAST8 "hho"  /* uint_least8_t */
#define SCNoLEAST16 "ho"  /* uint_least16_t */
#define SCNoLEAST32 "o"   /* uint_least32_t */
#define SCNoLEAST64 "llo" /* uint_least64_t */

#define SCNoFAST8 "o"    /* uint_fast8_t */
#define SCNoFAST16 "o"   /* uint_fast16_t */
#define SCNoFAST32 "o"   /* uint_fast32_t */
#define SCNoFAST64 "llo" /* uint_fast64_t */

#define SCNoMAX "jo" /* uintmax_t */
#define SCNoPTR "lo" /* uintptr_t */

#define SCNu8 "hhu"  /* uint8_t */
#define SCNu16 "hu"  /* uint16_t */
#define SCNu32 "u"   /* uint32_t */
#define SCNu64 "llu" /* uint64_t */

#define SCNuLEAST8 "hhu"  /* uint_least8_t */
#define SCNuLEAST16 "hu"  /* uint_least16_t */
#define SCNuLEAST32 "u"   /* uint_least32_t */
#define SCNuLEAST64 "llu" /* uint_least64_t */

#define SCNuFAST8 "u"    /* uint_fast8_t */
#define SCNuFAST16 "u"   /* uint_fast16_t */
#define SCNuFAST32 "u"   /* uint_fast32_t */
#define SCNuFAST64 "llu" /* uint_fast64_t */

#define SCNuMAX "ju" /* uintmax_t */
#define SCNuPTR "lu" /* uintptr_t */

#define SCNx8 "hhx"  /* uint8_t */
#define SCNx16 "hx"  /* uint16_t */
#define SCNx32 "x"   /* uint32_t */
#define SCNx64 "llx" /* uint64_t */

#define SCNxLEAST8 "hhx"  /* uint_least8_t */
#define SCNxLEAST16 "hx"  /* uint_least16_t */
#define SCNxLEAST32 "x"   /* uint_least32_t */
#define SCNxLEAST64 "llx" /* uint_least64_t */

#define SCNxFAST8 "x"    /* uint_fast8_t */
#define SCNxFAST16 "x"   /* uint_fast16_t */
#define SCNxFAST32 "x"   /* uint_fast32_t */
#define SCNxFAST64 "llx" /* uint_fast64_t */

#define SCNxMAX "jx" /* uintmax_t */
#define SCNxPTR "lx" /* uintptr_t */

#endif // MOCK_INTTYPES_H
