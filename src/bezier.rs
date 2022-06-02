#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn from(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

#[repr(C)]
#[derive(Default, Clone)]
pub struct Bezier2 {
    pub e0: Vec2,
    pub e1: Vec2,
    pub c: Vec2,
}

#[inline]
fn almost_equal(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

#[inline]
fn t_valid(t: f32) -> bool {
    t <= 1.0 && t >= 0.0
}

#[inline]
fn x_from_t(t: f32, A: &Vec2, B: &Vec2, C: &Vec2) -> f32 {
    (1.0 - (t)) * (1.0 - (t)) * A.x + 2.0 * (t) * (1.0 - (t)) * B.x + (t) * (t) * C.x
}

impl Bezier2 {
    pub fn from(x1: f32, x2: f32, x3: f32, x4: f32, x5: f32, x6: f32) -> Bezier2 {
        Bezier2 {
            e0: Vec2 { x: x1, y: x2 },
            e1: Vec2 { x: x3, y: x4 },
            c: Vec2 { x: x5, y: x6 },
        }
    }
    pub fn intersect_vert(&self, x: f32) -> Vec<f32> {
        let inverse = Bezier2 {
            e0: Vec2 {
                x: self.e0.y,
                y: self.e0.x,
            },
            e1: Vec2 {
                x: self.e1.y,
                y: self.e1.x,
            },
            c: Vec2 {
                x: self.c.y,
                y: self.c.x,
            },
        };
        inverse.intersect_horz(x)
    }

    pub fn intersect_horz(&self, y: f32) -> Vec<f32> //{//, float outX[2])
    {
        let a = &self.e0;
        let b = &self.c;
        let c = &self.e1;
        let mut ret = Vec::new();

        // Parts of the bezier function solved for t
        let u = a.y - 2.0 * b.y + c.y;

        // In the condition that a=0, the standard formulas won't work
        if almost_equal(u, 0.0) {
            let t = (2.0 * b.y - c.y - y) / (2.0 * (b.y - c.y));
            if t_valid(t) {
                ret.push(x_from_t(t, a, b, c));
            }
            return ret;
        }

        let sqrt_term = (y * u + b.y * b.y - a.y * c.y).sqrt();

        let t = (a.y - b.y + sqrt_term) / u;
        if t_valid(t) {
            ret.push(x_from_t(t, a, b, c));
        }

        let t = (a.y - b.y - sqrt_term) / u;
        if t_valid(t) {
            ret.push(x_from_t(t, a, b, c));
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use crate::{bezier::Bezier2, test_data::test_data::test_curves};

    #[test]
    fn test_intersect_horz() {
        let curves = test_curves();
        let c = &curves[0];
        assert_eq!(c.intersect_horz(0.0), Vec::from([]));
        assert_eq!(c.intersect_horz(72.5), Vec::from([]));
        // assert_eq!(c.intersect_horz(362.5), Vec::from([731.0]));

        // assert_eq!(c.intersect_vert(1328.1), Vec::from([4.59121e-41]));

        let c = Bezier2::from(1313.0, 344.0, 1071.0, 89.0, 1229.0, 178.0);
        assert_eq!(c.intersect_vert(0.0), Vec::from([]));
        assert_eq!(c.intersect_vert(1258.2), Vec::from([254.39021]));

        let c = Bezier2::from(1314.0, 1116.0, 1398.0, 731.0, 1398.0, 953.0);
        assert_eq!(c.intersect_vert(1398.0), Vec::from([731.0, 731.0]));
    }
}
