<!-- # A `main` interface -->

# `main`インタフェース

<!-- 
We have a minimal working program now, but we need to package it in a way that the end user can build
safe programs on top of it. In this section, we'll implement a `main` interface like the one standard
Rust programs use.
 -->

現在、最小限の動くプログラムがあります。しかし、エンドユーザーが安全なプログラムをビルドできるようにパッケージを作る必要があります。
このセクションでは、標準のRustプログラムが使うような`main`インタフェースを実装します。

<!-- First, we'll convert our binary crate into a library crate: -->

まず最初に、バイナリクレートをライブラリクレートに変換します。

``` console
$ mv src/main.rs src/lib.rs
```

<!-- And then rename it to `rt` which stands for "runtime". -->

そして、クレートを「runtime」を意味する`rt`という名前に変えます。

``` console
$ sed -i s/app/rt/ Cargo.toml

$ head -n4 Cargo.toml
```

``` toml
{{#include ../ci/main/rt/Cargo.toml:1:4}}
```

<!-- The first change is to have the reset handler call an external `main` function: -->

最初の変更は、リセットハンドから外部の`main`関数を呼び出すようにすることです。

``` console
$ head -n13 src/lib.rs
```

``` rust
{{#include ../ci/main/rt/src/lib.rs:1:13}}
```

<!-- We also drop the `#![no_main]` attribute has it has no effect on library crates. -->

`#![no_main]`アトリビュートも取り除いています。このアトリビュートはライブラリクレートには効果がありません。

<!-- 
> There's an orthogonal question that arises at this stage: Should the `rt`
> library provide a standard panicking behavior, or should it *not* provide a
> `#[panic_handler]` function and leave the end user choose the panicking
> behavior? This document won't delve into that question and for simplicity will
> leave the dummy `#[panic_handler]` function in the `rt` crate. However, we
> wanted to inform the reader that there are other options.
 -->

> ここで、直交する疑問が湧きます。`rt`ライブラリは標準のパニック動作を提供すべきでしょうか？それとも、
> `#[panic_handler]`関数を提供せずに、ユーザーがパニック動作を選べるように残しておくべきでしょうか？
> 本ドキュメントでは、この疑問を深堀りせず、単純化のために`rt`クレートにダミーの`#[panic_handler]`関数を残しておきます。
> しかしながら、他の選択肢があることを読者に伝えておきます。

<!-- 
The second change involves providing the linker script we wrote before to the application crate. You
see the linker will search for linker scripts in the library search path (`-L`) and in the directory
from which it's invoked. The application crate shouldn't need to carry around a copy of `link.x` so
we'll have the `rt` crate put the linker script in the library search path using a [build script].
 -->

2つ目の変更は、これまでに書いたリンカスクリプトを、アプリケーションクレートに提供することです。
リンカがライブラリサーチパス（`-L`）とリンカを呼び出したディレクトリから、リンカスクリプトを探すことはご存知でしょう。
アプリケーションクレートが`link.x`のコピーを持たなくて済むように、[ビルドスクリプト]を使って`rt`クレートが、
ライブラリサーチパスにリンカスクリプトを置くようにします。

