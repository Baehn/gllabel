use std::{
    cmp::min,
    collections::{HashMap, HashSet},
};

use ordered_float::OrderedFloat;

use crate::{
    bezier::{Bezier2, Vec2},
    buffer::write_glyph_data_to_buffer,
};

const kBezierIndexUnused: u8 = 0;
const kBezierIndexFirstReal: u8 = 2;
const kBezierIndexSortMeta: u8 = 1;
pub static kGridAtlasSize: u16 = 256; // Fits exactly 1024 8x8 grids
const kAtlasChannels: u8 = 4; // Must be 4 (RGBA), otherwise code breaks
const kBezierAtlasSize: u16 = 256; // Fits around 700-1000 glyphs, depending on their curves
pub static kGridMaxSize: u8 = 20;

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
struct GlVertex {
    // XY coords of the vertex
    pos: Vec2,

    // Bit 0 (low) is norm coord X (varies per vertex)
    // Bit 1 is norm coord Y (varies per vertex)
    // Bits 2-31 are texel offset (byte offset / 4) into
    //   glyphDataBuf (same for all verticies of a glyph)
    data: u32,

    // RGBA color [0,255]
    color: [u8; 4],
}

#[derive(Default)]
struct Glyph {
    size: [u16; 2],           // Width and height in FT units
    offset: [i16; 2],         // Offset of glyph in FT units
    bezierAtlasPos: [u16; 2], // XZ pixel coordinates (Z being atlas index)
    advance: i16,             // Amount to advance after character in FT units
}

struct Buffers {}

fn find_cells_intersections(
    beziers: &Vec<Bezier2>,
    glyph_size: &Vec2,
    grid_width: u32,
    grid_height: u32,
) -> Vec<HashSet<usize>> {
    let mut ret: Vec<HashSet<usize>> = Vec::new();
    ret.resize((grid_width * grid_height) as usize, HashSet::new());

    let mut setgrid = |x: f32, y: f32, bezier_index: usize| {
        let x = x.clamp(0.0, grid_width as f32 - 1.0) as u32;
        let y = y.clamp(0.0, grid_height as f32 - 1.0) as u32;
        let i = (y * grid_width) + x;
        (&mut ret[i as usize]).insert(bezier_index);
    };

    for (i, bezier) in beziers.iter().enumerate() {
        let mut any_intersections = false;

        // Every vertical grid line including edges
        for x in 0..=grid_width {
            let int_y = bezier.intersect_vert((x as f32) * glyph_size.x / grid_width as f32);
            for y in int_y.iter() {
                let y = y * grid_height as f32 / glyph_size.y;
                setgrid(x as f32, y, i); // right
                setgrid(x as f32 - 1.0, y, i); // left
                any_intersections = true;
            }
        }

        // Every horizontal grid line including edges
        for y in 0..=grid_height {
            let int_x = bezier.intersect_horz((y as f32) * glyph_size.y / grid_height as f32);
            for x in int_x.iter() {
                let x = x * grid_width as f32 / glyph_size.x;
                setgrid(x, y as f32, i); // right
                setgrid(x, y as f32 - 1.0, i); // left
                any_intersections = true;
            }
        }

        // If no grid line intersections, bezier is fully contained in
        // one cell. Mark this bezier as intersecting that cell.
        if !any_intersections {
            let x = beziers[i].e0.x * grid_width as f32 / glyph_size.x;
            let y = beziers[i].e0.y * grid_height as f32 / glyph_size.y;
            setgrid(x, y, i);
        }
    }

    ret
}

