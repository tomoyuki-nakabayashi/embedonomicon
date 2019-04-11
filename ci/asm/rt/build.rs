use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

use cc::Build;

fn main() -> Result<(), Box<Error>> {
    // このクレートのビルドディレクトリです
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // ライブラリサーチパスを追加します
    println!("cargo:rustc-link-search={}", out_dir.display());

    // `link.x`をビルドディレクトリに置きます
    File::create(out_dir.join("link.x"))?.write_all(include_bytes!("link.x"))?;

    // `asm.s`ファイルをアセンブルします
    Build::new().file("asm.s").compile("asm"); // <- 追加!

    Ok(())
}
