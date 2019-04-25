<!-- # Logging with symbols -->

# シンボルでのロギング

<!-- 
This section will show you how to utilize symbols and the ELF format to achieve
super cheap logging.
 -->

このセクションでは、極めて軽量なロギングを行うために、シンボルとELFフォーマットを利用する方法をお見せします。

<!-- ## Arbitrary symbols -->

## 任意のシンボル

<!-- 
Whenever we needed a stable symbol interface between crates we have mainly used
the `no_mangle` attribute and sometimes the `export_name` attribute. The
`export_name` attribute takes a string which becomes the name of the symbol
whereas `#[no_mangle]` is basically sugar for `#[export_name = <item-name>]`.
 -->

クレート間で、安定したシンボルインタフェースが必要な場合、`no_mangle`アトリビュートを主に使用し、
時には、`export_name`アトリビュートを使用します。
`export_name`アトリビュートは、シンボル名になる文字列を引数に取ります。
一方、`#[no_mangle]`は、基本的に`#[export_name = <item-name>]`のシンタックスシュガーです。

<!-- 
Turns out we are not limited to single word names; we can use arbitrary strings,
e.g. sentences, as the argument of the `export_name` attribute. As least when
the output format is ELF anything that doesn't contain a null byte is fine.
 -->

引数に取れる文字列は、1単語の名前だけに限定されないことがわかりました。
例えば、文のような任意の文字列を`export_name`アトリビュートの引数として使うことができます。
少なくても出力形式がELFの場合、nullバイトを含まないものならば何でも構いません。

<!-- Let's check that out: -->

そのことを確認してみましょう。

``` console
$ cargo new --lib foo

$ cat foo/src/lib.rs
```

``` rust
#[export_name = "Hello, world!"]
#[used]
static A: u8 = 0;

#[export_name = "こんにちは"]
#[used]
static B: u8 = 0;
```

``` console
$ ( cd foo && cargo nm --lib )
foo-d26a39c34b4e80ce.3lnzqy0jbpxj4pld.rcgu.o:
0000000000000000 r Hello, world!
0000000000000000 V __rustc_debug_gdb_scripts_section__
0000000000000000 r こんにちは
```

<!-- Can you see where this is going? -->

これがどこに繋がるか、わかりますか？

<!-- ## Encoding -->

## エンコードする

<!-- 
Here's what we'll do: we'll create one `static` variable per log message but
instead of storing the messages *in* the variables we'll store the messages in
the variables' *symbol names*. What we'll log then will not be the contents of
the `static` variables but their addresses.
 -->

やることは、次の通りです。ログメッセージごとに`static`変数を1つ作りますが、
メッセージをその変数の*中に*格納せずに、変数の*シンボル名*にメッセージを格納します。
ログ出力するものは、`static`変数の内容ではなく、そのアドレスです。

<!-- 
As long as the `static` variables are not zero sized each one will have a
different address. What we're doing here is effectively encoding each message
into a unique identifier, which happens to be the variable address. Some part of
the log system will have to decode this id back into the message.
 -->

`static`変数のサイズがゼロでない限り、各変数は異なるアドレスを持ちます。
ここで行うことは、各メッセージを一意の識別子（変数のアドレスになります）に効率的にエンコードすることです。
ログシステムの一部は、この識別子をメッセージにデコードしなければなりません。

<!-- Let's write some code to illustrate the idea. -->

このアイデアを実現するコードを書いていきましょう。

<!-- 
In this example we'll need some way to do I/O so we'll use the
[`cortex-m-semihosting`] crate for that. Semihosting is a technique for having a
target device borrow the host I/O capabilities; the host here usually refers to
the machine that's debugging the target device. In our case, QEMU supports
semihosting out of the box so there's no need for a debugger. On a real device
you'll have other ways to do I/O like a serial port; we use semihosting in this
case because it's the easiest way to do I/O on QEMU.
 -->