fn find_cells_mids_inside(
    beziers: &Vec<Bezier2>,
    glyph_size: &Vec2,
    grid_width: u32,
    grid_height: u32,
) -> Vec<bool> {
    let mut cell_mids = Vec::new();
    cell_mids.resize((grid_width * grid_height) as usize, false);

    // // Find whether the center of each cell is inside the glyph
    for y in 0..grid_height {
        // Find all intersections with cells horizontal midpoint line
        // and store them sorted from left to right
        let mut intersections = HashSet::new();
        let y_mid = y as f32 + 0.5;
        for b in beziers.iter() {
            // 		float intX[2];
            let int_x = b.intersect_horz(y_mid * glyph_size.y as f32 / grid_height as f32);
            for x in int_x.iter() {
                let x = x * grid_width as f32 / glyph_size.x;
                intersections.insert(OrderedFloat(x));
            }
        }
        let mut intersections = intersections.into_iter().collect::<Vec<_>>();
        intersections.sort();

        // Traverse intersections (whole grid row, left to right).
        // Every 2nd crossing represents exiting an "inside" region.
        // All properly formed glyphs should have an even number of
        // crossings.
        let mut outside = false;
        let mut start: f32 = 0.0;
        for end in intersections.iter() {
            let end = end.0;

            // println!("{}", end);
            // Upon exiting, the midpoint of every cell between
            // start and end, rounded to the nearest int, is
            // inside the glyph.
            if outside {
                // let startCell = clamp((int)std::round(start), 0, gridWidth);
                // int endCell = clamp((int)std::round(end), 0, gridWidth);
                let start_cell = start.round().clamp(0.0, grid_width as f32) as usize;
                let end_cell = end.round().clamp(0.0, grid_width as f32) as usize;
                // println!("{}, {}", start_cell, end_cell);
                for x in start_cell..end_cell {
                    cell_mids[((y * grid_width) as usize + x) as usize] = true;
                }
            }

            outside = !outside;
            start = end;
        }
    }

    cell_mids
}

struct VGrid {
    width: u16,
    height: u16,
    cellBeziers: Vec<HashSet<usize>>,
    cellMids: Vec<bool>,
}

impl VGrid {
    pub fn from(curves: &Vec<Bezier2>, glyph_size: &Vec2, width: u32, height: u32) -> VGrid {
        let cellBeziers = find_cells_intersections(curves, glyph_size, width, height);
        let cellMids = find_cells_mids_inside(curves, glyph_size, width, height);
        VGrid {
            cellBeziers,
            cellMids,
            width: width as u16,
            height: height as u16,
        }
    }

    fn write_vgrid_at(&self, at_x: u16, at_y: u16, data: &mut [u8]) {
        self._write_vgrid_at(
            at_x,
            at_y,
            data,
            kGridAtlasSize,
            kGridAtlasSize,
            kAtlasChannels,
        )
    }

    fn _write_vgrid_at(
        &self,
        at_x: u16,
        at_y: u16,
        data: &mut [u8],
        width: u16,
        height: u16,
        depth: u8,
    ) {
        assert!((at_x + self.width) <= width);
        assert!((at_y + self.height) <= height);

        for y in 0..self.height {
            for x in 0..self.width {
                let cell_idx = xy2i(x, y, self.width);
                let atlas_idx = xy2i(at_x + x, at_y + y, width) * depth as usize;

                let beziers = &self.cellBeziers[cell_idx];
                if beziers.len() > depth as usize {
                    panic!("WARN: Too many beziers in one grid cell")
                }
                for i in 0..depth {
                    data[atlas_idx + i as usize] = 100;
                }
                write_vgrid_cell_to_buffer(
                    &self.cellBeziers,
                    &self.cellMids,
                    cell_idx,
                    &mut data[atlas_idx..atlas_idx + depth as usize],
                );
            }
        }
    }
}

fn xy2i(x: u16, y: u16, w: u16) -> usize {
    (y as usize * w as usize) + x as usize
}

fn write_vgrid_cell_to_buffer(
    cell_beziers: &Vec<HashSet<usize>>,
    cell_mids: &Vec<bool>,
    cell_idx: usize,
    data: &mut [u8],
) {
    let beziers = &cell_beziers[cell_idx];
    let mut beziers: Vec<&usize> = beziers.into_iter().collect();
    beziers.sort();

    // Clear texel
    for item in data.iter_mut() {
        *item = kBezierIndexUnused;
    }

    // Write out bezier indices to atlas texel
    let mut i: usize = 0;
    let nbeziers = min(beziers.len(), data.len());
    for it in beziers.iter().take(nbeziers) {
        // TODO: The uint8_t cast wont overflow because the bezier
        // limit is checked when loading the glyph. But try to encode
        // that info into the data types so no cast is needed.
        data[i] = **it as u8 + kBezierIndexFirstReal;
        i += 1;
    }

    let mid_inside = cell_mids[cell_idx];

    // Because the order of beziers doesn't matter and a single bezier is
    // never referenced twice in one cell, metadata can be stored by
    // adjusting the order of the bezier indices. In this case, the
    // midInside bit is 1 if data[0] > data[1].
    // Note that the bezier indices are already sorted from smallest to
    // largest because of std::set.
    if mid_inside {
        // If cell is empty, there's nothing to swap (both values 0).
        // So a fake "sort meta" value must be used to make data[0]
        // be larger. This special value is treated as 0 by the shader.
        if beziers.len() == 0 {
            data[0] = kBezierIndexSortMeta;
        }
        // If there's just one bezier, data[0] is always > data[1] so
        // nothing needs to be done. Otherwise, swap data[0] and [1].
        else if beziers.len() != 1 {
            let tmp = data[0];
            data[0] = data[1];
            data[1] = tmp;
        }
    // If midInside is 0, make sure that data[0] <= data[1]. This can only
    // not happen if there is only 1 bezier in this cell, for the reason
    // described above. Solve by moving the only bezier into data[1].
    } else if beziers.len() == 1 {
        data[1] = data[0];
        data[0] = kBezierIndexUnused;
    }
}

