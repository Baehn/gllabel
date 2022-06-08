/*
 * zelbrium <zelbrium@gmail.com>, 2016-2020.
 *
 * Demo code for GLLabel. Depends on GLFW3, GLEW, GLM, FreeType2, and C++11.
 */

#include <gllabel.hpp>
#include <glfw3.h>
#define GLM_ENABLE_EXPERIMENTAL
#include <glm/gtx/transform.hpp>
#include <codecvt>
#include <iomanip>
#include <string>
#include <sstream>
#include <iostream>
#include <glm/gtc/type_ptr.hpp>




#define sq(x) ((x) * (x))

static const uint8_t kGridMaxSize = 20;
static const uint16_t kGridAtlasSize = 256;	  // Fits exactly 1024 8x8 grids
static const uint16_t kBezierAtlasSize = 256; // Fits around 700-1000 glyphs, depending on their curves
static const uint8_t kAtlasChannels = 4;	  // Must be 4 (RGBA), otherwise code breaks

static uint32_t width = 1280;
static uint32_t height = 800;

static GLLabel *Label;
static bool spin = false;
float horizontalTransform = -0.9;
float verticalTransform = 0.6;
float scale = 1;

void onScroll(GLFWwindow *window, double deltaX, double deltaY);
void onResize(GLFWwindow *window, int width, int height);
std::u32string toUTF32(const std::string &s);
static glm::vec3 pt(float pt);

const char *kGlyphVertexShader = R"(
#version 330 core
uniform samplerBuffer uGlyphData;
uniform mat4 uTransform;

layout(location = 0) in vec2 vPosition;
layout(location = 1) in uint vData;
layout(location = 2) in vec4 vColor;

out vec4 oColor;
flat out uint glyphDataOffset;
flat out ivec4 oGridRect;
out vec2 oNormCoord;

float ushortFromVec2(vec2 v)
{
	return (v.y * 65280.0 + v.x * 255.0);
}

ivec2 vec2FromPixel(uint offset)
{
	vec4 pixel = texelFetch(uGlyphData, int(offset));
	return ivec2(ushortFromVec2(pixel.xy), ushortFromVec2(pixel.zw));
}

void main()
{
	oColor = vColor;
	glyphDataOffset = vData >> 2u;
	oNormCoord = vec2((vData & 2u) >> 1, vData & 1u);
	oGridRect = ivec4(vec2FromPixel(glyphDataOffset), vec2FromPixel(glyphDataOffset + 1u));
	gl_Position = uTransform*vec4(vPosition, 0.0, 1.0);
}
)";

const char *kGlyphFragmentShader = R"(
// This shader slightly modified from source code by Will Dobbie.
// See dobbieText.cpp for more info.

#version 330 core
precision highp float;

#define numSS 4
#define pi 3.1415926535897932384626433832795
#define kPixelWindowSize 1.0

uniform sampler2D uGridAtlas;
uniform samplerBuffer uGlyphData;

in vec4 oColor;
flat in uint glyphDataOffset;
flat in ivec4 oGridRect;
in vec2 oNormCoord;

layout(location = 0) out vec4 outColor;

float positionAt(float p0, float p1, float p2, float t)
{
	float mt = 1.0 - t;
	return mt*mt*p0 + 2.0*t*mt*p1 + t*t*p2;
}

float tangentAt(float p0, float p1, float p2, float t)
{
	return 2.0 * (1.0-t) * (p1 - p0) + 2.0 * t * (p2 - p1);
}

bool almostEqual(float a, float b)
{
	return abs(a-b) < 1e-5;
}

float normalizedUshortFromVec2(vec2 v)
{
	return (v.y * 65280.0 + v.x * 255.0) / 65536.0;
}

vec4 getPixelByOffset(int offset)
{
	return texelFetch(uGlyphData, offset);
}

void fetchBezier(int coordIndex, out vec2 p[3])
{
	for (int i=0; i<3; i++) {
		vec4 pixel = getPixelByOffset(int(glyphDataOffset) + 2 + coordIndex*3 + i);
		p[i] = vec2(normalizedUshortFromVec2(pixel.xy), normalizedUshortFromVec2(pixel.zw)) - oNormCoord;
	}
}

