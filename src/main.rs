use ascii_art::{Config, convert};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "画像ファイルから点字アスキーアートを生成する")]
struct Args {
    /// 変換する画像ファイルのパス
    image: PathBuf,

    /// 出力幅（点字文字数）
    #[arg(short, long, default_value_t = 80)]
    width: u32,

    /// 輝度閾値 0–255（この値以下を「点あり」にする）
    #[arg(short, long, default_value_t = 128)]
    threshold: u8,

    /// 明暗を反転する
    #[arg(short, long)]
    invert: bool,
}

fn main() {
    let args = Args::parse();
    let img = image::open(&args.image).unwrap_or_else(|e| {
        eprintln!("画像を開けませんでした: {e}");
        std::process::exit(1);
    });
    let cfg = Config {
        width: args.width,
        threshold: args.threshold,
        invert: args.invert,
    };
    println!("{}", convert(&img, &cfg));
}