fn insert_curves(
    verts: &mut Vec<GlVertex>,
    curves: &Vec<Bezier2>,
    glyphDataBuf: &mut [u16],
    gridAtlas: &mut [u8],
) {
    // this->glyphs.resize(text.size());

    // GlyphVertex emptyVert{};
    // this->verts.insert(this->verts.begin() + index * 6, text.size() * 6, emptyVert);

    // glm::vec2 appendOffset(0, 0);

    let i = 0;
    let index = 0;
    let color: [f32; 4] = [0.5, 0.0, 0.0, 1.0];

    // for (size_t i = 0; i < text.size(); i++)
    // {
    let glyph = get_glyph_for_codepoint(&curves, glyphDataBuf, gridAtlas);

    let mut vs = [GlVertex::default(); 6]; // Insertion code depends on v[0] equaling appendOffset (therefore it is also set before continue;s above)
    vs[0].pos = Vec2::from(0.0, 0.0);
    vs[1].pos = Vec2::from(glyph.size[0] as f32, 0.0);
    vs[2].pos = Vec2::from(0.0, glyph.size[1] as f32);
    vs[3].pos = Vec2::from(glyph.size[0] as f32, glyph.size[1] as f32);
    vs[4].pos = Vec2::from(0.0, glyph.size[1] as f32);
    vs[5].pos = Vec2::from(glyph.size[0] as f32, 0.0);
    for (j, mut v) in vs.iter_mut().enumerate() {
        // 		v[j].pos += appendOffset;
        v.pos.x += glyph.offset[0] as f32;
        v.pos.y += glyph.offset[1] as f32;

        v.color = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];

        // 		// Encode both the bezier position and the norm coord into one int
        // 		// This theoretically could overflow, but the atlas position will
        // 		// never be over half the size of a uint16, so it's fine.
        let k = (if j < 4 { j } else { 6 - j }) as u32;
        let normX = k & 1;
        let normY = if k > 1 { 1_u32 } else { 0_u32 };
        let norm = (normX << 1) + normY;
        v.data = ((glyph.bezierAtlasPos[0] as u32) << 2) + norm;
        // verts[(index + i) * 6 + j] = v.clone();
        verts.push(v.clone());
    }

    // 	appendOffset.x += glyph->advance;
    // 	this->glyphs[index + i] = glyph;
    // }
}

fn get_glyph_for_codepoint(
    curves: &Vec<Bezier2>,
    glyphDataBuf: &mut [u16],
    gridAtlas: &mut [u8],
) -> Glyph {
    // AtlasGroup *atlas = this->GetOpenAtlasGroup();

    let glyph_width = 1398;
    let glyph_height = 1450;

    let hori_bearing_x = 97;
    let hori_bearing_y = 1430;
    let hori_advance = 1593;

    let grid_width = kGridMaxSize;
    let grid_height = kGridMaxSize;
    let glyph_size = Vec2::from(glyph_width as f32, glyph_height as f32);

    let grid = VGrid::from(&curves, &glyph_size, grid_width as u32, grid_height as u32);

    // Although the data is represented as a 32bit texture, it's actually
    // two 16bit ints per pixel, each with an x and y coordinate for
    // the bezier. Every six 16bit ints (3 pixels) is a full bezier
    // Plus two pixels for grid position information
    let bezierPixelLength = 2 + curves.len() * 3;

    // uint8_t *bezierData = atlas->glyphDataBuf + (atlas->glyphDataBufOffset * kAtlasChannels);

    write_glyph_data_to_buffer(
        glyphDataBuf,
        &curves,
        &glyph_size,
        0,
        0,
        kGridMaxSize as u16,
        kGridMaxSize as u16,
    );

    // // TODO: Integrate with AtlasGroup / replace AtlasGroup
    // WriteVGridAt(grid, atlas->nextGridPos[0], atlas->nextGridPos[1], atlas->gridAtlas, kGridAtlasSize, kGridAtlasSize, kAtlasChannels);
    grid.write_vgrid_at(0, 0, gridAtlas);

    let mut glyph = Glyph::default();
    glyph.bezierAtlasPos[0] = 0;
    glyph.bezierAtlasPos[1] = 0;
    glyph.size[0] = glyph_width;
    glyph.size[1] = glyph_height;
    glyph.offset[0] = hori_bearing_x;
    glyph.offset[1] = hori_bearing_y as i16 - glyph_height as i16;
    glyph.advance = hori_advance;

    glyph
}