int getAxisIntersections(float p0, float p1, float p2, out vec2 t)
{
	if (almostEqual(p0, 2.0*p1 - p2)) {
		t[0] = 0.5 * (p2 - 2.0*p1) / (p2 - p1);
		return 1;
	}

	float sqrtTerm = p1*p1 - p0*p2;
	if (sqrtTerm < 0.0) return 0;
	sqrtTerm = sqrt(sqrtTerm);
	float denom = p0 - 2.0*p1 + p2;
	t[0] = (p0 - p1 + sqrtTerm) / denom;
	t[1] = (p0 - p1 - sqrtTerm) / denom;
	return 2;
}

float integrateWindow(float x)
{
	float xsq = x*x;
	return sign(x) * (0.5 * xsq*xsq - xsq) + 0.5;  // parabolic window
	//return 0.5 * (1.0 - sign(x) * xsq);          // box window
}

mat2 getUnitLineMatrix(vec2 b1, vec2 b2)
{
	vec2 V = b2 - b1;
	float normV = length(V);
	V = V / (normV*normV);

	return mat2(V.x, -V.y, V.y, V.x);
}

ivec2 normalizedCoordToIntegerCell(vec2 ncoord)
{
	return clamp(ivec2(ncoord * oGridRect.zw), ivec2(0), oGridRect.zw - 1);
}

void updateClosestCrossing(in vec2 porig[3], mat2 M, inout float closest, ivec2 integerCell)
{
	vec2 p[3];
	for (int i=0; i<3; i++) {
		p[i] = M * porig[i];
	}

	vec2 t;
	int numT = getAxisIntersections(p[0].y, p[1].y, p[2].y, t);

	for (int i=0; i<2; i++) {
		if (i == numT) {
			break;
		}

		if (t[i] > 0.0 && t[i] < 1.0) {
			float posx = positionAt(p[0].x, p[1].x, p[2].x, t[i]);
			vec2 op = vec2(positionAt(porig[0].x, porig[1].x, porig[2].x, t[i]),
			               positionAt(porig[0].y, porig[1].y, porig[2].y, t[i]));
			op += oNormCoord;

			bool sameCell = normalizedCoordToIntegerCell(op) == integerCell;

			//if (posx > 0.0 && posx < 1.0 && posx < abs(closest)) {
			if (sameCell && abs(posx) < abs(closest)) {
				float derivy = tangentAt(p[0].y, p[1].y, p[2].y, t[i]);
				closest = (derivy < 0.0) ? -posx : posx;
			}
		}
	}
}

mat2 inverse(mat2 m)
{
	return mat2(m[1][1],-m[0][1], -m[1][0], m[0][0])
		/ (m[0][0]*m[1][1] - m[0][1]*m[1][0]);
}

void main()
{
	ivec2 integerCell = normalizedCoordToIntegerCell(oNormCoord);
	ivec2 indicesCoord = ivec2(oGridRect.xy + integerCell);
	vec2 cellMid = (integerCell + 0.5) / oGridRect.zw;

	mat2 initrot = inverse(mat2(dFdx(oNormCoord) * kPixelWindowSize, dFdy(oNormCoord) * kPixelWindowSize));

	float theta = pi/float(numSS);
	mat2 rotM = mat2(cos(theta), sin(theta), -sin(theta), cos(theta)); // note this is column major ordering

	ivec4 indices1 = ivec4(texelFetch(uGridAtlas, indicesCoord, 0) * 255.0);

	// The mid-inside flag is encoded by the order of the beziers indices.
	// See write_vgrid_cell_to_buffer() for details.
	bool midInside = indices1[0] > indices1[1];

	float midClosest = midInside ? -2.0 : 2.0;

	float firstIntersection[numSS];
	for (int ss=0; ss<numSS; ss++) {
		firstIntersection[ss] = 2.0;
	}

	float percent = 0.0;

	mat2 midTransform = getUnitLineMatrix(oNormCoord, cellMid);

	for (int bezierIndex=0; bezierIndex<4; bezierIndex++) {
		int coordIndex;

		//if (bezierIndex < 4) {
			coordIndex = indices1[bezierIndex];
		//} else {
		//	 if (!moreThanFourIndices) break;
		//	 coordIndex = indices2[bezierIndex-4];
		//}

		// Indices 0 and 1 are both "no bezier" -- see
		// write_vgrid_cell_to_buffer() for why.
		if (coordIndex < 2) {
			continue;
		}

		vec2 p[3];
		fetchBezier(coordIndex-2, p);

		updateClosestCrossing(p, midTransform, midClosest, integerCell);

		// Transform p so fragment in glyph space is a unit circle
		for (int i=0; i<3; i++) {
			p[i] = initrot * p[i];
		}

		// Iterate through angles
		for (int ss=0; ss<numSS; ss++) {
			vec2 t;
			int numT = getAxisIntersections(p[0].x, p[1].x, p[2].x, t);

			for (int tindex=0; tindex<2; tindex++) {
				if (tindex == numT) break;

				if (t[tindex] > 0.0 && t[tindex] <= 1.0) {

					float derivx = tangentAt(p[0].x, p[1].x, p[2].x, t[tindex]);
					float posy = positionAt(p[0].y, p[1].y, p[2].y, t[tindex]);

					if (posy > -1.0 && posy < 1.0) {
						// Note: whether to add or subtract in the next statement is determined
						// by which convention the path uses: moving from the bezier start to end,
						// is the inside to the right or left?
						// The wrong operation will give buggy looking results, not a simple inverse.
						float delta = integrateWindow(posy);
						percent = percent + (derivx < 0.0 ? delta : -delta);

						float intersectDist = posy + 1.0;
						if (intersectDist < abs(firstIntersection[ss])) {
							firstIntersection[ss] = derivx < 0.0 ? -intersectDist : intersectDist;
						}
					}
				}
			}

			if (ss+1<numSS) {
				for (int i=0; i<3; i++) {
					p[i] = rotM * p[i];
				}
			}
		} // ss
	}

	bool midVal = midClosest < 0.0;

	// Add contribution from rays that started inside
	for (int ss=0; ss<numSS; ss++) {
		if ((firstIntersection[ss] >= 2.0 && midVal) || (firstIntersection[ss] > 0.0 && abs(firstIntersection[ss]) < 2.0)) {
			percent = percent + 1.0 /*integrateWindow(-1.0)*/;
		}
	}

	percent = percent / float(numSS);
	outColor = oColor;
	outColor.a *= percent;
}
)";

