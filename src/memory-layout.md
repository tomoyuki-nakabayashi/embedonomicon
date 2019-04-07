<!-- # Memory layout -->

# メモリレイアウト

<!-- 
The next step is to ensure the program has the right memory layout so that the target system will be
able to execute it. In our example, we'll be working with a virtual Cortex-M3 microcontroller: the
[LM3S6965]. Our program will be the only process running on the device so it must also take care of
initializing the device.
 -->

次のステップは、ターゲットシステムがプログラムを実行できるように、プログラムに正しいメモリレイアウトを持たせることです。
例では、[LM3S6965]という仮想のCortex-M3マイクロコントローラを取り扱います。
私達のプログラムは、デバイス上で動作する唯一のプロセスになります。そのため、デバイスの初期化も面倒を見る必要があります。

<!-- ## Background information -->

## 背景となる情報

[LM3S6965]: http://www.ti.com/product/LM3S6965

<!-- 
Cortex-M devices require a [vector table] to be present at the start of their [code memory region].
The vector table is an array of pointers; the first two pointers are required to boot the device;
the rest of pointers are related to exceptions -- we'll ignore them for now.
 -->

Cortex-Mデバイスは、[コードメモリ領域]の開始地点に[ベクタテーブル]があること、を要求します。
ベクタテーブルはポインタの配列です。最初の2つのポインタは、デバイスが起動するときに必要です。
残りのポインタは例外に関係するもので、今は無視します。

<!-- 
[code memory region]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/memory-model
[vector table]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table
 -->

[コードメモリ領域]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/memory-model
[ベクタテーブル]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

<!-- 
Linkers decide the final memory layout of programs, but we can use [linker scripts] to have some
control over it. The control granularity that linker scripts give us over the layout
is at the level of *sections*. A section is a collection of *symbols* laid out in contiguous memory.
Symbols, in turn, can be data (a static variable), or instructions (a Rust function).
 -->

リンカは、プログラムの最終的なメモリレイアウトを決定します。しかし、[リンカスクリプト]を使うことで、メモリレイアウトを制御できます。
リンカスクリプトによる制御の粒度は、*セクション*レベルです。セクションは、連続したメモリに置かれる*シンボル*の集まりです。
ここで、シンボルはデータ（静的変数）か命令（Rustの関数）になります。

<!-- 
[linker scripts]: https://sourceware.org/binutils/docs/ld/Scripts.html
 -->

[リンカスクリプト]: https://sourceware.org/binutils/docs/ld/Scripts.html

<!-- 
Every symbol has a name assigned by the compiler. As of Rust 1.28 , the Rust compiler assigns to
symbols names of the form: `_ZN5krate6module8function17he1dfc17c86fe16daE`, which demangles to
`krate::module::function::he1dfc17c86fe16da` where `krate::module::function` is the path of the
function or variable and `he1dfc17c86fe16da` is some sort of hash. The Rust compiler will place each
symbol into its own and unique section; for example the symbol mentioned before will be placed in a
section named `.text._ZN5krate6module8function17he1dfc17c86fe16daE`.
 -->

全てのシンボルは、コンパイラによって割り当てられた名前を持ちます。Rust 1.28以降では、Rustコンパイラは、
`_ZN5krate6module8function17he1dfc17c86fe16daE`、のような形式でシンボル名を割り当てます。
このシンボルは、`krate::module::function::he1dfc17c86fe16da`にデマングルできます。
ここで、`krate::module::function`は、関数か変数のパスです。そして、`he1dfc17c86fe16da`は何らかのハッシュです。
Rustコンパイラは、各シンボルをシンボル固有のセクションに配置します。例えば、上述したシンボルは、
`.text._ZN5krate6module8function17he1dfc17c86fe16daE`というセクションの配置されます。

<!-- 
These compiler generated symbol and section names are not guaranteed to remain constant across
different releases of the Rust compiler. However, the language lets us control symbol names and
section placement via these attributes:
 -->

コンパイラが生成したシンボル名とセクション名は、Rustコンパイラのリリースごとに変わる可能性があります。
しかし、次のアトリビュートを使って、シンボル名やセクション配置を制御することができます。

