# The embedonomicon

<!-- 
The embedonomicon walks you through the process of creating a `#![no_std]` application from scratch
and through the iterative process of building architecture-specific functionality for Cortex-M
microcontrollers.
 -->

embedonomiconは、`#![no_std]`アプリケーションをスクラッチから作成するプロセスと、
Cortex-Mマイクロコントローラ向けにアーキテクチャ固有の機能を作るイテレーティブなプロセスを案内します。

<!-- ## Objectives -->

## 目的

<!-- By reading this book you will learn -->

本書を読むことで、次のことを学べます。

<!-- 
- How to build a `#![no_std]` application. This is much more complex than building a `#![no_std]`
  library because the target system may not be running an OS (or you could be aiming to build an
  OS!) and the program could be the only process running in the target (or the first one).
  In that case, the program may need to be customized for the target system.
 -->

- `#[no_std]`アプリケーションのビルド方法。これは、`#![no_std]`ライブラリをビルドするより、はるかに複雑です。
  なぜなら、ターゲットシステムではOSが動いていないからです（もしくは、OSを作ろうとしているかもしれません！）。
  そして、プログラムは、ターゲット上で動作する唯一（もしくは、最初の1つ）のプロセスになります。
  この場合、プログラムは、ターゲットシステム向けにカスタマイズが必要です。

<!-- 
- Tricks to finely control the memory layout of a Rust program. You'll learn about linkers, linker
  scripts and about the Rust features that let you control a bit of the ABI of Rust programs.
 -->

- Rustプログラムのメモリレイアウトを細かく制御するためのコツ。
  リンカ、リンカスクリプト、および、RustプログラムのABIの一部を制御できるようにするRustの機能について学びます。

<!-- 
- A trick to implement default functionality that can be statically overridden (no runtime cost).
 -->

- （実行時にコストがかからない）静的オーバーライド可能なデフォルト機能を実装する秘訣。

<!-- ## Target audience -->

## 対象読者

<!-- This book mainly targets to two audiences: -->

本書は主に、2つの読者を対象としています。

<!-- 
- People that wish to bootstrap bare metal support for an architecture that the ecosystem doesn't
  yet support (e.g. Cortex-R as of Rust 1.28), or for an architecture that Rust just gained support
  for (e.g. maybe Xtensa some time in the future).
 -->

- エコシステムがまだサポートしていないアーキテクチャ（例えば、Rust 1.28におけるCortex-R）や、
  Rustがサポートを始めたばかりのアーキテクチャ（例えば、Extensaは将来サポートされるかもしれません）に対して、
  ベアメタルでのブートを提供したい人々

<!-- 
- People that are curious about the unusual implementation of *runtime* crates like [`cortex-m-rt`],
  [`msp430-rt`] and [`riscv-rt`].
 -->

- [`cortex-m-rt`]、[`msp430-rt`]、[`riscv-rt`]のような*ランタイム*クレートの珍しい実装方法について興味がある人々。

[`cortex-m-rt`]: https://crates.io/crates/cortex-m-rt
[`msp430-rt`]: https://crates.io/crates/msp430-rt
[`riscv-rt`]: https://crates.io/crates/riscv-rt

<!-- ## Requirements -->

## 要求事項

<!-- 
This book is self contained. The reader doesn't need to be familiar with the
Cortex-M architecture, nor is access to a Cortex-M microcontroller needed -- all
the examples included in this book can be tested in QEMU. You will, however,
need to install the following tools to run and inspect the examples in this
book:
 -->

本書は、自己完結しています。読者は、Cortex-Mアーキテクチャについて詳しかったり、Cortex-Mマイクロコントローラを持っている必要はありません。
本書内の例は、全てQEMU上でテストできます。しかしながら、本書内の例を実行したり調査するため、次のツールをインストールする必要があります。

<!-- 
- All the code in this book uses the 2018 edition. If you are not familiar with
  the 2018 features and idioms check the [`edition guide`].
 -->

