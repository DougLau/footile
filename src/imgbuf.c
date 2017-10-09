/* imgbuf.c         Functions for blending image buffers.
 *
 * Copyright (c) 2017  Douglas P Lau
 */
#include <stdint.h>
#if defined (__SSE2__)
#   include <emmintrin.h>
#endif
#if defined (__ARM_NEON__)
#   include <arm_neon.h>
#endif

/* Blend two alpha buffers with saturating add.
 *
 * dst: Destination buffer.
 * src: Source buffer.
 * len: Size of buffers (in bytes).
 */
void alpha_buf_saturating_add(uint8_t *dst, const uint8_t *src, int len) {
    const int ln = len - 16;
    int i = 0;
#if defined (__SSE2__)
    for ( ; i <= ln; i += 16) {
        const __m128i *s = (const __m128i *) (src + i);
        __m128i *d = (__m128i *) (dst + i);
        __m128i a = _mm_loadu_si128(s);
        __m128i b = _mm_loadu_si128(d);
        _mm_storeu_si128(d, _mm_adds_epu8(a, b));
    }
#elif defined (__ARM_NEON__)
    for ( ; i <= ln; i += 16) {
        uint8x16_t a = vld1q_u8(src + i);
        uint8x16_t b = vld1q_u8(dst + i);
        vst1q_u8(dst + i, vqaddq_u8(a, b));
    }
#endif
    for ( ; i < len; i++) {
        int32_t r = src[i] + dst[i];
        dst[i] = (r < UINT8_MAX) ? r : UINT8_MAX;
    }
}