<!-- 
- `#[export_name = "foo"]` sets the symbol name to `foo`.
- `#[no_mangle]` means: use the function or variable name (not its full path) as its symbol name.
  `#[no_mangle] fn bar()` will produce a symbol named `bar`.
- `#[link_section = ".bar"]` places the symbol in a section named `.bar`.
 -->

- `#[export_name = "foo"]`は、シンボル名を`foo`に設定します。
- `#[no_mangle]`は、関数名や変数名を（フルパスではなく）シンボル名として使うことを意味します。`#[no_mangle] fn bar()`は、`bar`というシンボル名を生成します。
- `#[link_section = ".bar"]`は、シンボルを`.bar`という名前のセクションに配置します。

<!-- 
With these attributes, we can expose a stable ABI of the program and use it in the linker script.
 -->

これらのアトリビュートにより、プログラムの安定的なABIを公開することができ、リンカスクリプトで利用することができます。

<!-- ## The Rust side -->

## Rust側

<!-- 
Like mentioned before, for Cortex-M devices, we need to populate the first two entries of the
vector table. The first one, the initial value for the stack pointer, can be populated using
only the linker script. The second one, the reset vector, needs to be created in Rust code
and placed correctly using the linker script.
 -->

上述の通り、Cortex-Mデバイスに対して、ベクタテーブルの最初の2つのエントリを配置する必要があります。
1つ目は、スタックポインタの初期値で、リンカスクリプトだけを使って配置することができます。
2つ目のリセットベクタは、Rustのコードを作成する必要があり、リンカスクリプトを使って正しく配置しなければなりません。

<!-- 
The reset vector is a pointer into the reset handler. The reset handler is the function that the
device will execute after a system reset, or after it powers up for the first time. The reset
handler is always the first stack frame in the hardware call stack; returning from it is undefined
behavior as there's no other stack frame to return to. We can enforce that the reset handler never
returns by making it a divergent function, which is a function with signature `fn(/* .. */) -> !`.
 -->

リセットベクタは、リセットハンドラのポインタです。リセットハンドラは、デバイスがシステムリセットの後、もしくは、
最初に電源が入った後に実行する関数です。リセットハンドラは、常にハードウェアコールスタックの最初のスタックフレームになります。
戻るためのスタックフレームがないため、リセットハンドラから戻ることは、未定義動作です。
発散関数のマーキングを行うことで、リセットハンドラが決して戻らないように強制できます。
発散関数は、`fn(/* .. */) -> !`というシグネチャがついた関数です。

``` rust
{{#include ../ci/memory-layout/src/main.rs:7:19}}
```

<!-- 
The hardware expects a certain format here, to which we adhere by using `extern "C"` to tell the
compiler to lower the function using the C ABI, instead of the Rust ABI, which is unstable.
 -->

ここで、ハードウェアは、特定の形式を期待しています。これに従うため、`extern "C"`を使うことで、コンパイラがこの関数をC ABIを使うように指示します。
そうしなければ、安定していないRust ABIが使われます。

<!-- 
To refer to the reset handler and reset vector from the linker script, we need them to have a stable
symbol name so we use `#[no_mangle]`. We need fine control over the location of `RESET_VECTOR`, so we
place it in a known section, `.vector_table.reset_vector`. The exact location of the reset handler
itself, `Reset`, is not important. We just stick to the default compiler generated section.
 -->

リンカスクリプトからリセットハンドラとリセットベクタを参照するために、`#[no_mangle]`を使って安定したシンボル名を与えます。
`RESET_VECTOR`の位置を細かく制御しなければなりません。そこで、`.vector_table.reset_vector`と呼ばれるセクションに配置します。
リセットハンドラである`Reset`自身の正確な位置は重要ではありません。これに対しては、デフォルトでコンパイラが生成するセクションを使用します。

<!-- 
Also, the linker will ignore symbols with internal linkage, AKA internal symbols, while traversing
the list of input object files, so we need our two symbols to have external linkage. The only way to
make a symbol external in Rust is to make its corresponding item public (`pub`) and *reachable* (no
private module between the item and the root of the crate).
 -->