<!-- [build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html -->

[ビルドスクリプト]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

``` console
$ # `rt`のルートディレクトリに、以下の内容でbuild.rsを作ります
$ cat build.rs
```

``` rust
{{#include ../ci/main/rt/build.rs}}
```

<!-- 
Now the user can write an application that exposes the `main` symbol and link it to the `rt` crate.
The `rt` will take care of giving the program the right memory layout.
 -->

これで、ユーザーは`main`シンボルを公開するアプリケーションを書くことができ、`rt`クレートとリンクすることができます。
`rt`は、アプリケーションプログラムに正しいメモリレイアウトを提供します。

``` console
$ cd ..

$ cargo new --edition 2018 --bin app

$ cd app

$ # Cargo.tomlを`rt`クレートとの依存関係を持つように修正します
$ tail -n2 Cargo.toml
```

``` toml
{{#include ../ci/main/app/Cargo.toml:7:8}}
```

``` console
$ # デフォルトターゲットとリンカ呼び出しを微調整した設定ファイルをコピーします
$ cp -r ../rt/.cargo .

$ # `main.rs`の内容を下記の通り変更します
$ cat src/main.rs
```

``` rust
{{#include ../ci/main/app/src/main.rs}}
```

<!-- The disassembly will be similar but will now include the user `main` function. -->

逆アセンブリの結果は似ていますが、ここではユーザーの`main`関数を含んでいます。

``` console
$ cargo objdump --bin app -- -d -no-show-raw-insn
```

``` text
{{#include ../ci/main/app/app.objdump}}
```

<!-- ## Making it type safe -->

## 型安全にする

<!-- 
The `main` interface works, but it's easy to get it wrong: For example, the user could write `main`
as a non-divergent function, and they would get no compile time error and undefined behavior (the
compiler will misoptimize the program).
 -->

`main`インタフェースは機能しますが、簡単に誤った使い方ができてしまいます。
例えば、ユーザーは発散しない関数として`main`関数を書くかもしれません。その結果、コンパイルエラーは発生しませんが、
未定義動作になるでしょう（コンパイラはプログラムに誤った最適化を行います）。

<!-- 
We can add type safety by exposing a macro to the user instead of the symbol interface. In the
`rt` crate, we can write this macro:
 -->

シンボルインタフェースではなくマクロをユーザーに公開することで、型安全性を追加することができます。
`rt`クレートに、次のマクロを書くことができます。

``` console
$ tail -n12 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/main/rt/src/lib.rs:25:37}}
```

<!-- Then the application writers can invoke it like this: -->

そして、アプリケーション作成者は、このマクロを次のように呼び出します。

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/main/app2/src/main.rs}}
```

<!-- 
Now the author will get an error if they change the signature of `main` to be
non divergent function, e.g. `fn()`.
 -->

今度は、アプリケーション作成者は、`main`のシグネチャを`fn()`のような発散しない関数に変更すると、
エラーに遭遇するでしょう。

<!-- ## Life before main -->

## main前の生活

<!-- 
`rt` is looking good but it's not feature complete! Applications written against it can't use
`static` variables or string literals because `rt`'s linker script doesn't define the standard
`.bss`, `.data` and `.rodata` sections. Let's fix that!
 -->

`rt`は良さそうに見えますが、まだ機能が完全ではありません！rtクレートに対して書かれたアプリケーションは、
`static`変数や文字列リテラルを使うことができません。`rt`のリンカスクリプトが、
標準の`.bss`、`.data`、`.rodata`セクションを定義していないからです。これを直していきましょう！

<!-- The first step is to define these sections in the linker script: -->

最初のステップはリンカスクリプトに下記のセクションを定義することです。

``` console
$ # ファイルの一部のみを見せます
$ sed -n 25,46p ../rt/link.x
```

``` text
{{#include ../ci/main/rt/link.x:25:46}}
```

<!-- 
They just re-export the input sections and specify in which memory region each output section will
go.
 -->

これらは入力セクションを単に再度エスクポートし、各メモリ領域のどこに出力セクションが置かれるかを指定しているだけです。

<!-- With these changes, the following program will compile: -->

これらの変更で、下記のプログラムがコンパイル可能になります。

``` rust
{{#include ../ci/main/app3/src/main.rs}}
```

<!-- 
However if you run this program on real hardware and debug it, you'll observe that the `static`
variables `BSS` and `DATA` don't have the values `0` and `1` by the time `main` has been reached.
Instead, these variables will have junk values. The problem is that the contents of RAM are
random after powering up the device. You won't be able to observe this effect if you run the
program in QEMU.
 -->

しかし、実際のハードウェア上でプログラムを実行し、デバッグすると、`main`に到達した時点で、
`static`変数の`BSS`と`DATA`が、`0`と`1`になっていないことに気づくでしょう。
代わりに、これらの変数はゴミデータを持っています。この問題は、デバイスの電源投入時、RAMがランダムなデータを持つためです。
プログラムをQEMUで実行すると、この現象は観測できません。

<!-- 
As things stand if your program reads any `static` variable before performing a write to it then
your program has undefined behavior. Let's fix that by initializing all `static` variables before
calling `main`.
 -->

実は、プログラムが`static`変数に書き込みを行う前に、その変数を読むことは、未定義動作です。
`main`を呼ぶ前に、全ての`static`変数を初期化するように修正しましょう。

<!-- We'll need to tweak the linker script a bit more to do the RAM initialization: -->

RAM初期化のために、リンカスクリプトをさらに微修正しなければなりません。

``` console
$ # ファイルの一部のみを見せます
$ sed -n 25,52p ../rt/link.x
```

``` text
{{#include ../ci/main/rt2/link.x:25:52}}
```

<!-- Let's go into the details of these changes: -->

変更内容を詳細に見ていきましょう。

``` text
{{#include ../ci/main/rt2/link.x:38}}
```

``` text
{{#include ../ci/main/rt2/link.x:40}}
```

``` text
{{#include ../ci/main/rt2/link.x:45}}
```

``` text
{{#include ../ci/main/rt2/link.x:47}}
```

<!-- 
We associate symbols to the start and end addresses of the `.bss` and `.data` sections, which we'll
later use from Rust code.
 -->

シンボルを`.bss`セクションと`.data`セクションの開始アドレスと終了アドレスに関連付けます。
これらは後ほど、Rustコードで使用します。

``` text
{{#include ../ci/main/rt2/link.x:43}}
```

<!-- 
We set the Load Memory Address (LMA) of the `.data` section to the end of the `.rodata`
section. The `.data` contains `static` variables with a non-zero initial value; the Virtual Memory
Address (VMA) of the `.data` section is somewhere in RAM -- this is where the `static` variables are
located. The initial values of those `static` variables, however, must be allocated in non volatile
memory (Flash); the LMA is where in Flash those initial values are stored.
 -->

`.rodata`セクションの終わりに、`.data`セクションのロードメモリアドレス（LMA; Load Memory Address）を設定します。
`.data`はゼロでない初期値をもった`static`変数が含まれています。`.data`セクションの仮想メモリアドレス（VMA; Virtual Memory Address）は、
RAMのどこかにあります。これは、`static`変数が配置されている場所です。
しかし、これらの`static`変数の初期値は、不揮発性メモリ（Flash）に割り当てられなければなりません。
LMAは、これらの初期値が格納されるFlashの場所を示しています。

``` text
{{#include ../ci/main/rt2/link.x:50}}
```

<!-- Finally, we associate a symbol to the LMA of `.data`. -->

最後に、`.data`セクションのLMAをシンボルに関連付けます。

<!-- 
On the Rust side, we zero the `.bss` section and initialize the `.data` section. We can reference
the symbols we created in the linker script from the Rust code. The *addresses*[^1] of these symbols are
the boundaries of the `.bss` and `.data` sections.
 -->

Rust側では、`.bss`セクションをゼロクリアし、`.data`セクションを初期化します。Rustコードからリンカスクリプトで作成したシンボルを参照できます。
これらのシンボルの*アドレス*[^1]は、`.bss`セクションと`.data`セクションの境界になります。

<!-- The updated reset handler is shown below: -->

リセットハンドラを、次のように更新します。

``` console
$ head -n32 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/main/rt2/src/lib.rs:1:31}}
```

<!-- 
Now end users can directly and indirectly make use of `static` variables without running into
undefined behavior!
 -->

これで、未定義動作なしに、エンドユーザーは直接的にも間接的にも`static`変数を使うことができます。

<!-- 
> In the code above we performed the memory initialization in a bytewise fashion. It's possible to
> force the `.bss` and `.data` sections to be aligned to, say, 4 bytes. This fact can then be used
> in the Rust code to perform the initialization wordwise while omitting alignment checks. If you
> are interested in learning how this can be achieved check the [`cortex-m-rt`] crate.
 -->

> 上記のコードでは、メモリ初期化をバイト単位の方法で初期化しています。`.bss`セクションと`.data`セクションを例えば4バイトでアライメントすることが可能です。
> このことは、アライメントチェックなしにワード単位の初期化を行うために、Rustコードで使うことができます。
> どのようにやるのか興味がある場合、[`cortex-m-rt`]クレートをチェックして下さい。

[`cortex-m-rt`]: https://github.com/japaric/cortex-m-rt/tree/v0.5.1

<!-- 
[^1]: The fact that the addresses of the linker script symbols must be used here can be confusing and
unintuitive. An elaborate explanation for this oddity can be found [here](https://stackoverflow.com/a/40392131).
 -->

[^1]: ここで使っているリンカスクリプトシンボルのアドレスを使用する必要があるということは、混乱を招きやすく、直感的ではありません。
この奇妙さについての詳細な説明は、[ここ](https://stackoverflow.com/a/40392131)にあります。
