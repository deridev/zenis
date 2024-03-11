use image::Rgb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Color {
    Rgb(u8, u8, u8),
    Integer(u32),
}

impl From<Rgb<f32>> for Color {
    fn from(rgb: Rgb<f32>) -> Self {
        let r = (rgb.0[0] * 255.0) as u8;
        let g = (rgb.0[1] * 255.0) as u8;
        let b = (rgb.0[2] * 255.0) as u8;

        Color::Rgb(r, g, b)
    }
}

impl Color {
    pub const WHITE: Color = Color::Rgb(255, 255, 255);
    pub const VERY_LIGHT_GRAY: Color = Color::Rgb(214, 212, 217);
    pub const LIGHT_GRAY: Color = Color::Rgb(157, 156, 161);
    pub const GRAY: Color = Color::Rgb(101, 101, 102);
    pub const DARK_GRAY: Color = Color::Rgb(73, 73, 74);
    pub const ALMOST_BLACK: Color = Color::Rgb(32, 32, 33);
    pub const BLACK: Color = Color::Rgb(1, 1, 1);

    pub const CYAN: Color = Color::Rgb(18, 240, 230);
    pub const LIGHT_CYAN: Color = Color::Rgb(125, 250, 250);
    pub const DARK_CYAN: Color = Color::Rgb(5, 170, 170);

    pub const VERY_LIGHT_BLUE: Color = Color::Rgb(189, 198, 240);
    pub const LIGHT_BLUE: Color = Color::Rgb(168, 183, 230);
    pub const BLUE: Color = Color::Rgb(108, 139, 235);
    pub const DARK_BLUE: Color = Color::Rgb(67, 104, 217);
    pub const BLURPLE: Color = Color::Rgb(131, 118, 204);
    pub const VERY_DARK_BLUE: Color = Color::Rgb(34, 31, 166);

    pub const VERY_LIGHT_RED: Color = Color::Rgb(242, 162, 170);
    pub const LIGHT_RED: Color = Color::Rgb(230, 106, 118);
    pub const RED: Color = Color::Rgb(240, 41, 60);
    pub const DARK_RED: Color = Color::Rgb(148, 27, 38);
    pub const VERY_DARK_RED: Color = Color::Rgb(69, 5, 11);

    pub const CYAN_GREEN: Color = Color::Rgb(90, 230, 235);

    pub const VERY_LIGHT_GREEN: Color = Color::Rgb(177, 240, 192);
    pub const LIGHT_GREEN: Color = Color::Rgb(126, 242, 154);
    pub const GREEN: Color = Color::Rgb(56, 242, 102);
    pub const DARK_GREEN: Color = Color::Rgb(24, 161, 58);
    pub const VERY_DARK_GREEN: Color = Color::Rgb(6, 74, 23);

    pub const VERY_LIGHT_YELLOW: Color = Color::Rgb(242, 209, 116);
    pub const LIGHT_YELLOW: Color = Color::Rgb(240, 209, 84);
    pub const YELLOW: Color = Color::Rgb(245, 207, 37);
    pub const DARK_YELLOW: Color = Color::Rgb(199, 165, 14);
    pub const VERY_DARK_YELLOW: Color = Color::Rgb(120, 98, 34);

    pub const LIGHT_BEIGE: Color = Color::Rgb(240, 200, 177);
    pub const BEIGE: Color = Color::Rgb(245, 194, 164);
    pub const LIGHT_ORANGE: Color = Color::Rgb(242, 147, 87);
    pub const ORANGE: Color = Color::Rgb(242, 113, 31);
    pub const DARK_ORANGE: Color = Color::Rgb(207, 84, 6);
    pub const BROWN: Color = Color::Rgb(145, 84, 45);
    pub const DARK_BROWN: Color = Color::Rgb(61, 31, 12);

    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Rgb(r, g, b) => ((*r as u32) << 16u32) + ((*g as u32) << 8u32) + (*b as u32),
            Self::Integer(int) => *int,
        }
    }

    pub fn to_rgb_u8(&self) -> Rgb<u8> {
        match self {
            Color::Rgb(r, g, b) => Rgb::from([*r, *g, *b]),
            Color::Integer(int) => {
                let r = (*int >> 16) as u8;
                let g = ((*int >> 8) & 0xFF) as u8;
                let b = (*int & 0xFF) as u8;
                Rgb::from([r, g, b])
            }
        }
    }

    pub fn from_hexcode(hex: String) -> Self {
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
        Self::Rgb(r, g, b)
    }

    pub fn to_rgb(&self) -> Rgb<f32> {
        match self {
            Color::Rgb(r, g, b) => {
                let r = *r as f32 / 255.0;
                let g = *g as f32 / 255.0;
                let b = *b as f32 / 255.0;
                Rgb::from([r, g, b])
            }
            Color::Integer(int) => {
                let r = (*int >> 16) as u8 as f32 / 255.0;
                let g = ((*int >> 8) & 0xFF) as u8 as f32 / 255.0;
                let b = (*int & 0xFF) as u8 as f32 / 255.0;
                Rgb::from([r, g, b])
            }
        }
    }
}