また、入力のオブジェクトファイルを解析する間、リンカは、内部シンボルと呼ばれる内部リンケージのシンボルを無視します。
そこで、2つのシンボルが外部リンケージを持つようにする必要があります。Rustでシンボルを外部向けにする唯一の方法は、
関連するアイテムをpublic (`pub`) にして、*到達可能*（アイテムとクレートのトップレベルとの間にプライベートなモジュールがない）なものにすることです。

<!-- ## The linker script side -->

## リンカスクリプト側

<!-- 
Below is shown a minimal linker script that places the vector table in the right location. Let's
walk through it.
 -->

下記に、正しい位置にベクタテーブルを配置する最小限のリンカスクリプトを示します。
全体に目を通してみましょう。

``` console
$ cat link.x
```

``` text
{{#include ../ci/memory-layout/link.x}}
```

### `MEMORY`

<!-- 
This section of the linker script describes the location and size of blocks of memory in the target.
Two memory blocks are defined: `FLASH` and `RAM`; they correspond to the physical memory available
in the target. The values used here correspond to the LM3S6965 microcontroller.
 -->

リンカスクリプトのこのセクションは、ターゲット内のメモリブロックの位置とサイズを記述します。
2つのメモリブロックが定義されています。`FLASH`と`RAM`です。これらは、ターゲットで利用可能な物理メモリと関連しています。
ここで使用されている値は、LM3S6965マイクロコントローラのものです。

### `ENTRY`

<!-- 
Here we indicate to the linker that the reset handler -- whose symbol name is `Reset` -- is the
*entry point* of the program. Linkers aggressively discard unused sections. Linkers consider the
entry point and functions called from it as *used* so they won't discard them. Without this line,
the linker would discard the `Reset` function and all subsequent functions called from it.
 -->

ここでは、リンカに`Reset`というシンボル名を持つリセットハンドラが、プログラムの*エントリポイント*であることを教えています。
リンカは、不要なセクションを積極的に破棄します。リンカは、エントリポイントと、エントリポイント関数から呼ばれる関数を*使用される*と考え、
破棄しなくなります。この行がないと、リンカは、`Reset`関数と、そこから呼ばれる全ての関数を破棄するでしょう。

### `EXTERN`

<!-- 
Linkers are lazy; they will stop looking into the input object files once they have found all the
symbols that are recursively referenced from the entry point. `EXTERN` forces the linker to look
for `EXTERN`'s argument even after all other referenced symbols have been found. As a rule of thumb,
if you need a symbol that's not called from the entry point to always be present in the output binary,
you should use `EXTERN` in conjunction with `KEEP`.
 -->

リンカは怠け者です。エントリポイントから再帰的に参照されるシンボルが全て見つかった時点で、入力オブジェクトファイルの解析を停止します。
`EXTERN`により、他の参照されるシンボルが全て見つかった後でも、リンカは`EXTERN`の引数が見つかるまで探し続けます。
基本、エントリポイントから呼ばれないシンボルが出力バイナリで必要な場合、`KEEP`と関連付けて`EXTERN`を使う必要があります。

### `SECTIONS`

<!-- 
This part describes how sections in the input object files, AKA *input sections*, are to be arranged
in the sections of the output object file, AKA output sections; or if they should be discarded. Here
we define two output sections:
 -->

ここでは、入力オブジェクトファイル内のセクション（*入力セクション*）がどのように出力オブジェクトファイルのセクション（出力セクション）に配置されるのか、
もしくは破棄されるのか、を説明します。
2つの出力セクションを定義します。

``` text
  .vector_table ORIGIN(FLASH) : { /* .. */ } > FLASH
```

<!-- 
`.vector_table`, which contains the vector table and is located at the start of `FLASH` memory,
 -->

`.vector_table`は、ベクタテーブルを含んでおり、`FLASH`メモリの開始地点に配置されます。

``` text
  .text : { /* .. */ } > FLASH
```

