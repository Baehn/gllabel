#include "util.hpp"
#include <iostream>

// A bezier is written as 6 16-bit integers (12 bytes). Increments buffer by
// the number of bytes written (always 12). Coords are scaled from
// [0,glyphSize] to [0,UINT16_MAX].
void write_bezier_to_buffer(uint16_t **pbuffer, Bezier2 *bezier, Vec2 *glyphSize)
{
	uint16_t *buffer = *pbuffer;
	buffer[0] = bezier->e0.x * UINT16_MAX / glyphSize->w;
	buffer[1] = bezier->e0.y * UINT16_MAX / glyphSize->h;
	buffer[2] = bezier->c.x  * UINT16_MAX / glyphSize->w;
	buffer[3] = bezier->c.y  * UINT16_MAX / glyphSize->h;
	buffer[4] = bezier->e1.x * UINT16_MAX / glyphSize->w;
	buffer[5] = bezier->e1.y * UINT16_MAX / glyphSize->h;
	*pbuffer += 6;
}

void write_glyph_data_to_buffer(
	uint8_t *buffer8,
	std::vector<Bezier2> &beziers,
	Vec2 &glyphSize,
	uint16_t gridX,
	uint16_t gridY,
	uint16_t gridWidth,
	uint16_t gridHeight)
{
	std::cout << std::endl;
	uint16_t *buffer = (uint16_t *)buffer8;
	buffer[0] = gridX;
	buffer[1] = gridY;
	buffer[2] = gridWidth;
	buffer[3] = gridHeight;
	buffer += 4;

	for (size_t i = 0; i < beziers.size(); i++) {
		write_bezier_to_buffer(&buffer, &beziers[i], &glyphSize);
	}

	std::cout << glyphSize.x << std::endl;
	std::cout << glyphSize.y << std::endl;
	uint16_t *buffer2 = (uint16_t *)buffer8;
	for (int i = 0; i < 200; i++)
	{
		std::cout << unsigned(buffer2[i]) <<  ", ";
	}
}