この例では、I/Oが必要なため、[`cortex-m-semihosting`]クレートを使用します。
セミホスティングは、ターゲットデバイスがホストのI/O機能を借りられるようにするための技術です。
今回の場合、QEMUは細かい設定なしでセミホスティングが使えるため、デバッガは不要です。
実機の場合、シリアルポートのようなI/Oが必要になります。
QEMU上でI/Oを使う最も簡単な方法であるため、今回はセミホスティングを使います。

[`cortex-m-semihosting`]: https://crates.io/crates/cortex-m-semihosting

<!-- Here's the code -->

コードは次のとおりです。

``` rust
{{#include ../ci/logging/app/src/main.rs}}
```

<!-- 
We also make use of the `debug::exit` API to have the program terminate the QEMU
process. This is a convenience so we don't have to manually terminate the QEMU
process.
 -->

プログラムがQEMUプロセスを終了できるようにするため、`debug::exit`も使えるようにしてあります。
QEMUプロセスを手動で終了しなくて良いため、便利です。

<!-- 
And here's the `dependencies` section of the Cargo.toml:
 -->

そして、こちらはCargo.tomlの`dependencies`セクションです。

``` toml
{{#include ../ci/logging/app/Cargo.toml:7:9}}
```

<!-- Now we can build the program -->

これでプログラムをビルドできます。

``` console
$ cargo build
```

<!-- 
To run it we'll have to add the `--semihosting-config` flag to our QEMU
invocation:
 -->

実行するためには、QEMU起動時に、`--semihosting-config`フラグを付け加えます。

``` console
$ qemu-system-arm \
      -cpu cortex-m3 \
      -machine lm3s6965evb \
      -nographic \
      -semihosting-config enable=on,target=native \
      -kernel target/thumbv7m-none-eabi/debug/app
```

``` text
{{#include ../ci/logging/app/dev.out}}
```

<!-- 
> **NOTE**: These addresses may not be the ones you get locally because
> addresses of `static` variable are not guaranteed to remain the same when the
> toolchain is changed (e.g. optimizations may have improved).
 -->

> **注記** これらのアドレスは、あなたが得たアドレスと異なるかもしれません。
> `static`変数のアドレスは、
> ツールチェインが更新された時（例えば、最適化が改善されるかもしれません）に変わる可能性があるからです。

<!-- 
Now we have two addresses printed to the console.
 -->

コンソールに2つのアドレスが表示されました。

<!-- ## Decoding -->

## デコードする

<!-- 
How do we convert these addresses into strings? The answer is in the symbol
table of the ELF file.
 -->

どのようにして、このアドレスを文字列に変換するのでしょうか？
答えはELFファイルのシンボルテーブルです。

``` console
$ cargo objdump --bin app -- -t | grep '\.rodata\s*0*1\b'
```

``` text
{{#include ../ci/logging/app/dev.objdump}}
$ # 1列目はシンボルのアドレス、最終列はシンボル名です。
```

<!-- 
`objdump -t` prints the symbol table. This table contains *all* the symbols but
we are only looking for the ones in the `.rodata` section and whose size is one
byte (our variables have type `u8`).
 -->

`objdump -t`はシンボルテーブルを表示します。このテーブルは*全ての*シンボルを含んでいますが、
`.rodata`セクションの中にある1バイトの大きさ（変数の型は`u8`です）のアドレスだけを詳しく見ていきます。

<!-- 
It's important to note that the address of the symbols will likely change when
optimizing the program. Let's check that.
 -->

プログラムを最適化すると、シンボルのアドレスが変わる可能性があるため、注意して下さい。
確認してみましょう。

<!-- 
> **PROTIP** You can set `target.thumbv7m-none-eabi.runner` to the long QEMU
> command from before (`qemu-system-arm -cpu (..) -kernel`) in the Cargo
> configuration file (`.cargo/conifg`) to have `cargo run` use that *runner* to
> execute the output binary.
 -->

> **専門家によるアドバイス** `target.thumbv7m-none-eabi.runner`を、
> 長いQEMUコマンド（`qemu-system-arm -cpu (..) -kernel`）に設定することができます。
> Cargo設定ファイル（`.cargo/config`）にコマンドを書くことで、
> `cargo run`がその*ランナー*を使ってバイナリを実行します。

``` console
$ head -n2 .cargo/config
```

