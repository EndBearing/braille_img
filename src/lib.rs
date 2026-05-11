use image::{DynamicImage, GenericImageView};

// 2x4ブロックの各ドット位置 (col, row) -> 点字ビット番号
// 点字Unicode: U+2800 + bits
//   dot1=bit0  dot2=bit3
//   dot3=bit1  dot4=bit4
//   dot5=bit2  dot6=bit5
//   dot7=bit6  dot8=bit7
const DOT_BIT: [(u32, u32, u8); 8] = [
    (0, 0, 0), // dot1
    (1, 0, 3), // dot2
    (0, 1, 1), // dot3
    (1, 1, 4), // dot4
    (0, 2, 2), // dot5
    (1, 2, 5), // dot6
    (0, 3, 6), // dot7
    (1, 3, 7), // dot8
];

pub struct Config {
    /// 出力幅（点字文字数）
    pub width: u32,
    /// 輝度閾値 0–255。この値以下のピクセルを「点あり」にする
    pub threshold: u8,
    /// true にすると明暗を反転する
    pub invert: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 80,
            threshold: 128,
            invert: false,
        }
    }
}

pub fn convert(img: &DynamicImage, cfg: &Config) -> String {
    let (orig_w, orig_h) = img.dimensions();

    // 出力ピクセル幅 = 点字文字数 * 2列
    let px_w = cfg.width * 2;
    // 点字1文字は 2px幅 × 4px高さ。標準端末フォント（縦横比1:2）では
    // 1画像ピクセルの物理幅 = W/2、物理高さ = 2W/4 = W/2 → 補正不要
    let px_h = (orig_h as f64 * px_w as f64 / orig_w as f64) as u32;
    let px_h = px_h.max(4);

    let resized = img.resize_exact(px_w, px_h, image::imageops::FilterType::Lanczos3);
    let gray = resized.to_luma8();

    let char_rows = (px_h + 3) / 4;
    let char_cols = cfg.width;

    let mut lines = Vec::with_capacity(char_rows as usize);
    for cy in 0..char_rows {
        let mut line = String::with_capacity(char_cols as usize * 3);
        for cx in 0..char_cols {
            let mut bits: u8 = 0;
            for (dx, dy, bit) in &DOT_BIT {
                let px = cx * 2 + dx;
                let py = cy * 4 + dy;
                if px < px_w && py < px_h {
                    let luma = gray.get_pixel(px, py).0[0];
                    let lit = if cfg.invert { luma > cfg.threshold } else { luma <= cfg.threshold };
                    if lit {
                        bits |= 1 << bit;
                    }
                }
            }
            let ch = char::from_u32(0x2800 + bits as u32).unwrap();
            line.push(ch);
        }
        lines.push(line);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    fn make_img(w: u32, h: u32, fill: u8) -> DynamicImage {
        DynamicImage::ImageLuma8(GrayImage::from_pixel(w, h, Luma([fill])))
    }

    #[test]
    fn blank_image_is_empty_braille() {
        let img = make_img(4, 8, 255); // 白一面 → ドットなし
        let result = convert(&img, &Config { width: 2, threshold: 128, invert: false });
        // すべての文字が U+2800（空点字）であること
        assert!(result.chars().filter(|c| *c != '\n').all(|c| c == '\u{2800}'));
    }

    #[test]
    fn full_black_image_fills_all_dots() {
        let img = make_img(4, 8, 0); // 黒一面 → 全ドットあり
        let result = convert(&img, &Config { width: 2, threshold: 128, invert: false });
        assert!(result.chars().filter(|c| *c != '\n').all(|c| c == '\u{28FF}'));
    }

    #[test]
    fn invert_flips_blank_and_full() {
        let white = make_img(4, 8, 255);
        let cfg_inv = Config { width: 2, threshold: 128, invert: true };
        let result = convert(&white, &cfg_inv);
        assert!(result.chars().filter(|c| *c != '\n').all(|c| c == '\u{28FF}'));
    }
}
