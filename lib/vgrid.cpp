#include "vgrid.hpp"
// #include <iostream>
#include <cmath>
#include <assert.h>
#include <stdexcept>
#include <iostream>
#include <bits/stdc++.h>

// Converts X,Y to index in a row-major 2D array
constexpr size_t xy2i(const size_t x, const size_t y, const size_t w)
{
	return (y * w) + x;
}

template <class T>
constexpr const T &clamp(const T &v, const T &min, const T &max)
{
	return std::max(std::min(v, max), min);
}

// Returns a list of the beziers that intersect each grid cell.
// The returned outer vector is always size gridWidth*gridHeight.
static std::vector<std::vector<size_t>> find_cells_intersections(
	std::vector<Bezier2> &beziers,
	Vec2 glyphSize,
	int gridWidth,
	int gridHeight)
{
	std::vector<std::set<size_t>> cellBeziers;
	cellBeziers.resize(gridWidth * gridHeight);

	auto setgrid = [&](int x, int y, size_t bezierIndex)
	{
		x = clamp(x, 0, gridWidth - 1);
		y = clamp(y, 0, gridHeight - 1);
		cellBeziers[(y * gridWidth) + x].insert(bezierIndex);
	};

	for (size_t i = 0; i < beziers.size(); i++)
	{
		bool anyIntersections = false;

		// Every vertical grid line including edges
		for (int x = 0; x <= gridWidth; x++)
		{
			float intY[2];
			int numInt = beziers[i].IntersectVert(
				x * glyphSize.w / gridWidth,
				intY);

			for (int j = 0; j < numInt; j++)
			{
				int y = intY[j] * gridHeight / glyphSize.h;
				setgrid(x, y, i);	  // right
				setgrid(x - 1, y, i); // left
				anyIntersections = true;
			}
		}

		// Every horizontal grid line including edges
		for (int y = 0; y <= gridHeight; y++)
		{
			float intX[2];
			int numInt = beziers[i].IntersectHorz(
				y * glyphSize.h / gridHeight,
				intX);
			for (int j = 0; j < numInt; j++)
			{
				int x = intX[j] * gridWidth / glyphSize.w;
				setgrid(x, y, i);	  // up
				setgrid(x, y - 1, i); // down
				anyIntersections = true;
			}
		}

		// If no grid line intersections, bezier is fully contained in
		// one cell. Mark this bezier as intersecting that cell.
		if (!anyIntersections)
		{
			int x = beziers[i].e0.x * gridWidth / glyphSize.w;
			int y = beziers[i].e0.y * gridHeight / glyphSize.h;
			setgrid(x, y, i);
		}

		
	}

	std::vector<std::vector<size_t>> ret;
	ret.resize(gridWidth * gridHeight);

	for (int i = 0; i < cellBeziers.size(); i++)
	{
		auto s = cellBeziers[i];
		std::vector<size_t> v(s.begin(), s.end());
		ret[i] = v;
	}

	return ret;
}