``` toml
{{#include ../ci/logging/app/.cargo/config:1:2}}
```

``` console
$ cargo run --release
     Running `qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/release/app`
```

``` text
{{#include ../ci/logging/app/release.out}}
```

``` console
$ cargo objdump --bin app --release -- -t | grep '\.rodata\s*0*1\b'
```

``` text
{{#include ../ci/logging/app/release.objdump}}
```

<!-- 
So make sure to always look for the strings in the ELF file you executed.
 -->

常に実行したELFファイル内の文字列を探すようにして下さい。

<!-- 
Of course, the process of looking up the strings in the ELF file can be automated
using a tool that parses the symbol table (`.symtab` section) contained in the
ELF file. Implementing such tool is out of scope for this book and it's left as
an exercise for the reader.
 -->

もちろん、ELFファイルに含まれるシンボルテーブル（`.symtab`セクション）を解析するツールを使うことで、
ELFファイル内の文字列を探すプロセスを自動化することが可能です。
そのようなツールを実装することは、本書のスコープ外です。
そのため、読者の演習とします。

<!-- ## Making it zero cost -->

## ゼロコストにする

<!-- 
Can we do better? Yes, we can!
 -->

より良いものにできるでしょうか？もちろんできます！

<!-- 
The current implementation places the `static` variables in `.rodata`, which
means they occupy size in Flash even though we never use their contents. Using a
little bit of linker script magic we can make them occupy *zero* space in Flash.
 -->

現在の実装は、`static`変数を`.rodata`に配置しています。これは、その変数の値を決して使わないにも関わらず、
Flashの容量を専有することを意味します。
リンカスクリプトの魔法を少し使うことで、Flashの使用量を*ゼロ*にできます。

``` console
$ cat log.x
```

``` text
{{#include ../ci/logging/app2/log.x}}
```

<!-- 
We'll place the `static` variables in this new output `.log` section. This
linker script will collect all the symbols in the `.log` sections of input
object files and put them in an output `.log` section. We have seen this pattern
in the [Memory layout] chapter.
 -->

`static`変数を新しい`.log`出力セクションに配置します。
このリンカスクリプトは、入力オブジェクトファイルの`.log`セクションにある全てのシンボルを集め、
`.log`出力セクションに置きます。
このパターンは、[メモリレイアウト]の章でやりました。

<!-- [Memory layout]: /memory-layout.html -->

[メモリレイアウト]: /memory-layout.html

<!-- 
The new bit here is the `(INFO)` part; this tells the linker that this section
is a non-allocatable section. Non-allocatable sections are kept in the ELF
binary as metadata but they are not loaded onto the target device.
 -->

少し新しい部分は、`(INFO)`の部分です。これは、リンカに、このセクションは割当不可セクションであることを教えます。
割当不可セクションは、ELFバイナリにメタデータとして残りますが、ターゲットデバイスにはロードされません。

<!-- 
We also specified the start address of this output section: the `0` in `.log 0
(INFO)`.
 -->

また、この出力セクションの開始アドレスを`.log 0 (INFO)`で`0`に指定しています。

<!-- 
The other improvement we can do is switch from formatted I/O (`fmt::Write`) to
binary I/O, that is send the addresses to the host as bytes rather than as
strings.
 -->

他に改善点は、フォーマットされたI/O（`fmt::Write`）から、バイナリI/Oに切り替えることです。
つまり、文字列としてではなく、バイトとしてホストにアドレスを送ります。

<!-- 
Binary serialization can be hard but we'll keep things super simple by
serializing each address as a single byte. With this approach we don't have to
worry about endianness or framing. The downside of this format is that a single
byte can only represent up to 256 different addresses.
 -->

バイナリシリアライゼーションは、複雑になる可能性がありますが、各アドレスを1バイトとしてシリアライズすることで、
極めて簡潔になります。この方法により、エンディアネスやフレーム化について悩まなくて済みます。
この形式の欠点は、1バイトは256のアドレスしか表現できないことです。

<!-- Let's make those changes: -->

これらの変更を加えましょう。

``` rust
{{#include ../ci/logging/app2/src/main.rs}}
```

