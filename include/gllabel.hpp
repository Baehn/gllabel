#include <string>
#include <vector>
#include <memory>
#include <map>
#include <glew.h>
#include <glm/glm.hpp>
// #include <ft2build.h>
// #include FT_FREETYPE_H
// #include FT_OUTLINE_H

class GLFontManager
{
public:
	struct AtlasGroup
	{
		// Grid atlas contains an array of square grids with side length
		// gridMaxSize. Each grid takes a single glyph and splits it into
		// cells that inform the fragment shader which curves of the glyph
		// intersect that cell. The cell contains coords to data in the bezier
		// atlas. The bezier atlas contains the actual bezier curves for each
		// glyph. All data for a single glyph must lie in a single row, although
		// multiple glyphs can be in one row. Each bezier curve takes three
		// "RGBA pixels" (12 bytes) of data.
		// Both atlases also encode some extra information, which is explained
		// where it is used in the code.
		uint32_t gridAtlasId;
		uint8_t *gridAtlas;
		uint16_t nextGridPos[2]; // XY pixel coordinates
		bool full; // For faster checking
		bool uploaded;

		uint32_t glyphDataBufId, glyphDataBufTexId;
		uint8_t *glyphDataBuf;
		uint16_t glyphDataBufOffset; // pixel coordinates
	};

	struct Glyph
	{
		uint16_t size[2]; // Width and height in FT units
		int16_t offset[2]; // Offset of glyph in FT units
		uint16_t bezierAtlasPos[2]; // XZ pixel coordinates (Z being atlas index)
		int16_t advance; // Amount to advance after character in FT units
	};
	std::map<int, std::map<uint32_t, Glyph> > glyphs;

public: // TODO: private
	std::vector<AtlasGroup> atlases;
	// FT_Library ft;
	GLuint glyphShader, uGridAtlas, uTransform;
	GLuint uGlyphData;

	GLFontManager();

	AtlasGroup * GetOpenAtlasGroup();

public:
	~GLFontManager();

	static std::shared_ptr<GLFontManager> singleton;
	static std::shared_ptr<GLFontManager> GetFontManager();

	Glyph * GetGlyphForCodepoint(uint32_t point);

};

class GLLabel
{
public:
	enum class Align
	{
		Start,
		Center,
		End
	};

	struct Color
	{
		uint8_t r,g,b,a;
	};

private:

	// Each of these arrays store the same "set" of data, but different versions
	// of it. Consequently, each of these will be exactly the same length
	// (except verts, which is six times longer than the other two, since
	// six verts per glyph).
	// Can't put them all into one array, because verts is needed alone as a
	// buffer to upload to the GPU, and text is needed alone mostly for GetText.
	std::vector<GLFontManager::Glyph *> glyphs;

	bool showingCaret;
	size_t caretPosition;
	float prevTime, caretTime;

public:
	std::shared_ptr<GLFontManager> manager;
	struct GlyphVertex
	{
		// XY coords of the vertex
		glm::vec2 pos;

		// Bit 0 (low) is norm coord X (varies per vertex)
		// Bit 1 is norm coord Y (varies per vertex)
		// Bits 2-31 are texel offset (byte offset / 4) into
		//   glyphDataBuf (same for all verticies of a glyph)
		uint32_t data;

		// RGBA color [0,255]
		Color color;
	};
	std::vector<GlyphVertex> verts;
	GLuint vertBuffer;
	GLLabel();
	~GLLabel();

	void InsertText(std::u32string text, size_t index, glm::vec4 color);
	inline void SetText(std::u32string text, glm::vec4 color) {
		this->InsertText(text, 0, color);
	}

	void SetHorzAlignment(Align horzAlign);
	void SetVertAlignment(Align vertAlign);

	// Render the label. Also uploads modified textures as necessary. 'time'
	// should be passed in monotonic seconds (no specific zero time necessary).
};
