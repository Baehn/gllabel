
#ifndef _H_UTIL
#define _H_UTIL

#include "types.hpp"
#include <vector>

// A bezier is written as 6 16-bit integers (12 bytes). Increments buffer by
// the number of bytes written (always 12). Coords are scaled from
// [0,glyphSize] to [0,UINT16_MAX].
void write_bezier_to_buffer(uint16_t **pbuffer, Bezier2 *bezier, Vec2 *glyphSize);
void write_glyph_data_to_buffer(
    uint8_t *buffer8,
    std::vector<Bezier2> &beziers,
    Vec2 &glyphSize,
    uint16_t gridX,
    uint16_t gridY,
    uint16_t gridWidth,
    uint16_t gridHeight);
#endif // _H_CUBIC2QUAD