// Returns whether the midpoint of the cell is inside the glyph for each cell.
// The returned vector is always size gridWidth*gridHeight.
static std::vector<char> find_cells_mids_inside(
	std::vector<Bezier2> &beziers,
	Vec2 glyphSize,
	int gridWidth,
	int gridHeight)
{
	std::vector<char> cellMids;
	cellMids.resize(gridWidth * gridHeight);

	// Find whether the center of each cell is inside the glyph
	for (int y = 0; y < gridHeight; y++)
	{
		// Find all intersections with cells horizontal midpoint line
		// and store them sorted from left to right
		std::set<float> intersections;
		float yMid = y + 0.5;
		for (size_t i = 0; i < beziers.size(); i++)
		{
			float intX[2];
			int numInt = beziers[i].IntersectHorz(
				yMid * glyphSize.h / gridHeight,
				intX);
			for (int j = 0; j < numInt; j++)
			{
				float x = intX[j] * gridWidth / glyphSize.w;
				intersections.insert(x);
			}
		}

		// Traverse intersections (whole grid row, left to right).
		// Every 2nd crossing represents exiting an "inside" region.
		// All properly formed glyphs should have an even number of
		// crossings.
		bool outside = false;
		float start = 0;
		for (auto it = intersections.begin(); it != intersections.end(); it++)
		{
			float end = *it;

				// std::cout << end << std::endl;

			// Upon exiting, the midpoint of every cell between
			// start and end, rounded to the nearest int, is
			// inside the glyph.
			if (outside)
			{
				int startCell = clamp((int)std::round(start), 0, gridWidth);
				int endCell = clamp((int)std::round(end), 0, gridWidth);
				// std::cout << startCell << "," << endCell << std::endl;
				for (int x = startCell; x < endCell; x++)
				{
					cellMids[(y * gridWidth) + x] = true;
				}
			}

			outside = !outside;
			start = end;
		}

	}
			// std::cout << "vert: " << i << std::endl;
			// // << " :";
			// // std::cout << i << " ";
			// auto b = beziers[i];
			// std::cout << "b: " << b.e0.x << ", " << b.e0.y << ", " << b.e1.x << ", " << b.e1.y << ", " << b.c.x << ", " << b.c.y << ", " << std::endl;
			// // std::cout << x * glyphSize.w / gridWidth << std::endl;
			// // for (int j = 0; j < numInt; j++)
			// // {
			// // 	std::cout << intY[j] << ", ";
			// // }
			// // std::cout << std::endl;
			// std::cout << gridWidth << std::endl;
			// std::cout << gridHeight << std::endl;
			// std::cout << glyphSize.x << std::endl;
			// std::cout << glyphSize.y << std::endl;

			// for (int i = 0; i < cellBeziers.size(); i++)
			// {
			// 	auto s = cellBeziers[i];
			// 	std::vector<size_t> v(s.begin(), s.end());

			// 	sort(v.begin(), v.end());

			// 	for (int j = 0; j < v.size(); j++)
			// 	{
			// 		std::cout << v[j] << ", ";
			// 	}
			// }
			// std::cout << std::endl;
	
	// 		std::cout << gridWidth << std::endl;
	// 		std::cout << gridHeight << std::endl;
	// 		std::cout << glyphSize.x << std::endl;
	// 		std::cout << glyphSize.y << std::endl;
	// for(int i = 0; i< cellMids.size(); i++){
	// 	std::cout << (bool)cellMids[i] << ", ";
	// }
	// std::cout << std::endl;

	return cellMids;
}

VGrid::VGrid(
	std::vector<Bezier2> &beziers,
	Vec2 glyphSize,
	int gridWidth,
	int gridHeight)
	: width(gridWidth), height(gridHeight)
{
	this->cellBeziers = find_cells_intersections(
		beziers, glyphSize, gridWidth, gridHeight);
	this->cellMids = find_cells_mids_inside(
		beziers, glyphSize, gridWidth, gridHeight);
}

// Each bezier index is represented as one byte in the grid cell,
// and values 0 and 1 are reserved for special meaning.
// This leaves a limit of 254 beziers per grid/glyph.
// More on the meaning of values 1 and 0 in the VGridAtlas struct
// definition and in write_vgrid_cell_to_buffer().
static const uint8_t kBezierIndexUnused = 0;
static const uint8_t kBezierIndexSortMeta = 1;
static const uint8_t kBezierIndexFirstReal = 2;
// static const uint8_t kMaxBeziersPerGrid = 256 - kBezierIndexFirstReal;