#[cfg(test)]
mod test {
    use std::{iter::zip, mem::size_of};

    use crate::{
        bezier::{Bezier2, Vec2},
        grid::{find_cells_mids_inside, kBezierAtlasSize},
        test_data::test_data::test_curves,
    };

    use super::{
        find_cells_intersections, insert_curves, kAtlasChannels, kGridAtlasSize, GlVertex, VGrid,
    };

    #[test]
    fn test_find_cells_intersections_test_curves() {
        let curves = test_curves();
        let ret = find_cells_intersections(&curves, &Vec2::from(1398.0, 1450.0), 20, 20);
        let ret: Vec<Vec<usize>> = ret
            .iter()
            .map(|v| v.into_iter().cloned().collect())
            .collect();
        let mut sorted = Vec::new();
        for mut v in ret {
            v.sort();
            sorted.extend(v);
        }
        let exp: Vec<usize> = vec![
            3, 3, 3, 3, 2, 3, 2, 2, 2, 2, 2, 4, 3, 4, 3, 2, 1, 2, 1, 4, 4, 16, 16, 16, 16, 17, 17,
            17, 17, 17, 1, 1, 4, 4, 16, 16, 16, 17, 17, 17, 1, 1, 4, 5, 15, 16, 17, 18, 0, 1, 5, 5,
            15, 15, 18, 18, 0, 0, 5, 15, 18, 0, 5, 15, 15, 18, 18, 0, 5, 15, 18, 0, 5, 15, 18, 0,
            5, 6, 14, 15, 11, 18, 0, 10, 6, 14, 11, 10, 6, 14, 14, 11, 11, 10, 6, 14, 11, 10, 6,
            14, 14, 11, 11, 10, 10, 6, 6, 13, 14, 11, 12, 9, 10, 6, 6, 13, 13, 13, 12, 12, 12, 9,
            9, 6, 7, 7, 13, 13, 13, 13, 12, 13, 12, 12, 12, 9, 9, 7, 7, 8, 8, 9, 9, 7, 7, 7, 7, 7,
            7, 7, 8, 8, 8, 8, 8,
        ];
        assert_eq!(sorted, exp);
    }

    #[test]
    fn test_find_cells_mids_inside() {
        let curves = test_curves();
        let ret = find_cells_mids_inside(&curves, &Vec2::from(1398.0, 1450.0), 20, 20);
        let exp: Vec<usize> = vec![
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0,
            0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
            1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0,
            0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
        ];
        let exp: Vec<bool> = exp.iter().map(|e| e > &0).into_iter().collect();
        assert_eq!(ret, exp);
    }

