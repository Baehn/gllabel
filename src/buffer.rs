use crate::bezier::{Bezier2, Vec2};

const UINT16_MAX: f32 = u16::max_value() as f32;

pub fn write_glyph_data_to_buffer(
    buffer: &mut [u16],
    beziers: &Vec<Bezier2>,
    glyph_size: &Vec2,
    grid_x: u16,
    grid_y: u16,
    grid_width: u16,
    grid_height: u16,
) {
    buffer[0] = grid_x;
    buffer[1] = grid_y;
    buffer[2] = grid_width;
    buffer[3] = grid_height;

    let mut i = 4;
    for bezier in beziers {
        buffer[i + 0] = (bezier.e0.x * UINT16_MAX / glyph_size.x) as u16;
        buffer[i + 1] = (bezier.e0.y * UINT16_MAX / glyph_size.y) as u16;
        buffer[i + 2] = (bezier.c.x * UINT16_MAX / glyph_size.x) as u16;
        buffer[i + 3] = (bezier.c.y * UINT16_MAX / glyph_size.y) as u16;
        buffer[i + 4] = (bezier.e1.x * UINT16_MAX / glyph_size.x) as u16;
        buffer[i + 5] = (bezier.e1.y * UINT16_MAX / glyph_size.y) as u16;
        i += 6;
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::kGridMaxSize;
    use crate::test_data::test_data::test_curves;
    use crate::{bezier::Vec2, buffer::write_glyph_data_to_buffer};

    #[test]
    pub fn test_write_glyph_data_to_buffer() {
        let mut data = vec![0 as u16; 200];
        let beziers = test_curves();
        let glyph_size = Vec2 {
            x: 1398.0,
            y: 1450.0,
        };

        write_glyph_data_to_buffer(
            &mut data,
            &beziers,
            &glyph_size,
            0,
            0,
            kGridMaxSize as u16,
            kGridMaxSize as u16,
        );
        assert_eq!(
            data,
            [
                0, 0, 20, 20, 65535, 33038, 65535, 23050, 61550, 15547, 61550, 15547, 57612, 8044,
                50205, 4022, 50205, 4022, 42799, 0, 32720, 0, 32720, 0, 22548, 0, 15141, 3977,
                15141, 3977, 7781, 7954, 3890, 15457, 3890, 15457, 0, 23005, 0, 33038, 0, 33038, 0,
                48315, 8672, 56902, 8672, 56902, 17344, 65535, 32814, 65535, 32814, 65535, 42893,
                65535, 50299, 61648, 50299, 61648, 57706, 57806, 61597, 50439, 61597, 50439, 65535,
                43072, 65535, 33038, 56393, 33038, 56393, 44925, 50205, 51704, 50205, 51704, 44065,
                58484, 32814, 58484, 32814, 58484, 21469, 58484, 15282, 51795, 15282, 51795, 9094,
                45106, 9094, 33038, 9094, 33038, 9094, 21061, 15329, 14010, 15329, 14010, 21610,
                7005, 32720, 7005, 32720, 7005, 44158, 7005, 50252, 13784, 50252, 13784, 56393,
                20609, 56393, 33038, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
    }
}