// Writes the data of a single vgrid cell into a texel. At most `depth` bytes
// will be written, even if there are more beziers.
static void write_vgrid_cell_to_buffer(
	std::vector<std::vector<size_t>> &cellBeziers,
	std::vector<char> &cellMids,
	size_t cellIdx, // which cell in `grid` to write
	uint8_t *data,	// texel buffer, `depth` bytes long
	uint8_t depth)
{
	std::vector<size_t> *beziers = &cellBeziers[cellIdx];

	// Clear texel
	for (uint8_t i = 0; i < depth; i++) {
		data[i] = kBezierIndexUnused;
	}

	// Write out bezier indices to atlas texel
	size_t i = 0;
	size_t nbeziers = std::min(beziers->size(), (size_t)depth);
	auto end = beziers->begin();
	std::advance(end, nbeziers);
	for (auto it = beziers->begin(); it != end; it++)
	{
		// TODO: The uint8_t cast wont overflow because the bezier
		// limit is checked when loading the glyph. But try to encode
		// that info into the data types so no cast is needed.
		data[i] = (uint8_t)(*it) + kBezierIndexFirstReal;
		i++;
	}

	bool midInside = cellMids[cellIdx];

	// Because the order of beziers doesn't matter and a single bezier is
	// never referenced twice in one cell, metadata can be stored by
	// adjusting the order of the bezier indices. In this case, the
	// midInside bit is 1 if data[0] > data[1].
	// Note that the bezier indices are already sorted from smallest to
	// largest because of std::set.
	if (midInside)
	{
		// If cell is empty, there's nothing to swap (both values 0).
		// So a fake "sort meta" value must be used to make data[0]
		// be larger. This special value is treated as 0 by the shader.
		if (beziers->size() == 0)
		{
			data[0] = kBezierIndexSortMeta;
		}
		// If there's just one bezier, data[0] is always > data[1] so
		// nothing needs to be done. Otherwise, swap data[0] and [1].
		else if (beziers->size() != 1)
		{
			uint8_t tmp = data[0];
			data[0] = data[1];
			data[1] = tmp;
		}
		// If midInside is 0, make sure that data[0] <= data[1]. This can only
		// not happen if there is only 1 bezier in this cell, for the reason
		// described above. Solve by moving the only bezier into data[1].
	}
	else if (beziers->size() == 1)
	{
		data[1] = data[0];
		data[0] = kBezierIndexUnused;
	}
}

// Writes an entire vgrid into the atlas, where the bottom-left of the vgrid
// will be written at (atX, atY). It will take up (grid->width, grid->height)
// atlas texels and overwrite all contents in that rectangle.
void WriteVGridAt(VGrid &grid, uint16_t atX, uint16_t atY, uint8_t *data, uint16_t width, uint16_t height, uint8_t depth)
{
	assert((atX + grid.width) <= width);
	assert((atY + grid.height) <= height);

			std::cout << grid.width << std::endl;
			std::cout << grid.height << std::endl;

	for (uint16_t y = 0; y < grid.height; y++)
	{
		for (uint16_t x = 0; x < grid.width; x++)
		{
			size_t cellIdx = xy2i(x, y, grid.width);
			// std::cout << cellIdx << std::endl;
			size_t atlasIdx = xy2i(atX + x, atY + y, width) * depth;

			std::vector<size_t> *beziers = &grid.cellBeziers[cellIdx];
			if (beziers->size() > depth)
			{
				// std::cerr << "WARN: Too many beziers in one grid cell ("
				// 	<< "max: " << this->depth
				// 	<< ", need: " << beziers->size()
				// 	<< ", x: " << x
				// 	<< ", y: " << y << ")\n";
				throw std::runtime_error("WARN: Too many beziers in one grid cell");
			}
			for (uint8_t i = 0; i < depth; i++)
			{
				(&data[atlasIdx])[i] = 100;
			}
			write_vgrid_cell_to_buffer(grid.cellBeziers, grid.cellMids, cellIdx, &data[atlasIdx], depth);
		}
	}
	// std::cout << std::endl;
	// for(int i = 0; i< 256; i++){
	// 	std::cout << (u_int16_t)data[i] << ", ";
	// }
	// std::cout << std::endl;
}