<!-- 
Before you run this you'll have to append `-Tlog.x` to the arguments passed to
the linker. That can be done in the Cargo configuration file.
 -->

実行する前に、リンカに渡す引数に`-Tlog.x`を追加しなければなりません。
Cargo設定ファイルで、追加できます。

``` console
$ cat .cargo/config
```

``` toml
{{#include ../ci/logging/app2/.cargo/config}}
```

<!-- 
Now you can run it! Since the output now has a binary format we'll pipe it
through the `xxd` command to reformat it as a hexadecimal string.
 -->

これで実行することができます！今回、出力はバイナリ形式であるため、
`xxd`コマンドにパイプし、16進数の文字列に再変換します。

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/logging/app2/dev.out}}
```

<!-- 
The addresses are `0x00` and `0x01`. Let's now look at the symbol table.
 -->

アドレスは、`0x00`と`0x01`です。では、シンボルテーブルを見てみましょう。

``` console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/logging/app2/dev.objdump}}
```

<!-- 
There are our strings. You'll notice that their addresses now start at zero;
this is because we set a start address for the output `.log` section.
 -->

目的の文字列があります。今回は、アドレスがゼロから開始していることに気づくでしょう。
これは、`.log`出力セクションに、開始アドレスを設定したためです。

<!-- 
Each variable is 1 byte in size because we are using `u8` as their type. If we
used something like `u16` then all address would be even and we would not be
able to efficiently use all the address space (`0...255`).
 -->

`u8`を型として使っているため、各変数は1バイトの大きさです。
もし`u16`のような型を使った場合、全てのアドレスは偶数になり、全てのアドレス空間（`0...255`）を、
効率的に利用することができないでしょう。

<!-- ## Packaging it up -->

## パッケージする

<!-- 
You've noticed that the steps to log a string are always the same so we can
refactor them into a macro that lives in its own crate. Also, we can make the
logging library more reusable by abstracting the I/O part behind a trait.
 -->

文字列をログ出力するステップは、常に一緒です。そこで、
クレート内でだけ利用可能なマクロにリファクタリングします。
また、I/O部分をトレイトで抽象化することで、ロギングライブラリをより再利用可能にできます。

``` console
$ cargo new --lib log