static GLuint loadShaderProgram(const char *vsCodeC, const char *fsCodeC)
{
	// Compile vertex shader
	GLuint vertexShaderId = glCreateShader(GL_VERTEX_SHADER);
	glShaderSource(vertexShaderId, 1, &vsCodeC, NULL);
	glCompileShader(vertexShaderId);

	GLint result = GL_FALSE;
	int infoLogLength = 0;
	glGetShaderiv(vertexShaderId, GL_COMPILE_STATUS, &result);
	glGetShaderiv(vertexShaderId, GL_INFO_LOG_LENGTH, &infoLogLength);
	if (infoLogLength > 1)
	{
		std::vector<char> infoLog(infoLogLength + 1);
		glGetShaderInfoLog(vertexShaderId, infoLogLength, NULL, &infoLog[0]);
		std::cerr << "[Vertex] " << &infoLog[0] << "\n";
	}
	if (!result)
	{
		return 0;
	}

	// Compile fragment shader
	GLuint fragmentShaderId = glCreateShader(GL_FRAGMENT_SHADER);
	glShaderSource(fragmentShaderId, 1, &fsCodeC, NULL);
	glCompileShader(fragmentShaderId);

	result = GL_FALSE, infoLogLength = 0;
	glGetShaderiv(fragmentShaderId, GL_COMPILE_STATUS, &result);
	glGetShaderiv(fragmentShaderId, GL_INFO_LOG_LENGTH, &infoLogLength);
	if (infoLogLength > 1)
	{
		std::vector<char> infoLog(infoLogLength);
		glGetShaderInfoLog(fragmentShaderId, infoLogLength, NULL, &infoLog[0]);
		std::cerr << "[Fragment] " << &infoLog[0] << "\n";
	}
	if (!result)
	{
		return 0;
	}

	// Link the program
	GLuint programId = glCreateProgram();
	glAttachShader(programId, vertexShaderId);
	glAttachShader(programId, fragmentShaderId);
	glLinkProgram(programId);

	result = GL_FALSE, infoLogLength = 0;
	glGetProgramiv(programId, GL_LINK_STATUS, &result);
	glGetProgramiv(programId, GL_INFO_LOG_LENGTH, &infoLogLength);
	if (infoLogLength > 1)
	{
		std::vector<char> infoLog(infoLogLength + 1);
		glGetProgramInfoLog(programId, infoLogLength, NULL, &infoLog[0]);
		std::cerr << "[Shader Linker] " << &infoLog[0] << "\n";
	}
	if (!result)
	{
		return 0;
	}

	glDetachShader(programId, vertexShaderId);
	glDetachShader(programId, fragmentShaderId);

	glDeleteShader(vertexShaderId);
	glDeleteShader(fragmentShaderId);

	return programId;
}