    #[test]
    fn test_find_cells_intersections() {
        let curve = Bezier2::from(1398.0, 731.0, 1313.0, 344.0, 1398.0, 510.0);
        let curves = vec![curve];
        let ret = find_cells_intersections(&curves, &Vec2::from(1398.0, 1450.0), 20, 20);
        let ret: Vec<usize> = ret.into_iter().flatten().collect();
        assert_eq!(ret, vec![0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_write_vgrid_at() {
        let curves = test_curves();
        let grid = VGrid::from(&curves, &Vec2::from(1398.0, 1450.0), 20, 20);
        let n = kGridAtlasSize as usize * kGridAtlasSize as usize * kAtlasChannels as usize;
        let mut data = Vec::new();
        data.resize(n as usize, 0);
        grid.write_vgrid_at(0, 0, &mut data);
        let exp: [u8; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 5, 0, 0, 0, 5,
            0, 0, 0, 5, 0, 0, 0, 5, 4, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 4,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(data[0..256], exp);
    }

    #[test]
    fn test_insert_curves() {
        let mut glyph_data_buf = Vec::new();
        glyph_data_buf.resize(
            kBezierAtlasSize as usize * kBezierAtlasSize as usize * (kAtlasChannels / 2) as usize,
            0,
        );

        let mut grid_atlas = Vec::new();
        grid_atlas.resize(
            kGridAtlasSize as usize * kGridAtlasSize as usize * kAtlasChannels as usize,
            0,
        );
        let curves = test_curves();
        let mut verts = Vec::new();
        insert_curves(&mut verts, &curves, &mut glyph_data_buf, &mut grid_atlas);

        //     let ptr = verts.as_ptr() as u8;
        //     assert_eq!(ptr,)
        // let mut vec = unsafe {
        //     assert!(!array.is_null());
        //     Vec::from_raw_parts(array, size as usize, size as usize)
        let (ptr, size) = ffi_utils::vec_into_raw_parts(verts);
        let vec: Vec<u8> = unsafe {
            let size = size * size_of::<GlVertex>() as usize;
            Vec::from_raw_parts(ptr as *mut u8, size, size)
        };
        assert_eq!(
            vec,
            vec![
                0, 0, 194, 66, 0, 0, 160, 193, 0, 0, 0, 0, 127, 0, 0, 255, 0, 224, 186, 68, 0, 0,
                160, 193, 2, 0, 0, 0, 127, 0, 0, 255, 0, 0, 194, 66, 0, 192, 178, 68, 1, 0, 0, 0,
                127, 0, 0, 255, 0, 224, 186, 68, 0, 192, 178, 68, 3, 0, 0, 0, 127, 0, 0, 255, 0, 0,
                194, 66, 0, 192, 178, 68, 1, 0, 0, 0, 127, 0, 0, 255, 0, 224, 186, 68, 0, 0, 160,
                193, 2, 0, 0, 0, 127, 0, 0, 255,
            ]
        );
        assert_eq!(
            grid_atlas[0..256],
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 5, 0, 0, 0,
                5, 0, 0, 0, 5, 0, 0, 0, 5, 4, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
                0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        );
        let (ptr, size) = ffi_utils::vec_into_raw_parts(glyph_data_buf);
        let vec: Vec<u8> = unsafe {
            let size = size * size_of::<u8>() as usize;
            Vec::from_raw_parts(ptr as *mut u8, size, size)
        };
        assert_eq!(
            vec[0..256],
            [
                0, 0, 0, 0, 20, 0, 20, 0, 255, 255, 14, 129, 255, 255, 10, 90, 110, 240, 187, 60,
                110, 240, 187, 60, 12, 225, 108, 31, 29, 196, 182, 15, 29, 196, 182, 15, 47, 167,
                0, 0, 208, 127, 0, 0, 208, 127, 0, 0, 20, 88, 0, 0, 37, 59, 137, 15, 37, 59, 137,
                15, 101, 30, 18, 31, 50, 15, 97, 60, 50, 15, 97, 60, 0, 0, 221, 89, 0, 0, 14, 129,
                0, 0, 14, 129, 0, 0, 187, 188, 224, 33, 70, 222, 224, 33, 70, 222, 192, 67, 255,
                255, 46, 128, 255, 255, 46, 128, 255, 255, 141, 167, 255, 255, 123, 196, 208, 240,
                123, 196, 208, 240, 106, 225, 206, 225, 157, 240, 7, 197, 157, 240, 7, 197, 255,
                255, 64, 168, 255, 255, 14, 129, 73, 220, 14, 129, 73, 220, 125, 175, 29, 196, 248,
                201, 29, 196, 248, 201, 33, 172, 116, 228, 46, 128, 116, 228, 46, 128, 116, 228,
                221, 83, 116, 228, 178, 59, 83, 202, 178, 59, 83, 202, 134, 35, 50, 176, 134, 35,
                14, 129, 134, 35, 14, 129, 134, 35, 69, 82, 225, 59, 186, 54, 225, 59, 186, 54,
                106, 84, 93, 27, 208, 127, 93, 27, 208, 127, 93, 27, 126, 172, 93, 27, 76, 196,
                216, 53, 76, 196, 216, 53, 73, 220, 129, 80, 73, 220, 14, 129, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
