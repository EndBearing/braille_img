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
braille-img = "0.2"
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
| `--dither <MODE>` | `-d` | `none` | ディザリングモード：`none` / `bayer` / `floyd` |

### 使用例

```sh
# デフォルト設定で変換
braille-img photo.png

# 白背景のロゴを反転して変換
braille-img logo.png --invert

# Bayer ディザリング（格子状パターン）
braille-img photo.png --width 160 --dither bayer

# Floyd-Steinberg ディザリング（より自然な粒状感）
braille-img photo.png --width 160 --dither floyd

# ファイルに保存
braille-img photo.png --dither floyd > output.txt
```

## ライブラリの使い方

```rust
use braille_img::{Config, DitherMode, convert};

let img = image::open("photo.png").unwrap();

// デフォルト設定で変換
let art = convert(&img, &Config::default());
println!("{art}");

// Floyd-Steinberg ディザリングで変換（写真向き）
let cfg = Config {
    width: 120,
    threshold: 128,
    invert: false,
    dither: DitherMode::FloydSteinberg,
};
let art = convert(&img, &cfg);
println!("{art}");
```

## ディザリングについて

`--dither` オプションで中間調の再現方法を選択できます。

| モード | CLI 値 | 特徴 | 向いている画像 |
|---|---|---|---|
| なし（デフォルト） | `none` | 閾値のみで判定 | ロゴ・線画・文字など輪郭がくっきりした画像 |
| Bayer | `bayer` | 格子状パターンで中間調を表現 | グラデーションが多い画像 |
| Floyd-Steinberg | `floyd` | 誤差拡散でより自然な粒状感 | 写真・自然画像（最も滑らかな再現） |

## ライセンス

MIT
