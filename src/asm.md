<!-- # Assembly on stable -->

# stableでのアセンブリ

<!-- 
So far we have managed to boot the device and handle interrupts without a single
line of assembly. That's quite a feat! But depending on the architecture you are
targeting you may need some assembly to get to this point. There are also some
operations like context switching that require assembly, etc.
 -->

ここまで、デバイスの起動と割り込み処理とを、1行のアセンブリも書くことなくうまくやって来ました。
これはかなりの偉業です！しかし、ターゲットアーキテクチャ次第では、
ここまで到達するためにアセンブリが必要になるかもしれません。
他にも、コンテキストスイッチのようなアセンブリを必要とする操作があります。

<!-- 
The problem is that both *inline* assembly (`asm!`) and *free form* assembly
(`global_asm!`) are unstable, and there's no estimate for when they'll be
stabilized, so you can't use them on stable . This is not a showstopper because
there are some workarounds which we'll document here.
 -->

問題は、*インライン*アセンブリ（`asm!`）も*自由形式*アセンブリ（`global_asm!`）もunstableなことです。
そして、これらがいつ安定化されるかは分かっていないため、stableでは使えません。
これから説明するように、いくつかのワークアラウンドがあるため、致命的な問題ではありません。

<!-- 
To motivate this section we'll tweak the `HardFault` handler to provide
information about the stack frame that generated the exception.
 -->

本セクションの動機付けとして、`HardFault`ハンドラを、
例外を発生させたスタックフレームの情報を提供するように修正します。

<!-- Here's what we want to do: -->

やりたいことは下記の通りです。

<!-- 
Instead of letting the user directly put their `HardFault` handler in the vector
table we'll make the `rt` crate put a trampoline to the user-defined `HardFault`
handler in the vector table.
 -->

ベクタテーブルにユーザーが`HardFault`ハンドラを直接配置する代わりに、
`rt`クレートがユーザー定義の`HardFault`をトランポリンするハンドラをベクタテーブルに配置します。

``` console
$ tail -n36 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/asm/rt/src/lib.rs:61:96}}
```

<!-- 
This trampoline will read the stack pointer and then call the user `HardFault`
handler. The trampoline will have to be written in assembly:
 -->

このトランポリンはスタックポインタを読んで、ユーザーの`HardFault`ハンドラを呼びます。
トランポリンはアセンブリで次のように書かなければなりません。

``` armasm
{{#include ../ci/asm/rt/asm.s:5:6}}
```

<!-- 
Due to how the ARM ABI works this sets the Main Stack Pointer (MSP) as the first
argument of the `HardFault` function / routine. This MSP value also happens to
be a pointer to the registers pushed to the stack by the exception. With these
changes the user `HardFault` handler must now have signature
`fn(&StackedRegisters) -> !`.
 -->

ARM ABIでは、このメインスタックポインタ（MSP; Main Stack Pointer）の設定は、`HardFault`関数/ルーチンの第一引数になります。
このMSPの値は、例外によってスタックにプッシュされたレジスタへのポインタです。
これらの変更により、ユーザーの`HardFault`ハンドラは、`fn(&StackedRegisters) -> !`というシグネチャを持たなければなりません。

<!-- ## `.s` files -->

## `.s`ファイル

<!-- One approach to stable assembly is to write the assembly in an external file: -->

stableでアセンブリを書く方法の1つは、アセンブリを外部ファイルに書くことです。

``` console
$ cat ../rt/asm.s
```

``` armasm
{{#include ../ci/asm/rt/asm.s}}
```

<!-- 
And use the `cc` crate in the build script of the `rt` crate to assemble that
file into an object file (`.o`) and then into an archive (`.a`).
 -->

そして、`rt`クレートのビルドスクリプト内で、アセンブリファイルをオブジェクトファイル（`.o`）にアセンブルし、
アーカイブ（`.a`）にするために、`cc`クレートを使います。

``` console
$ cat ../rt/build.rs
```

``` rust
{{#include ../ci/asm/rt/build.rs}}
```

``` console
$ tail -n2 ../rt/Cargo.toml
```

``` toml
{{#include ../ci/asm/rt/Cargo.toml:7:8}}
```

<!-- And that's it! -->

これで全てです！

<!-- 
We can confirm that the vector table contains a pointer to `HardFaultTrampoline`
by writing a very simple program.
 -->

とても簡単なプログラムを書くだけで、ベクタテーブルが`HardFaultTrampoline`へのポインタを持つことが確認できます。

``` rust
{{#include ../ci/asm/app/src/main.rs}}
```

<!-- Here's the disassembly. Look at the address of `HardFaultTrampoline`. -->

逆アセンブリの結果は、以下の通りです。`HardFaultTrampoline`のアドレスを見て下さい。

``` console
$ cargo objdump --bin app --release -- -d -no-show-raw-insn -print-imm-hex
```

``` text
{{#include ../ci/asm/app/release.objdump}}
```

<!-- 
> **NOTE:** To make this disassembly smaller I commented out the initialization
> of RAM
 -->

> **注記** この逆アセンブリ結果を小さくするために、RAMの初期化をコメントアウトしています。

