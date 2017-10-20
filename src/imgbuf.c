/* imgbuf.c         Functions for blending image buffers.
 *
 * Copyright (c) 2017  Douglas P Lau
 */
#include <stdbool.h>
#include <stdint.h>
#if defined (__SSSE3__)
#   include <tmmintrin.h>
#endif
#if defined (__ARM_NEON__)
#   include <arm_neon.h>
#endif

#define min(a, b)               \
    ({ __typeof__ (a) _a = (a); \
       __typeof__ (b) _b = (b); \
        _a < _b ? _a : _b; })
#define max(a, b)               \
    ({ __typeof__ (a) _a = (a); \
       __typeof__ (b) _b = (b); \
        _a > _b ? _a : _b; })

/* Accumulate signed area buffer and store in dest buffer.
 * Source buffer is zeroed upon return.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers.
 */
void accumulate_non_zero_c(uint8_t *dst, int16_t *src, int len) {
    int i = 0;
#if defined (__SSSE3__)
    __m128i zero = _mm_setzero_si128();
    __m128i sum = zero;
    /* mask for shuffling final sum into all lanes */
    __m128i mask = _mm_set1_epi16(0x0F0E);
    while (true) {
        __m128i a = _mm_loadu_si128((__m128i *) &src[i]);
        /* zeroing now is faster than memset later */
        _mm_storeu_si128((__m128i *) &src[i], zero);
        /*   a7 a6 a5 a4 a3 a2 a1 a0 */
        /* + a3 a2 a1 a0 __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 8));
        /* + a5 a4 a3 a2 a1 a0 __ __ */
        /* + a1 a0 __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 4));
        /* + a6 a5 a4 a3 a2 a1 a0 __ */
        /* + a2 a1 a0 __ __ __ __ __ */
        /* + a4 a3 a2 a1 a0 __ __ __ */
        /* + a0 __ __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 2));
        a = _mm_add_epi16(a, sum);
        __m128i b = _mm_packus_epi16(a, a);
        _mm_storel_epi64((__m128i *) &dst[i], b);
        i += 8;
        /* breaking here saves one shuffle */
        if (i >= len)
            break;
        sum = _mm_shuffle_epi8(a, mask);
    }
#else
    int16_t s = 0;
    for ( ; i < len; i++) {
        s += src[i];
        src[i] = 0;
        dst[i] = min(max(0, s), 255);
    }
#endif
}

/* Accumulate signed area buffer and store in dest buffer.
 * Source buffer is zeroed upon return.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers.
 */
void accumulate_odd_c(uint8_t *dst, int16_t *src, int len) {
    int i = 0;
#if defined (__SSSE3__)
    __m128i zero = _mm_setzero_si128();
    __m128i sum = zero;
    /* mask for shuffling final sum into all lanes */
    __m128i mask = _mm_set1_epi16(0x0F0E);
    while (true) {
        __m128i a = _mm_loadu_si128((__m128i *) &src[i]);
        /* zeroing now is faster than memset later */
        _mm_storeu_si128((__m128i *) &src[i], zero);
        /*   a7 a6 a5 a4 a3 a2 a1 a0 */
        /* + a3 a2 a1 a0 __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 8));
        /* + a5 a4 a3 a2 a1 a0 __ __ */
        /* + a1 a0 __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 4));
        /* + a6 a5 a4 a3 a2 a1 a0 __ */
        /* + a2 a1 a0 __ __ __ __ __ */
        /* + a4 a3 a2 a1 a0 __ __ __ */
        /* + a0 __ __ __ __ __ __ __ */
        a = _mm_add_epi16(a, _mm_slli_si128(a, 2));
        a = _mm_add_epi16(a, sum);
        __m128i c = _mm_and_si128(a, _mm_set1_epi16(0xFF));
        __m128i d = _mm_and_si128(a, _mm_set1_epi16(0x100));
        c = _mm_sub_epi16(c, d);
        c = _mm_abs_epi16(c);
        __m128i b = _mm_packus_epi16(c, c);
        _mm_storel_epi64((__m128i *) &dst[i], b);
        i += 8;
        /* breaking here saves one shuffle */
        if (i >= len)
            break;
        sum = _mm_shuffle_epi8(a, mask);
    }
#else
    int16_t s = 0;
    for ( ; i < len; i++) {
        s += src[i];
        src[i] = 0;
        dst[i] = min(max(0, s), 255);
    }
#endif
}