$ cat log/src/lib.rs
```

``` rust
{{#include ../ci/logging/log/src/lib.rs}}
```

<!-- 
Given that this library depends on the `.log` section it should be its
responsibility to provide the `log.x` linker script so let's make that happen.
 -->

このライブラリが`.log`セクションに依存することを考えると、このライブラリが`log.x`リンカスクリプトの提供に責任を持つべきです。
それでは、そうしましょう。

``` console
$ mv log.x ../log/
```

``` console
$ cat ../log/build.rs
```

``` rust
{{#include ../ci/logging/log/build.rs}}
```

<!-- 
Now we can refactor our application to use the `log!` macro:
 -->

それでは、`log!`マクロを使って、アプリケーションをリファクタリングしましょう。

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/logging/app3/src/main.rs}}
```

<!-- 
Don't forget to update the `Cargo.toml` file to depend on the new `log` crate.
 -->

新しい`log`クレートへの依存を、`Cargo.toml`に追加するのを忘れないようにしましょう。

``` console
$ tail -n4 Cargo.toml
```

``` toml
{{#include ../ci/logging/app3/Cargo.toml:7:10}}
```

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/logging/app3/dev.out}}
```

``` console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/logging/app3/dev.objdump}}
```

<!-- Same output as before! -->

以前と同じ出力になりました！

<!-- ## Bonus: Multiple log levels -->

## おまけ：複数のログレベル

<!-- 
Many logging frameworks provide ways to log messages at different *log levels*.
These log levels convey the severity of the message: "this is an error", "this
is just a warning", etc. These log levels can be used to filter out unimportant
messages when searching for e.g. error messages.
 -->

多くのログフレームワークは、異なる*ログレベル*でメッセージをロギングする方法を提供しています。
これらのログレベルは、メッセージの重要度を告げています。「これはエラーです」、「これはただの警告です」、など。
これらのログレベルは、例えばエラーメッセージを検索する時に、重要でないメッセージを除去するために使用されます。

<!-- 
We can extend our logging library to support log levels without increasing its
footprint. Here's how we'll do that:
 -->

私達のログライブラリを、フットプリントの増加なしに、ログレベルをサポートするように拡張できます。
やることは、次の通りです。

<!-- 
We have a flat address space for the messages: from `0` to `255` (inclusive). To
keep things simple let's say we only want to differentiate between error
messages and warning messages. We can place all the error messages at the
beginning of the address space, and all the warning messages *after* the error
messages. If the decoder knows the address of the first warning message then it
can classify the messages. This idea can be extended to support more than two
log levels.
 -->

メッセージ用に、0以上、255以下のフラットなアドレス空間があります。
簡単化のために、エラーメッセージと警告メッセージを区別したいだけ、としましょう。
全てのエラーメッセージをアドレス空間の最初に置き、警告メッセージをエラーメッセージの*後*に置きます。
デコーダが最初の警告メッセージのアドレスを知っていれば、メッセージを分類可能です。
このアイデアは、3つ以上のログレベルをサポートするときに拡張できます。

<!-- 
Let's test the idea by replacing the `log` macro with two new macros: `error!`
and `warn!`.
 -->

`log`マクロを、`error!`と`warn!`の2つの新しいマクロで置き換えて、このアイデアを試してみましょう。

``` console
$ cat ../log/src/lib.rs
```

``` rust
{{#include ../ci/logging/log2/src/lib.rs}}
```

<!-- 
We distinguish errors from warnings by placing the messages in different link
sections.
 -->

メッセージを異なるリンクセクションに配置することでエラーと警告を区別します。

<!-- 
The next thing we have to do is update the linker script to place error messages
before the warning messages.
 -->

次にやらなければならないことは、エラーメッセージを警告メッセージの前に配置するように、
リンカスクリプトを更新することです。

``` console
$ cat ../log/log.x
```

``` text
{{#include ../ci/logging/log2/log.x}}
```

<!-- 
We also give a name, `__log_warning_start__`, to the boundary between the errors
and the warnings. The address of this symbol will be the address of the first
warning message.
 -->

エラーと警告との境界に、`__log_warning_start__`という名前をつけています。
このシンボルのアドレスは、最初の警告メッセージのアドレスになります。

<!-- 
We can now update the application to make use of these new macros.
 -->

次に、これらの新しいマクロを使うように、アプリケーションを更新します。

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/logging/app4/src/main.rs}}
```

<!-- The output won't change much: -->

出力は、それほど変わりません。

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/logging/app4/dev.out}}
```

<!-- 
We still get two bytes in the output but the error is given the address 0 and
the warning is given the address 1 even though the warning was logged first.
 -->

相変わらず2バイトの出力が得られています。
しかし、警告が最初にログ出力されているにも関わらず、エラーが0番地、警告が1番地になっています。

<!-- Now look at the symbol table. -->

それでは、シンボルテーブルを見てみます。

```  console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/logging/app4/dev.objdump}}
```

<!-- 
There's now an extra symbol, `__log_warning_start__`, in the `.log` section.
The address of this symbol is the address of the first warning message.
Symbols with addresses lower than this value are errors, and the rest of symbols
are warnings.
 -->

`.log`セクション内に`__log_warning_start__`という追加のシンボルがあります。
このシンボルのアドレスは、最初の警告メッセージのアドレスです。
この値より小さいアドレスを持つシンボルは、エラーになります。
それ以外のシンボルは警告です。

<!-- 
With an appropriate decoder you could get the following human readable output
from all this information:
 -->

適切なデコーダを使うと、これら全ての情報から、次の人間が読みやすい出力を得ることができます。

``` text
WARNING Hello, world!
ERROR Goodbye
```

---

<!-- 
If you liked this section check out the [`stlog`] logging framework which is a
complete implementation of this idea.
 -->

このセクションを気に入った場合、[`stlog`]ログフレームワークを確認して下さい。
このアイデアを完全に実装しています。

[`stlog`]: https://crates.io/crates/stlog