<!-- 
Now look at the vector table. The 4th entry should be the address of
`HardFaultTrampoline` plus one.
 -->

ここで、ベクタテーブルを見ます。
4つ目のエントリは、`HardFaultTrampoline`に1を足したアドレスになっているはずです。

``` console
$ cargo objdump --bin app --release -- -s -j .vector_table
```

``` text
{{#include ../ci/asm/app/release.vector_table}}
```

<!-- ## `.o` / `.a` files -->

## `.o` / `.a`ファイル

<!-- 
The downside of using the `cc` crate is that it requires some assembler program
on the build machine. For example when targeting ARM Cortex-M the `cc` crate
uses `arm-none-eabi-gcc` as the assembler.
 -->

`cc`クレートを使う欠点は、ビルドマシンにアセンブラプログラムが必要なことです。
例えば、ARM Cortex-Mをターゲットにする時、`cc`クレートはアセンブラとして`arm-none-eabi-gcc`を使います。

<!-- 
Instead of assembling the file on the build machine we can ship a pre-assembled
file with the `rt` crate. That way no assembler program is required on the build
machine. However, you would still need an assembler on the machine that packages
and publishes the crate.
 -->

ビルドマシン上でファイルをアセンブルする代わりに、`rt`クレートと一緒にあらかじめアセンブルしたファイルを配布できます。
この方法なら、ビルドマシンにアセンブラプログラムは必要ありません。
しかしながら、rtクレートをパッケージして発行するマシン上には、アセンブラが必要です。

<!-- 
There's not much difference between an assembly (`.s`) file and its *compiled*
version: the object (`.o`) file. The assembler doesn't do any optimization; it
simply chooses the right object file format for the target architecture.
 -->

アセンブリファイル（`.s`）と、コンパイルしたオブジェクトファイル（`.o`）とは、それほど違いがありません。
アセンブラは最適化を行いません。単純にターゲットアーキテクチャ向けに正しいオブジェクトファイル形式を選ぶだけです。

<!-- 
Cargo provides support for bundling archives (`.a`) with crates. We can package
object files into an archive using the `ar` command and then bundle the archive
with the crate. In fact, this what the `cc` crate does; you can see the commands
it invoked by searching for a file named `output` in the `target` directory.
 -->

Cargoは、クレートとアーカイブ（`.a`）をまとめる機能を提供しています。`ar`コマンドを使ってオブジェクトファイルをアーカイブにパッケージできます。
その後、アーカイブをクレートにまとめます。実は、これは`cc`クレートが行っていることなのです。
ccクレートが呼び出しているコマンドは、`target`ディレクトリの`output`という名前のファイルを探すと見つかります。

``` console
$ grep running $(find target -name output)
```

``` text
running: "arm-none-eabi-gcc" "-O0" "-ffunction-sections" "-fdata-sections" "-fPIC" "-g" "-fno-omit-frame-pointer" "-mthumb" "-march=armv7-m" "-Wall" "-Wextra" "-o" "/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out/asm.o" "-c" "asm.s"
running: "ar" "crs" "/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out/libasm.a" "/home/japaric/rust-embedded/embedonomicon/ci/asm/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out/asm.o"
```

``` console
$ grep cargo $(find target -name output)
```

``` tetx
cargo:rustc-link-search=/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out
cargo:rustc-link-lib=static=asm
cargo:rustc-link-search=native=/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out
```

<!-- We'll do something similar to produce an archive. -->

アーカイブを作成するために似たことを行います。

``` console
$ # `cc`が使う多くのフラグはアセンブル時には意味がないため、それらは取り除きます
$ arm-none-eabi-as -march=armv7-m asm.s -o asm.o

$ ar crs librt.a asm.o

$ arm-none-eabi-objdump -Cd librt.a
```

``` text
{{#include ../ci/asm/rt2/librt.objdump}}
```

<!-- 
Next we modify the build script to bundle this archive with the `rt` rlib.
 -->

次に、`rt` rlibにアーカイブをまとめるために、ビルドスクリプトを修正します。

``` console
$ cat ../rt/build.rs
```

``` rust
{{#include ../ci/asm/rt2/build.rs}}
```

<!-- 
Now we can test this new version against the simple program from before and
we'll get the same output.
 -->

ここで、新バージョンが前のシンプルなプログラムと同じ出力をすることをテストできます。

``` console
$ cargo objdump --bin app --release -- -d -no-show-raw-insn -print-imm-hex
```

``` text
{{#include ../ci/asm/app2/release.objdump}}
```

<!-- 
> **NOTE**: As before I have commented out the RAM initialization to make the
> disassembly smaller.
 -->

> **注記** 前回同様、逆アセンブリの結果を小さくするために、RAMの初期化をコメントアウトしています。

``` console
$ cargo objdump --bin app --release -- -s -j .vector_table
```

``` text
{{#include ../ci/asm/app2/release.vector_table}}
```

<!-- 
The downside of shipping pre-assembled archives is that, in the worst case
scenario, you'll need to ship one build artifact for each compilation target
your library supports.
 -->

あらかじめアセンブルしたアーカイブを配布する欠点は、最悪の場合、
ライブラリがサポートするターゲットごとにビルド生成物を配布しないといけないことです。
