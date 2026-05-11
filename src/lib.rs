//! 画像ファイルから点字アスキーアートを生成するクレートです。
//!
//! 画像を読み込み、輝度情報をもとに点字 Unicode（U+2800〜U+28FF）の
//! 文字列へ変換します。CLI ツールとしても、ライブラリとしても利用できます。
//!
//! # 使い方
//!
//! ```no_run
//! use braille_img::{Config, DitherMode, convert};
//!
//! let img = image::open("photo.png").unwrap();
//! let cfg = Config {
//!     width: 80,
//!     threshold: 128,
//!     invert: false,
//!     dither: DitherMode::None,
//! };
//! let art = convert(&img, &cfg);
//! println!("{art}");
//! ```
//!
//! # 仕組み
//!
//! 画像は 2×4 ピクセルのブロックに分割されます。各ブロックが点字1文字に対応し、
//! ブロック内の各ピクセルが点字の8ドットのいずれかにマッピングされます。
//! 輝度が閾値以下のピクセルを「点あり」として Unicode コードポイントを計算します。

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

// Bayer 4x4 ordered dithering matrix (値 0–15)
const BAYER4: [[i32; 4]; 4] = [
    [ 0,  8,  2, 10],
    [12,  4, 14,  6],
    [ 3, 11,  1,  9],
    [15,  7, 13,  5],
];

/// ディザリングのモードを表します。
///
/// 単純な閾値処理では中間調が失われますが、ディザリングを使うことで
/// 点の粗密によってグラデーションを疑似的に再現できます。
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum DitherMode {
    /// ディザリングなし。閾値のみで白黒を判定します（デフォルト）。
    ///
    /// ロゴ・線画・文字など、輪郭がくっきりした画像に向いています。
    #[default]
    None,
    /// Bayer 4×4 ordered dithering。
    ///
    /// 格子状のパターンを使って中間調を表現します。
    /// 写真や自然画像のグラデーションを滑らかに再現したい場合に有効です。
    Bayer,
}

/// 点字アスキーアート生成の設定です。
///
/// # Examples
///
/// ```
/// use braille_img::{Config, DitherMode};
///
/// // デフォルト設定
/// let cfg = Config::default();
///
/// // カスタム設定
/// let cfg = Config {
///     width: 120,
///     threshold: 100,
///     invert: true,
///     dither: DitherMode::Bayer,
/// };
/// ```
pub struct Config {
    /// 出力幅（点字文字数）。
    ///
    /// 1文字あたり 2×4 ピクセルのブロックに対応します。
    /// 値を大きくするほど高解像度になりますが、端末幅を超えると折り返しが発生します。
    pub width: u32,
    /// 輝度閾値（0〜255）。
    ///
    /// この値**以下**の輝度を持つピクセルを「点あり」として扱います。
    /// 小さくすると暗い部分のみ描画され、大きくすると広い範囲が描画されます。
    pub threshold: u8,
    /// `true` にすると明暗を反転します。
    ///
    /// 白背景のロゴや図などを変換する際に使用します。
    pub invert: bool,
    /// ディザリングモード。
    ///
    /// [`DitherMode::Bayer`] を指定すると写真などの中間調が滑らかに表現されます。
    pub dither: DitherMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 80,
            threshold: 128,
            invert: false,
            dither: DitherMode::None,
        }
    }
}

/// 画像を点字アスキーアートの文字列に変換します。
///
/// 画像をリサイズしてグレースケール化した後、2×4 ピクセルのブロックごとに
/// 点字 Unicode 文字を生成し、改行で結合した文字列を返します。
///
/// # Arguments
///
/// * `img` - 変換する画像
/// * `cfg` - 変換設定（[`Config`] を参照）
///
/// # Examples
///
/// ```no_run
/// use braille_img::{Config, DitherMode, convert};
///
/// let img = image::open("photo.png").unwrap();
///
/// // ディザリングなしで変換
/// let art = convert(&img, &Config::default());
/// println!("{art}");
///
/// // Bayer ディザリングで変換（写真向き）
/// let cfg = Config { dither: DitherMode::Bayer, ..Config::default() };
/// let art = convert(&img, &cfg);
/// println!("{art}");
/// ```
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
                    let effective = match cfg.dither {
                        DitherMode::None => luma,
                        // Bayer値 0–15 を -128〜127 にマップして輝度に加算
                        DitherMode::Bayer => {
                            let noise = BAYER4[py as usize % 4][px as usize % 4] * 17 - 128;
                            (luma as i32 + noise).clamp(0, 255) as u8
                        }
                    };
                    let lit = if cfg.invert {
                        effective > cfg.threshold
                    } else {
                        effective <= cfg.threshold
                    };
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
        let img = make_img(4, 8, 255);
        let result = convert(&img, &Config { width: 2, threshold: 128, invert: false, dither: DitherMode::None });
        assert!(result.chars().filter(|c| *c != '\n').all(|c| c == '\u{2800}'));
    }

    #[test]
    fn full_black_image_fills_all_dots() {
        let img = make_img(4, 8, 0);
        let result = convert(&img, &Config { width: 2, threshold: 128, invert: false, dither: DitherMode::None });
        assert!(result.chars().filter(|c| *c != '\n').all(|c| c == '\u{28FF}'));
    }

    #[test]
    fn invert_flips_blank_and_full() {
        let white = make_img(4, 8, 255);
        let cfg_inv = Config { width: 2, threshold: 128, invert: true, dither: DitherMode::None };
        let result = convert(&white, &cfg_inv);
        assert!(result.chars().filter(|c| *c != '\n').all(|c| c == '\u{28FF}'));
    }

    #[test]
    fn bayer_dither_midgray_produces_mixed_dots() {
        let img = make_img(8, 8, 128);
        let cfg_dither = Config { width: 4, threshold: 128, invert: false, dither: DitherMode::Bayer };
        let result = convert(&img, &cfg_dither);
        let chars: Vec<char> = result.chars().filter(|c| *c != '\n').collect();
        let all_full = chars.iter().all(|&c| c == '\u{28FF}');
        let all_empty = chars.iter().all(|&c| c == '\u{2800}');
        assert!(!all_full && !all_empty, "Bayer dither should produce mixed output for mid-gray");
    }
}
