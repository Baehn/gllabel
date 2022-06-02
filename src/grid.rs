use std::{
    cmp::min,
    collections::{HashMap, HashSet},
};

use ordered_float::OrderedFloat;

use crate::bezier::{Bezier2, Vec2};

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

impl VGrid{
    pub fn from(){
        VGrid {cellBeziers, cellMids, width, height}
    }
}

fn xy2i(x: u16, y: u16, w: u16) -> usize {
    (y as usize * w as usize) + x as usize
}

fn write_vgrid_at(
    grid: &VGrid,
    at_x: u16,
    at_y: u16,
    data: &mut [u8],
    width: u16,
    height: u16,
    depth: u8,
) {
    assert!((at_x + grid.width) <= width);
    assert!((at_y + grid.height) <= height);

    for y in 0..height {
        for x in 0..width {
            let cell_idx = xy2i(x, y, grid.width);
            let atlas_idx = xy2i(at_x + x, at_y + y, width) * depth as usize;

            let beziers = &grid.cellBeziers[cell_idx];
            if beziers.len() > depth as usize {
                panic!("WARN: Too many beziers in one grid cell")
            }
            for i in 0..depth {
                data[atlas_idx + i as usize] = 100;
            }
            // 			for (uint8_t i = 0; i < depth; i++)
            // 			{
            // 				(&data[atlasIdx])[i] = 100;
            // 			}
            write_vgrid_cell_to_buffer(
                &grid.cellBeziers,
                &grid.cellMids,
                cell_idx,
                &mut data[atlas_idx..atlas_idx + depth as usize],
            );
        }
    }
}

const kBezierIndexUnused: u8 = 0;
const kBezierIndexFirstReal: u8 = 2;
const kBezierIndexSortMeta: u8 = 1;
pub static kGridMaxSize: u8 = 20;

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

#[cfg(test)]
mod test {
    use std::iter::zip;

    use crate::{
        bezier::{Bezier2, Vec2},
        grid::find_cells_mids_inside,
        test_data::test_data::test_curves,
    };

    use super::{find_cells_intersections, VGrid};

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
        let grid = VGrid {};
    }
}
