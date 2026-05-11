# braille-img

画像ファイルから点字 Unicode（U+2800〜U+28FF）を使ったアスキーアートを生成する Rust クレートです。
CLI ツールとしても、ライブラリとしても利用できます。

```
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
⣿⣿⠟⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠻⣿⣿⣿⣿
⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿
⣿⣿⣦⣀⠀⠀⠀⣠⣴⣿⣿⣦⠀⠀⠀⣼⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
```

## インストール

### CLI ツールとして

```sh
cargo install braille-img
```

### ライブラリとして

`Cargo.toml` に追加してください：

```toml
[dependencies]
braille-img = "0.1"
```

## CLI の使い方

```sh
braille-img [OPTIONS] <IMAGE>
```

### オプション

| オプション | 短縮形 | デフォルト | 説明 |
|---|---|---|---|
| `--width <WIDTH>` | `-w` | `80` | 出力幅（点字文字数） |
| `--threshold <THRESHOLD>` | `-t` | `128` | 輝度閾値（0〜255）。この値以下を点ありにする |
| `--invert` | `-i` | なし | 明暗を反転する |
| `--dither` | `-d` | なし | Bayer 4×4 ディザリングを有効にする |

### 使用例

```sh
# デフォルト設定で変換
braille-img photo.png

# 白背景のロゴを反転して変換
braille-img logo.png --invert

# 高解像度 + ディザリング（写真に最適）
braille-img photo.png --width 160 --dither

# ファイルに保存
braille-img photo.png --dither > output.txt
```

## ライブラリの使い方

```rust
use braille_img::{Config, DitherMode, convert};

let img = image::open("photo.png").unwrap();

// デフォルト設定で変換
let art = convert(&img, &Config::default());
println!("{art}");

// Bayer ディザリングで変換（写真向き）
let cfg = Config {
    width: 120,
    threshold: 128,
    invert: false,
    dither: DitherMode::Bayer,
};
let art = convert(&img, &cfg);
println!("{art}");
```

## ディザリングについて

`--dither`（`DitherMode::Bayer`）を有効にすると、Bayer 4×4 ordered dithering によって
中間調を点の粗密で表現します。

| モード | 向いている画像 |
|---|---|
| なし（デフォルト） | ロゴ・線画・文字など輪郭がくっきりした画像 |
| Bayer | 写真・自然画像などグラデーションが多い画像 |

## ライセンス

MIT