void render(GLLabel *label, float time, glm::mat4 transform)
{
	glUseProgram(3);
	glUniformMatrix4fv(1, 1, GL_FALSE, glm::value_ptr(transform));

	glBindBuffer(GL_TEXTURE_BUFFER, label->manager->atlases[0].glyphDataBufId);
	glBufferData(GL_TEXTURE_BUFFER, sq(kBezierAtlasSize) * kAtlasChannels,
				 label->manager->atlases[0].glyphDataBuf, GL_STREAM_DRAW);

	glBindTexture(GL_TEXTURE_2D, label->manager->atlases[0].gridAtlasId);
	glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, kGridAtlasSize, kGridAtlasSize, 0, GL_RGBA, GL_UNSIGNED_BYTE, label->manager->atlases[0].gridAtlas);

	glActiveTexture(GL_TEXTURE0);
	glBindTexture(GL_TEXTURE_2D, label->manager->atlases[0].gridAtlasId);
	glActiveTexture(GL_TEXTURE1);
	glBindTexture(GL_TEXTURE_BUFFER, label->manager->atlases[0].glyphDataBufTexId);

	// Label->Render(time, transform);

	glEnable(GL_BLEND);
	glBindBuffer(GL_ARRAY_BUFFER, label->vertBuffer);
	glEnableVertexAttribArray(0);
	glEnableVertexAttribArray(1);
	glEnableVertexAttribArray(2);
	glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, sizeof(GLLabel::GlyphVertex), (void *)offsetof(GLLabel::GlyphVertex, pos));
	glVertexAttribIPointer(1, 1, GL_UNSIGNED_INT, sizeof(GLLabel::GlyphVertex), (void *)offsetof(GLLabel::GlyphVertex, data));
	glVertexAttribPointer(2, 4, GL_UNSIGNED_BYTE, GL_TRUE, sizeof(GLLabel::GlyphVertex), (void *)offsetof(GLLabel::GlyphVertex, color));

	glDrawArrays(GL_TRIANGLES, 0, label->verts.size());

	glDisableVertexAttribArray(0);
	glDisableVertexAttribArray(1);
	glDisableVertexAttribArray(2);
	glDisable(GL_BLEND);
}