<!-- 
and `.text`, which contains the program subroutines and is located somewhere in `FLASH`. Its start
address is not specified, but the linker will place it after the previous output section,
`.vector_table`.
 -->

そして、`.text`は、プログラムのサブルーチンを含んでおり、`FLASH`のどこかに配置されます。開始アドレスは指定されていませんが、
リンカは直前の出力セクションである`.vector_table`の後ろに、このセクションを配置するでしょう。

<!-- The output `.vector_table` section contains: -->

出力セクションの`.vecotr_table`は、次のものを含んでいます。

``` text
{{#include ../ci/memory-layout/link.x:18:19}}
```

<!-- 
We'll place the (call) stack at the end of RAM (the stack is *full descending*; it grows towards
smaller addresses) so the end address of RAM will be used as the initial Stack Pointer (SP) value.
That address is computed in the linker script itself using the information we entered for the `RAM`
memory block.
 -->

（コール）スタックをRAMの最後に配置します。スタックは、*完全な降順*です。すなわち、小さいアドレスに向かって伸びます。
そのため、RAMの最後のアドレスをスタックポインタ（SPモード）の初期値として使用します。
このアドレスは、リンカスクリプト内で`RAM`メモリブロックに入力した情報を使って、計算されます。

```
{{#include ../ci/memory-layout/link.x:21:22}}
```

<!-- 
Next, we use `KEEP` to force the linker to insert all input sections named
`.vector_table.reset_vector` right after the initial SP value. The only symbol located in that
section is `RESET_VECTOR`, so this will effectively place `RESET_VECTOR` second in the vector table.
 -->

次に、SPの初期値の直後に`.vector_table.reset_vector`と名付けられた全ての入力セクションがリンカによって挿入されるように、`KEEP`を使います。
`RESET_VECTOR`がこのセクションに配置される唯一のシンボルです。これは、ベクタテーブルの2つ目に`RESET_VECTOR`を配置するのに効率的な方法です。

<!-- The output `.text` section contains: -->

出力セクションの`.text`は、次の内容を含んでいます。

``` text
{{#include ../ci/memory-layout/link.x:27}}
```

<!-- 
This includes all the input sections named `.text` and `.text.*`. Note that we don't use `KEEP`
here to let the linker discard unused sections.
 -->

これは、`.text`と`.text.*`という名前の入力セクションを全て含んでいます。
リンカが不必要なセクションを破棄しないようにさせるための、`KEEP`を、ここでは使わないことに留意して下さい。

<!-- Finally, we use the special `/DISCARD/` section to discard -->

最後に、破棄用の特別な`/DISCARD/`セクションを使います。

``` text
{{#include ../ci/memory-layout/link.x:32}}
```

<!-- 
input sections named `.ARM.exidx.*`. These sections are related to exception handling but we are not
doing stack unwinding on panics and they take up space in Flash memory, so we just discard them.
 -->

`.ARM.exidx.*`という入力セクションを破棄します。これらのセクションは、例外処理に関連したものですが、
パニック時のスタック巻き戻しを行わないのと、これらのセクションはFlashメモリの容量を使うため、単に破棄します。

<!-- ## Putting it all together -->

## 1つにまとめる

<!-- Now we can link the application. For reference, here's the complete Rust program: -->

これで、アプリケーションをリンクできます。参考用に、完全なRustプログラムを示します。

``` rust
{{#include ../ci/memory-layout/src/main.rs}}
```

<!-- 
We have to tweak linker process to make it use our linker script. This is done
passing the `-C link-arg` flag to `rustc` but there are two ways to do it: you
can use the `cargo-rustc` subcommand instead of `cargo-build` as shown below:
 -->

私達のリンカスクリプトを使うために、リンカプロセスに手を加えなければなりません。これは、`rustc`に`-C link-arg`フラグを渡すことで達成できます。
しかし、2つのやり方があります。下記のように`cargo-rustc`サブコマンドを`cargo-build`の代わりに使用することができます。

<!-- 
**IMPORTANT**: Make sure you have the `.cargo/config` file that was added at the
end of the last section before running this command.
 -->

