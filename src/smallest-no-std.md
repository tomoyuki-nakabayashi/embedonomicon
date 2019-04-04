<!-- # The smallest `#![no_std]` program -->

# 最小限の`#![no_std]`プログラム

<!-- In this section we'll write the smallest `#![no_std]` program that *compiles*. -->

このセクションでは、コンパイルできる最小限の`#![no_std]`プログラムを書きます。

<!-- ## What does `#![no_std]` mean? -->

## `#![no_std]`はどういう意味でしょうか？

<!-- 
`#![no_std]` is a crate level attribute that indicates that the crate will link to the [`core`] crate
instead of the [`std`] crate, but what does this mean for applications?
 -->

`#![no_std]`は、クレートレベルのアトリビュートです。これは、このクレートに[`std`]クレートではなく[`core`]クレートをリンクすることを示します。
しかし、アプリケーションにとって、これは何を意味するのでしょうか？

[`core`]: https://doc.rust-lang.org/core/
[`std`]: https://doc.rust-lang.org/std/

<!-- 
The `std` crate is Rust's standard library. It contains functionality that assumes that the program
will run on an operating system rather than [*directly on the metal*]. `std` also assumes that the
operating system is a general purpose operating system, like the ones one would find in servers and
desktops. For this reason, `std` provides a standard API over functionality one usually finds in
such operating systems: Threads, files, sockets, a filesystem, processes, etc.
 -->

`std`クレートはRustの標準ライブラリです。
標準ライブラリは、プログラムがベアメタルではなく、オペレーティングシステム上で動作することを仮定した機能を、含んでいます。
`std`は、オペレーティングシステムが、サーバやデスクトップで使うような汎用オペレーティングシステムであることも仮定します。
この理由から、`std`は、スレッド、ファイル、ソケット、ファイルシステム、プロセス、など汎用オペレーティングシステムにある機能に対して標準APIを提供します。

<!-- [*directly on the metal*]: https://en.wikipedia.org/wiki/Bare_machine -->

[*ベアメタル*]: https://en.wikipedia.org/wiki/Bare_machine

<!-- 
On the other hand, the `core` crate is a subset of the `std` crate that makes zero assumptions about
the system the program will run on. As such, it provides APIs for language primitives like floats,
strings and slices, as well as APIs that expose processor features like atomic operations and SIMD
instructions. However it lacks APIs for anything that involves heap memory allocations and I/O.
 -->

その一方、`core`クレートは、`std`クレートのサブセットで、プログラムが動作するシステムについて、一切の仮定を置きません。
そのため、`core`クレートは、浮動小数点や文字列、スライスのような言語のプリミティブと、
アトミック操作やSIMD命令のようなプロセッサ機能を利用するためのAPIを提供します。
しかし、`core`クレートは、ヒープメモリアロケーションやI/Oといったものに対するAPIがありません。

<!-- 
For an application, `std` does more than just providing a way to access OS abstractions. `std` also
takes care of, among other things, setting up stack overflow protection, processing command line
arguments and spawning the main thread before a program's `main` function is invoked. A `#![no_std]`
application lacks all that standard runtime, so it must initialize its own runtime, if any is
required.
 -->

アプリケーションに対しては、`std`は単に抽象化されたOS機能へのアクセス方法を提供するだけに留まりません。
`std`は、とりわけ、スタックオーバーフロープロテクションの設定、コマンドライン引数の処理、
プログラムの`main`関数が呼び出される前のメインスレッド生成、の面倒をみます。
`#![no_std]`アプリケーションは、これらの標準的なランタイムを持ちません。そのため、必要に応じて、自身のランタイムを初期化しなければなりません。

<!-- 
Because of these properties, a `#![no_std]` application can be the first and / or the only code that
runs on a system. It can be many things that a standard Rust application can never be, for example:
- The kernel of an OS.
- Firmware.
- A bootloader.
 -->

これらの性質から、`#![no_std]`アプリケーションは、システム上で動作する最初の / 唯一のコードになれます。
標準のRustアプリケーションでは決して作ることができない、次のようなプログラムを書くことができます。

- OSのカーネル
- ファームウェア
- ブートローダ

<!-- ## The code -->

## コード

<!-- With that out of the way, we can move on to the smallest `#![no_std]` program that compiles: -->

この普通でない方法で、コンパイル可能な最小限の`#![no_std]`プログラムに取り掛かることができます。

``` console
$ cargo new --edition 2018 --bin app

$ cd app
```

``` console
$ # main.rsを下記の内容に修正して下さい
$ cat src/main.rs
```

``` rust
{{#include ../ci/smallest-no-std/src/main.rs}}
```

<!-- This program contains some things that you won't see in standard Rust programs: -->

このプログラムは、標準的なRustプログラムでは目にすることがない内容を含んでいます。

<!-- The `#![no_std]` attribute which we have already extensively covered. -->

`#![no_std]`アトリビュートについては、既に十分に説明しました。

<!-- 
The `#![no_main]` attribute which means that the program won't use the standard `main` function as
its entry point. At the time of writing, Rust's `main` interface makes some assumptions about the
environment the program executes in: For example, it assumes the existence of command line
arguments, so in general, it's not appropriate for `#![no_std]` programs.
 -->

`#![no_main]`アトリビュートは、エントリポイントとして標準の`main`関数を使わないプログラムであることを意味します。
本書を書いている時点では、Rustの`main`インタフェースは、プログラムを実行する環境について、いくつかの仮定を置いています。
例えば、コマンドライン引数が存在していることですが、これは一般的に`#![no_std]`プログラムにはふさわしくありません。

<!-- 
The `#[panic_handler]` attribute. The function marked with this attribute defines the behavior
of panics, both library level panics (`core::panic!`) and language level panics (out of bounds
indexing).
 -->

`#[panic_handler]`アトリビュートでマーキングされた関数は、パニック発生時の動作を定義します。
ライブラリレベルのパニック（`core::panic!`）と言語レベルのパニック（範囲外のインデックスアクセス）両方が対象です。

<!-- This program doesn't produce anything useful. In fact, it will produce an empty binary. -->

このプログラムは、役に立つものではありません。実際、空のバイナリを生成します。

``` console
$ $ `size target/thumbv7m-none-eabi/debug/app`と同じです
$ cargo size --target thumbv7m-none-eabi --bin app
```

``` text
{{#include ../ci/smallest-no-std/app.size}}
```

<!-- Before linking the crate does contain the panicking symbol. -->

リンク前、このクレートはパニックのシンボルを含んでいます。

``` console
$ cargo rustc --target thumbv7m-none-eabi -- --emit=obj

$ cargo nm -- target/thumbv7m-none-eabi/debug/deps/app-*.o | grep '[0-9]* [^n] '
```

``` text
{{#include ../ci/smallest-no-std/app.o.nm}}
```

<!-- 
However, it's our starting point. In the next section, we'll build something useful. But before
continuing, let's set a default build target to avoid having to pass the `--target` flag to every
Cargo invocation.
 -->

しかしながら、これがスタート地点です。次のセクションでは、役に立つものをビルドします。
しかしその前に、Cargo呼び出しごとに`--target`フラグを付けなくて良いように、デフォルトビルドターゲットを設定しましょう。

``` console
$ mkdir .cargo

$ # .cargo/configが下記内容になるように修正します
$ cat .cargo/config
```

``` toml
{{#include ../ci/smallest-no-std/.cargo/config}}
```