int main()
{

	// Create a window
	if (!glfwInit())
	{
		std::cerr << "Failed to initialize GLFW.\n";
		return -1;
	}

	glfwWindowHint(GLFW_SAMPLES, 8);
	glfwWindowHint(GLFW_DEPTH_BITS, 0);
	glfwWindowHint(GLFW_STENCIL_BITS, 0);
	glfwWindowHint(GLFW_ALPHA_BITS, 8);
	glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
	glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
	glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
	glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

	GLFWwindow *window = glfwCreateWindow(width, height, "Vector-Based GPU Text Rendering", NULL, NULL);
	if (!window)
	{
		std::cerr << "Failed to create GLFW window.\n";
		glfwTerminate();
		return -1;
	}

	glfwSetScrollCallback(window, onScroll);
	glfwSetWindowSizeCallback(window, onResize);

	// Create OpenGL context
	glfwMakeContextCurrent(window);
	glewExperimental = true;
	if (glewInit() != GLEW_OK)
	{
		std::cerr << "Failed to initialize GLEW.\n";
		glfwDestroyWindow(window);
		glfwTerminate();
		return -1;
	}

	std::cout << "GL Version: " << glGetString(GL_VERSION) << "\n";

	GLuint vertexArrayId;
	glGenVertexArrays(1, &vertexArrayId);
	glBindVertexArray(vertexArrayId);

	glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
	glEnable(GL_BLEND);

	loadShaderProgram(kGlyphVertexShader, kGlyphFragmentShader);
	glUseProgram(3);

	GLuint uGridAtlas = glGetUniformLocation(3, "uGridAtlas");
	GLuint uGlyphData = glGetUniformLocation(3, "uGlyphData");
	GLuint uTransform = glGetUniformLocation(3, "uTransform");

	std::cout << uTransform << std::endl;

	glUniform1i(uGridAtlas, 0);
	glUniform1i(uGlyphData, 1);

	glm::mat4 iden = glm::mat4(1.0);
	glUniformMatrix4fv(uTransform, 1, GL_FALSE, glm::value_ptr(iden));

	// Create new label
	Label = new GLLabel();

	std::cout << "Loading font files\n";

	Label->SetText(U"O", glm::vec4(0.5, 0, 0, 1));

	glGenBuffers(1, &Label->vertBuffer);
	glBindBuffer(GL_ARRAY_BUFFER, Label->vertBuffer);
	glBufferData(GL_ARRAY_BUFFER, Label->verts.capacity() * sizeof(GLLabel::GlyphVertex), NULL, GL_DYNAMIC_DRAW);
	glBufferSubData(GL_ARRAY_BUFFER, 0, Label->verts.size() * sizeof(GLLabel::GlyphVertex), &Label->verts[0]);

	GLFontManager::AtlasGroup& group = Label->manager->atlases[0];
	glGenBuffers(1, &group.glyphDataBufId);
	glBindBuffer(GL_TEXTURE_BUFFER, group.glyphDataBufId);
	glGenTextures(1, &group.glyphDataBufTexId);
	glBindTexture(GL_TEXTURE_BUFFER, group.glyphDataBufTexId);
	glTexBuffer(GL_TEXTURE_BUFFER, GL_RGBA8, group.glyphDataBufId);

	glGenTextures(1, &group.gridAtlasId);
	glBindTexture(GL_TEXTURE_2D, group.gridAtlasId);
	glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, kGridAtlasSize, kGridAtlasSize, 0, GL_RGBA, GL_UNSIGNED_BYTE, group.gridAtlas);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
	glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_R, GL_CLAMP_TO_EDGE);

	// GLLabel fpsLabel;
	// fpsLabel.SetText(toUTF32("FPS:"), glm::vec4(0,0,0,1), defaultFace);

	std::cout << "Starting render\n";

	int fpsFrame = 0;
	double fpsStartTime = glfwGetTime();
	while (!glfwWindowShouldClose(window))
	{
		float time = glfwGetTime();

		glClearColor(160 / 255.0, 169 / 255.0, 175 / 255.0, 1.0);
		glClear(GL_COLOR_BUFFER_BIT);

		glm::vec3 userTranslation(horizontalTransform, verticalTransform, 0);
		glm::vec3 userScale(scale, scale, 1.0);

		glm::mat4 textMat(1.0);
		textMat = glm::scale(textMat, userScale);
		textMat = glm::translate(textMat, userTranslation);
		if (spin)
		{
			textMat = glm::rotate(textMat, time / 3, glm::vec3(0.0, 0.0, 1.0));
			textMat = glm::scale(textMat, glm::vec3(sin(time) * 2, cos(time), 1.0));
		}
		textMat = glm::scale(textMat, pt(8));

		render(Label, time, textMat);

		// Window size might change, so recalculate this (and other pt() calls)
		glm::mat4 fpsMat(1.0);
		fpsMat = glm::scale(fpsMat, userScale);
		fpsMat = glm::translate(fpsMat, userTranslation + glm::vec3(0, 0.2, 0));
		if (spin)
		{
			fpsMat = glm::translate(fpsMat, glm::vec3(0.1, 0, 0));
			fpsMat = glm::rotate(fpsMat, time * 4, glm::vec3(0, 0, 1));
			fpsMat = glm::translate(fpsMat, glm::vec3(-0.1, 0, 0));
		}
		fpsMat = glm::scale(fpsMat, pt(7));
		// fpsLabel.Render(time, fpsMat);

		glfwPollEvents();
		glfwSwapBuffers(window);
	}

	glDeleteProgram(3);

	// Exit
	glfwDestroyWindow(window);
	glfwTerminate();
	return 0;
}

static bool leftShift = false;
static bool rightShift = false;

void onScroll(GLFWwindow *, double deltaX, double deltaY)
{
	if (leftShift)
	{
		scale += 0.1 * deltaY;
		if (scale < 0.1)
		{
			scale = 0.1;
		}
	}
	else
	{
		horizontalTransform += 0.1 * deltaX / scale;
		verticalTransform -= 0.1 * deltaY / scale;
	}
}

void onResize(GLFWwindow *, int w, int h)
{
	width = w;
	height = h;
	glViewport(0, 0, w, h);
}

std::u32string toUTF32(const std::string &s)
{
	std::wstring_convert<std::codecvt_utf8<char32_t>, char32_t> conv;
	return conv.from_bytes(s);
}

// Converts font points into a glm::vec3 scalar.
static glm::vec3 pt(float pt)
{
	static const float emUnits = 1.0 / 2048.0;
	const float aspect = (float)height / (float)width;

	float scale = emUnits * pt / 72.0;
	return glm::vec3(scale * aspect, scale, 0);
}