- 本書内の全コードは、2018エディションを使います。2018エディションの機能やイディオムを知らない場合は、
  [`エディションガイド`]を確認して下さい。

<!-- - Rust 1.31 or a newer toolchain PLUS ARM Cortex-M compilation support. -->

- Rust 1.31以上のツールチェインとARM Cortex-Mコンパイルのサポート

<!-- - [`cargo-binutils`](https://github.com/japaric/cargo-binutils). v0.1.4 or newer. -->

- [`cargo-binutils`](https://github.com/japaric/cargo-binutils)。v0.1.4以上。

- [`cargo-edit`](https://crates.io/crates/cargo-edit).

<!-- 
- QEMU with support for ARM emulation. The `qemu-system-arm` program must be
  installed on your computer.
 -->

- ARMエミュレーションをサポートしているQEMU。`qemu-system-arm`がインストールされていなければなりません。

<!-- - GDB with ARM support. -->

- ARMサポートのGDB。

<!-- [`edition guide`]: https://rust-lang-nursery.github.io/edition-guide/ -->

[`エディションガイド`]: https://rust-lang-nursery.github.io/edition-guide/

<!-- ### Example setup -->

### 設定例

<!-- Instructions common to all OSes -->

全てのOSに共通する手順です。

``` console
$ # Rustツールチェイン
$ # 1からやる場合、https://rustup.rs/からrustupを入手して下さい
$ rustup default stable

$ # ツールチェインは、これより新しくなければなりません
$ rustc -V
rustc 1.31.0 (abe02cefd 2018-12-04)

$ rustup target add thumbv7m-none-eabi

$ # cargo-binutils
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview

```

#### macOS

``` console
$ # arm-none-eabi-gdb
$ # 最初に`brew tap Caskroom/tap`を実行しなければならないかもしれません
$ brew cask install gcc-arm-embedded

$ # QEMU
$ brew install qemu
```

#### Ubuntu 16.04

``` console
$ # arm-none-eabi-gdb
$ sudo apt install gdb-arm-none-eabi

$ # QEMU
$ sudo apt install qemu-system-arm
```

#### Ubuntu 18.04 or Debian

``` console
$ # gdb-multiarch。gdbを起動する時は、`gdb-multiarch`を使って下さい
$ sudo apt install gdb-multiarch

$ # QEMU
$ sudo apt install qemu-system-arm
```

#### Windows

<!-- 
- [arm-none-eabi-gdb](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads).
  The GNU Arm Embedded Toolchain includes GDB.
 -->

- [arm-none-eabi-gdb](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads)。
  GDBを含むGNU Arm Embeddedツールチェイン

- [QEMU](https://www.qemu.org/download/#windows)

<!-- ## Installing a toolchain bundle from ARM (optional step) (tested on Ubuntu 18.04) -->

## （オプションのステップ）（Ubuntu 18.04でテスト済み）ARMからツールチェイン一式をインストール

<!-- 
- With the late 2018 switch from
[GCC's linker to LLD](https://rust-embedded.github.io/blog/2018-08-2x-psa-cortex-m-breakage/) for Cortex-M 
microcontrollers, [gcc-arm-none-eabi][1] is no longer 
required.  But for those wishing to use the toolchain 
anyway, install from [here][1] and follow the steps outlined below:
 -->

- 最近の2018では、Cortex-Mマイクロコントローラ向けのリンカが、[GCCのリンカからLLD](https://rust-embedded.github.io/blog/2018-08-2x-psa-cortex-m-breakage/)に切り替わりました。
  [gcc-arm-none-eabi][1]はもはや必要ありません。しかし、このツールチェインを使いたい人は、[ここ][1]から下記の手順でインストールできます。

``` console
$ tar xvjf gcc-arm-none-eabi-8-2018-q4-major-linux.tar.bz2
$ mv gcc-arm-none-eabi-<version_downloaded> <your_desired_path> # オプション
$ export PATH=${PATH}:<path_to_arm_none_eabi_folder>/bin # 設定を永続的にするため、この行を.bashrcに追加します。
```
[1]: https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads
