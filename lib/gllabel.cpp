/*
 * zelbrium <zelbrium@gmail.com>, 2016-2020.
 *
 * This code is based on Will Dobbie's WebGL vector-based text rendering (2016).
 * It can be found here:
 * https://wdobbie.com/post/gpu-text-rendering-with-vector-textures/
 *
 * Dobbie's original code used a pre-generated bezier curve atlas generated
 * from a PDF. This GLLabel class allows for live text rendering based on
 * glyph curves exported from FreeType2.
 *
 * Text is rendered size-independently. This means you can scale, rotate,
 * or reposition text rendered using GLLabel without any loss of quality.
 * All that's required is a font file to load the text from. Currently, any TTF
 * font that does not use cubic beziers or make use of very detailed glyphs,
 * such as many Hanzi / Kanji characters, should work.
 */

#include <gllabel.hpp>
#include "vgrid.hpp"
#include "util.hpp"
#include <set>
#include <iostream>
#include "data.hpp"

#define sq(x) ((x) * (x))

std::shared_ptr<GLFontManager> GLFontManager::singleton = nullptr;


static const uint8_t kGridMaxSize = 20;
static const uint16_t kGridAtlasSize = 256;	  // Fits exactly 1024 8x8 grids
static const uint16_t kBezierAtlasSize = 256; // Fits around 700-1000 glyphs, depending on their curves
static const uint8_t kAtlasChannels = 4;	  // Must be 4 (RGBA), otherwise code breaks

GLLabel::GLLabel()
	: showingCaret(false), caretPosition(0), prevTime(0)
{
	this->manager = GLFontManager::GetFontManager();
}

GLLabel::~GLLabel()
{
	// glDeleteBuffers(1, &this->vertBuffer);
}

void GLLabel::InsertText(std::u32string text, size_t index, glm::vec4 color)
{
	this->glyphs.resize(text.size());

	GlyphVertex emptyVert{};
	this->verts.insert(this->verts.begin() + index * 6, text.size() * 6, emptyVert);

	glm::vec2 appendOffset(0, 0);

	for (size_t i = 0; i < text.size(); i++)
	{
		GLFontManager::Glyph *glyph = this->manager->GetGlyphForCodepoint(text[i]);

		GlyphVertex v[6]{}; // Insertion code depends on v[0] equaling appendOffset (therefore it is also set before continue;s above)
		v[0].pos = glm::vec2(0, 0);
		v[1].pos = glm::vec2(glyph->size[0], 0);
		v[2].pos = glm::vec2(0, glyph->size[1]);
		v[3].pos = glm::vec2(glyph->size[0], glyph->size[1]);
		v[4].pos = glm::vec2(0, glyph->size[1]);
		v[5].pos = glm::vec2(glyph->size[0], 0);
		for (unsigned int j = 0; j < 6; j++)
		{
			v[j].pos += appendOffset;
			v[j].pos[0] += glyph->offset[0];
			v[j].pos[1] += glyph->offset[1];

			v[j].color = {(uint8_t)(color.r * 255), (uint8_t)(color.g * 255), (uint8_t)(color.b * 255), (uint8_t)(color.a * 255)};

			// Encode both the bezier position and the norm coord into one int
			// This theoretically could overflow, but the atlas position will
			// never be over half the size of a uint16, so it's fine.
			unsigned int k = (j < 4) ? j : 6 - j;
			unsigned int normX = k & 1;
			unsigned int normY = k > 1;
			unsigned int norm = (normX << 1) + normY;
			v[j].data = (glyph->bezierAtlasPos[0] << 2) + norm;
			this->verts[(index + i) * 6 + j] = v[j];
		}

		appendOffset.x += glyph->advance;
		this->glyphs[index + i] = glyph;
	}
}

GLFontManager::GLFontManager() //: defaultFace(nullptr)
{
}

GLFontManager::~GLFontManager()
{
	// TODO: Destroy atlases
	// glDeleteProgram(this->glyphShader);
	// FT_Done_FreeType(this->ft);
}

std::shared_ptr<GLFontManager> GLFontManager::GetFontManager()
{
	if (!GLFontManager::singleton)
	{
		GLFontManager::singleton = std::shared_ptr<GLFontManager>(new GLFontManager());
	}
	return GLFontManager::singleton;
}

GLFontManager::AtlasGroup *GLFontManager::GetOpenAtlasGroup()
{
	if (this->atlases.size() == 0 || this->atlases[this->atlases.size() - 1].full)
	{
		AtlasGroup group{};
		group.glyphDataBuf = new uint8_t[sq(kBezierAtlasSize) * kAtlasChannels]();
		group.gridAtlas = new uint8_t[sq(kGridAtlasSize) * kAtlasChannels]();
		group.uploaded = true;

		this->atlases.push_back(group);
	}

	return &this->atlases[this->atlases.size() - 1];
}

#pragma pack(push, 1)
struct bitmapdata
{
	char magic[2];
	uint32_t size;
	uint16_t res1;
	uint16_t res2;
	uint32_t offset;

	uint32_t biSize;
	uint32_t width;
	uint32_t height;
	uint16_t planes;
	uint16_t bitCount;
	uint32_t compression;
	uint32_t imageSizeBytes;
	uint32_t xpelsPerMeter;
	uint32_t ypelsPerMeter;
	uint32_t clrUsed;
	uint32_t clrImportant;
};
#pragma pack(pop)

GLFontManager::Glyph *GLFontManager::GetGlyphForCodepoint(uint32_t point)
{

	AtlasGroup *atlas = this->GetOpenAtlasGroup();


	int glyphWidth = 1398;
	int glyphHeight = 1450;

	int16_t horiBearingX = 97;
	int16_t horiBearingY = 1430;
	int16_t horiAdvance = 1593;

	uint8_t gridWidth = kGridMaxSize;
	uint8_t gridHeight = kGridMaxSize;


	std::vector<Bezier2> curves(19);
	write_test_curves(curves);

	VGrid grid(curves, Vec2(glyphWidth, glyphHeight), gridWidth, gridHeight);

	// Although the data is represented as a 32bit texture, it's actually
	// two 16bit ints per pixel, each with an x and y coordinate for
	// the bezier. Every six 16bit ints (3 pixels) is a full bezier
	// Plus two pixels for grid position information
	uint16_t bezierPixelLength = 2 + curves.size() * 3;

	uint8_t *bezierData = atlas->glyphDataBuf + (atlas->glyphDataBufOffset * kAtlasChannels);


	Vec2 glyphSize(glyphWidth, glyphHeight);
	write_glyph_data_to_buffer(
		bezierData,
		curves,
		glyphSize,
		atlas->nextGridPos[0],
		atlas->nextGridPos[1],
		kGridMaxSize,
		kGridMaxSize);

	// TODO: Integrate with AtlasGroup / replace AtlasGroup
	WriteVGridAt(grid, atlas->nextGridPos[0], atlas->nextGridPos[1], atlas->gridAtlas, kGridAtlasSize, kGridAtlasSize, kAtlasChannels);

	GLFontManager::Glyph glyph{};
	glyph.bezierAtlasPos[0] = atlas->glyphDataBufOffset;
	glyph.bezierAtlasPos[1] = this->atlases.size() - 1;
	glyph.size[0] = glyphWidth;
	glyph.size[1] = glyphHeight;
	glyph.offset[0] = horiBearingX;
	glyph.offset[1] = horiBearingY - glyphHeight;
	glyph.advance = horiAdvance;
	this->glyphs[0][point] = glyph;

	atlas->glyphDataBufOffset += bezierPixelLength;
	atlas->nextGridPos[0] += kGridMaxSize;
	atlas->uploaded = false;

	return &this->glyphs[0][point];
}