**重要**：このコマンドを実行する前に、前回のセクションの最後に追加した`.cargo/config`ファイルがあることを確認して下さい。

``` console
$ cargo rustc -- -C link-arg=-Tlink.x
```

<!-- 
Or you can set the rustflags in `.cargo/config` and continue using the
`cargo-build` subcommand. We'll do the latter because it better integrates with
`cargo-binutils`.
 -->

もしくは、`.cargo/config`にrustflagsを設定し、`cargo-build`サブコマンドを使い続けることもできます。
`cargo-binutils`との統合がやりやすいため、2つ目の方法を使います。

``` console
# .cargo/configを次の内容で修正します
$ cat .cargo/config
```

``` toml
{{#include ../ci/memory-layout/.cargo/config}}
```

<!-- 
The `[target.thumbv7m-none-eabi]` part says that these flags will only be used
when cross compiling to that target.
 -->

`[target.thumbv7m-none-eabi]`の部分は、このフラグがターゲット向けのクロスコンパイル時のみ有効であることを意味しています。

<!-- ## Inspecting it -->

## 調査

<!-- 
Now let's inspect the output binary to confirm the memory layout looks the way we want:
 -->

それでは、望み通りのメモリレイアウトになっているか確認するため、出力バイナリを調査してみましょう。

``` console
$ cargo objdump --bin app -- -d -no-show-raw-insn
```

``` text
{{#include ../ci/memory-layout/app.text.objdump}}
```

<!-- 
This is the disassembly of the `.text` section. We see that the reset handler, named `Reset`, is
located at address `0x8`.
 -->

これは`.text`セクションの逆アセンブリです。`Reset`というリセットハンドラが`0x8`番地に位置していることがわかります。

``` console
$ cargo objdump --bin app -- -s -section .vector_table
```

``` text
{{#include ../ci/memory-layout/app.vector_table.objdump}}
```

<!-- 
This shows the contents of the `.vector_table` section. We can see that the section starts at
address `0x0` and that the first word of the section is `0x2001_0000` (the `objdump` output is in
little endian format). This is the initial SP value and matches the end address of RAM. The second
word is `0x9`; this is the *thumb mode* address of the reset handler. When a function is to be
executed in thumb mode the first bit of its address is set to 1.
 -->

これは、`.vector_table`セクションの内容を示しています。セクションは`0x0`番地から開始しており、セクションの1つ目のワードは、`0x2001_0000`であることがわかります
（`objdump`はリトリエンディアン形式で出力します）。これはSPの初期値で、RAMの最後のアドレスと一致します。
2つ目のワードは`0x9`です。これは、リセットハンドラの*thumbモード*アドレスです。
関数がthumbモードで実行される場合、そのアドレスの1ビット目は1に設定されます。

<!-- ## Testing it -->

## テスト

<!-- 
This program is a valid LM3S6965 program; we can execute it in a virtual microcontroller (QEMU) to
test it out.
 -->
このプログラムは、有効なLM3S6965プログラムです。このプログラムをテストするため、仮想のマイクロコントローラ（QEMU）で実行できます。

``` console
$ # this program will block
$ qemu-system-arm \
      -cpu cortex-m3 \
      -machine lm3s6965evb \
      -gdb tcp::3333 \
      -S \
      -nographic \
      -kernel target/thumbv7m-none-eabi/debug/app
```

``` console
$ # 別ターミナル
$ arm-none-eabi-gdb -q target/thumbv7m-none-eabi/debug/app
Reading symbols from target/thumbv7m-none-eabi/debug/app...done.

(gdb) target remote :3333
Remote debugging using :3333
Reset () at src/main.rs:8
8       pub unsafe extern "C" fn Reset() -> ! {

(gdb) # SPがベクタテーブルにプログラムした初期値を持っています
(gdb) print/x $sp
$1 = 0x20010000

(gdb) step
9           let _x = 42;

(gdb) step
12          loop {}

(gdb) # 次にスタック変数の`_x`を調査します
(gdb) print _x
$2 = 42

(gdb) print &_x
$3 = (i32 *) 0x2000fffc

(gdb) quit
```
