<!-- # Exception handling -->

## 例外処理

<!-- 
During the "Memory layout" section, we decided to start out simple and leave out handling of
exceptions. In this section, we'll add support for handling them; this serves as an example of
how to achieve compile time overridable behavior in stable Rust (i.e. without relying on the
unstable `#[linkage = "weak"]` attribute, which makes a symbol weak).
 -->

「メモリレイアウト」セクションでは、簡単なところから始め、例外処理を省くことにしました。
このセクションでは、例外処理サポートを追加します。stable Rustでコンパル時にオーバーライド可能な振る舞いを実装する例を示します
（すなわち、シンボルをウィークにするunstableの`#[linkage = "weak"]`アトリビュートに頼りません）。

<!-- ## Background information -->

## 背景となる情報

<!-- 
In a nutshell, *exceptions* are a mechanism the Cortex-M and other architectures provide to let
applications respond to asynchronous, usually external, events. The most prominent type of exception,
that most people will know, is the classical (hardware) interrupt.
 -->

一言で言えば、*例外*は、アプリケーションが（主に外部からの）非同期イベントに応答するための、
Cortex-Mや他のアーキテクチャが提供する機構です。最も有名なほとんどの人々が知っているであろう例外の種別は、
古典的な（ハードウェア）割り込みです。

<!-- 
The Cortex-M exception mechanism works like this:
When the processor receives a signal or event associated to a type of exception, it suspends
the execution of the current subroutine (by stashing the state in the call stack) and then proceeds
to execute the corresponding exception handler, another subroutine, in a new stack frame. After
finishing the execution of the exception handler (i.e. returning from it), the processor resumes the
execution of the suspended subroutine.
 -->

Cortex-Mの例外機能は次のように動きます。
プロセッサが例外の種別に応じたシグナルもしくはイベントを受信すると、
（コールスタックに現在の状態を入れておくことで）現在のサブルーチンの実行を一時停止し、
関連する例外ハンドラ（別のサブルーチン）の実行を新しいスタックフレームで開始します。
例外ハンドラの実行が終了すると（つまり例外ハンドラから戻ると）、プロセッサは一時停止したサブルーチンの実行を再開します。

<!-- 
The processor uses the vector table to decide what handler to execute. Each entry in the table
contains a pointer to a handler, and each entry corresponds to a different exception type. For
example, the second entry is the reset handler, the third entry is the NMI (Non Maskable Interrupt)
handler, and so on.
 -->

プロセッサはどのハンドラを実行するか、を決めるためにベクタテーブルを使います。テーブルの各エントリはハンドラへのポインタです。
そして、各エントリは、異なる例外種別に対応しています。例えば、2つ目のエントリはリセットハンドラで、3つ目のエントリは、
NMI（Non Maskable Interrupt）と言った具合です。

<!-- 
As mentioned before, the processor expects the vector table to be at some specific location in memory,
and each entry in it can potentially be used by the processor at runtime. Hence, the entries must always
contain valid values. Furthermore, we want the `rt` crate to be flexible so the end user can customize the
behavior of each exception handler. Finally, the vector table resides in read only memory, or rather in not
easily modified memory, so the user has to register the handler statically, rather than at runtime.
 -->

これまで述べた通り、プロセッサはベクタテーブルがメモリ内の所定の位置にあることを期待しています。そして、各エントリは、
実行時にプロセッサによって使用される可能性があります。したがって、エントリは必ず値を持たなければなりません。
加えて、`rt`クレートにはエンドユーザーが各例外ハンドラの動作をカスタマイズできる柔軟さを持たせたいです。
最後に、ベクタテーブルは読み込み専用メモリ、もしくは、変更が容易でないメモリにあるため、ユーザーは実行時ではなく、
静的にハンドラを登録しなければなりません。

<!-- 
To satisfy all these constraints, we'll assign a *default* value to all the entries of the vector
table in the `rt` crate, but make these values kind of *weak* to let the end user override them
at compile time.
 -->

これら全ての制約を満たすため、`rt`クレートのベクタテーブルの全エントリに*デフォルト*値を割り当てますが、
このデフォルト値は、ユーザーがコンパイル時にオーバーライドできるように*ウィーク*相当のものにします。

<!-- ## Rust side -->

## Rust側

<!-- 
Let's see how all this can be implemented. For simplicity, we'll only work with the first 16 entries
of the vector table; these entries are not device specific so they have the same function on any
kind of Cortex-M microcontroller.
 -->

これを全て実装できる方法を見ていきましょう。簡単化のために、ベクタテーブルの最初の16エントリだけを扱います。
これらのエントリは、デバイス固有のものではなく、全てのCortex-Mマイクロコントローラ上に同じ機能があります。

<!-- 
The first thing we'll do is create an array of vectors (pointers to exception handlers) in the
`rt` crate's code:
 -->

まず最初にやることは、`rt`クレートのコードにベクタ配列（例外ハンドラへのポインタ）を作ることです。

``` console
$ sed -n 56,91p ../rt/src/lib.rs
```

``` rust
{{#include ../ci/exceptions/rt/src/lib.rs:56:91}}
```

<!-- 
Some of the entries in the vector table are *reserved*; the ARM documentation states that they
should be assigned the value `0` so we use a union to do exactly that. The entries that must point
to a handler make use of *external* functions; this is important because it lets the end user
*provide* the actual function definition.
 -->

ベクタテーブル内のいくつかのエントリは*予約済み*です。ARMのドキュメントには、これらのエントリに`0`を割り当てなければならないと書いてあります。
そこで、ユニオンを使って正確に実装します。
エントリは*外部*関数として使えるようにしたハンドラを指している必要があります。
これは、エンドユーザーが実際の関数定義を*提供する*ために重要です。

<!-- 
Next, we define a default exception handler in the Rust code. Exceptions that have not been assigned
a handler by the end user will make use of this default handler.
 -->

次に、Rustコードにデフォルトの例外ハンドラを定義します。
エンドユーザーによってハンドラが割り当てられない例外は、このデフォルトハンドラを使います。

``` console
$ tail -n4 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/exceptions/rt/src/lib.rs:93:97}}
```

<!-- ## Linker script side -->

## リンカスクリプト側

<!-- 
On the linker script side, we place these new exception vectors right after the reset vector.
 -->

リンカスクリプト側では、リセットベクタの直後に新しい例外ベクタを配置します。

``` console
$ sed -n 12,25p ../rt/link.x
```

``` text
{{#include ../ci/exceptions/rt/link.x:12:27}}
```

<!-- 
And we use `PROVIDE` to give a default value to the handlers that we left undefined in `rt` (`NMI`
and the others above):
 -->

`rt`で未定義のハンドラ（`NMI`など）にデフォルト値を与えるため、`PROVIDE`を使います。

``` console
$ tail -n8 ../rt/link.x
```

``` text
PROVIDE(NMI = DefaultExceptionHandler);
PROVIDE(HardFault = DefaultExceptionHandler);
PROVIDE(MemManage = DefaultExceptionHandler);
PROVIDE(BusFault = DefaultExceptionHandler);
PROVIDE(UsageFault = DefaultExceptionHandler);
PROVIDE(SVCall = DefaultExceptionHandler);
PROVIDE(PendSV = DefaultExceptionHandler);
PROVIDE(SysTick = DefaultExceptionHandler);
```

<!-- 
`PROVIDE` only takes effect when the symbol to the left of the equal sign is still undefined after
inspecting all the input object files. This is the scenario where the user didn't implement the
handler for the respective exception.
 -->

`PROVIDE`は、全ての入力オブジェクトファイルを調べた後、=の左辺が未定義のときのみ効果を発揮します。
これは、ユーザーが各例外についてハンドラを実装しなかった場合です。

<!-- ## Testing it -->

## テスト

<!-- 
That's it! The `rt` crate now has support for exception handlers. We can test it out with following
application:
 -->

これで全てです！これで、`rt`クレートは例外ハンドラをサポートします。
次のアプリケーションを使って、テストができます。

<!-- 
> **NOTE**: Turns out it's hard to generate an exception in QEMU. On real
> hardware a read to an invalid memory address (i.e. outside of the Flash and
> RAM regions) would be enough but QEMU happily accepts the operation and
> returns zero. A trap instruction works on both QEMU and hardware but
> unfortunately it's not available on stable so you'll have to temporarily
> switch to nightly to run this and the next example.
 -->

> **注記** QEMU上で例外を生成するのは難しいことがわかりました。実際のハードウェアでは、
> 不正なメモリアドレス（つまりFlashとRAM領域の外側）を読むだけで十分ですが、QEMUは幸運なことにこの操作を受け付け、ゼロを返します。
> トラップ命令はQEMUとハードウェア両方で機能しますが、不運なことにstableのRustでは利用できません。
> そのため、今回と次の例を動かすために、一時的にnightlyに切り替える必要があります。

``` rust
{{#include ../ci/exceptions/app/src/main.rs}}
```

``` console
(gdb) target remote :3333
Remote debugging using :3333
Reset () at ../rt/src/lib.rs:7
7       pub unsafe extern "C" fn Reset() -> ! {

(gdb) b DefaultExceptionHandler
Breakpoint 1 at 0xec: file ../rt/src/lib.rs, line 95.

(gdb) continue
Continuing.

Breakpoint 1, DefaultExceptionHandler ()
    at ../rt/src/lib.rs:95
95          loop {}

(gdb) list
90          Vector { handler: SysTick },
91      ];
92
93      #[no_mangle]
94      pub extern "C" fn DefaultExceptionHandler() {
95          loop {}
96      }
```

<!-- 
And for completeness, here's the disassembly of the optimized version of the program:
 -->

完全を期するため、最適化されたバージョンのプログラムの逆アセンブリを見せます。

``` console
$ cargo objdump --bin app --release -- -d -no-show-raw-insn -print-imm-hex
```

``` text
{{#include ../ci/exceptions/app/app.objdump:1:28}}
```

``` console
$ cargo objdump --bin app --release -- -s -j .vector_table
```

``` text
{{#include ../ci/exceptions/app/app.vector_table.objdump}}
```

<!-- 
The vector table now resembles the results of all the code snippets in this book
  so far. To summarize:
- In the [_Inspecting it_] section of the earlier memory chapter, we learned
  that:
    - The first entry in the vector table contains the initial value of the
      stack pointer.
    - Objdump prints in `little endian` format, so the stack starts at
      `0x2001_0000`.
    - The second entry points to address `0x0000_0045`, the Reset handler.
        - The address of the Reset handler can be seen in the disassembly above,
          being `0x44`.
        - The first bit being set to 1 does not alter the address due to
          alignment requirements. Instead, it causes the function to be executed
          in _thumb mode_.
- Afterwards, a pattern of addresses alternating between `0x7f` and `0x00` is
  visible.
    - Looking at the disassembly above, it is clear that `0x7f` refers to the
      `DefaultExceptionHandler` (`0x7e` executed in thumb mode).
    - Cross referencing the pattern to the vector table that was set up earlier
      in this chapter (see the definition of `pub static EXCEPTIONS`) with [the
      vector table layout for the Cortex-M], it is clear that the address of the
      `DefaultExceptionHandler` is present each time a respective handler entry
      is present in the table.
    - In turn, it is also visible that the layout of the vector table data
      structure in the Rust code is aligned with all the reserved slots in the
      Cortex-M vector table. Hence, all reserved slots are correctly set to a
      value of zero.
 -->

ベクタテーブルは、この本にあるこれまでのコードスニペット全ての結果を象徴しています。まとめると
- メモリレイアウトの章の[_調査_]セクションで、次のことを学びました。
    - ベクタテーブルの1つ目のエントリは、スタックポインタの初期値です。
    - objdumpは、`little endin`フォーマットで出力され、スタックは`0x2001_0000`から始まります。
    - `0x0000_0045`番地を指す2つ目のエントリは、リセットハンドラです。
        - リセットハンドラのアドレスは、上の逆アセンブリで`0x44`であることがわかります。
        - 最初のビットが1に設定されていますが、アライメント要件のため、アドレスは変わりません。代わりに、*thumbモード*で関数が実行されるようになります。
- その後は、`0x7f`と`0x00`が交互に現れるアドレスのパターンが見えます。
    - 上の逆アセンブリを見ると、`0x7f`が`DefaultExceptionHandler`（`0x7e`がthumbモードで実行される）を参照しているのは明らかです。
    - この章の前半で設定したベクタテーブルへのパターン（`pub static EXCEPTIONS`の定義を見て下さい）と[Cortex-Mのベクタテーブルレイアウト]とを相互参照すると、`DefaultExceptionHandler`のアドレスがテーブル内の各ハンドラエントリにあることが明らかです。
    - 次に、Rustコードのベクタテーブルのデータ構造のレイアウトが予約済みスロットも含めて、Cortex-Mベクタテーブルにアライメントされていることも見ることができます。そのため。全ての予約済みスロットは、正しくゼロに設定されています。

[_Inspecting it_]: https://docs.rust-embedded.org/embedonomicon/memory-layout.html#inspecting-it
[the vector table layout for the Cortex-M]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

[_調査_]: https://docs.rust-embedded.org/embedonomicon/memory-layout.html#inspecting-it
[Cortex-Mのベクタテーブルレイアウト]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

<!-- ## Overriding a handler -->

## ハンドラのオーバーライド

<!-- 
To override an exception handler, the user has to provide a function whose symbol name exactly
matches the name we used in `EXCEPTIONS`.
 -->

例外ハンドラをオーバーライドするため、ユーザーは`EXCEPTIONS`で使った名前と完全に一致するシンボルの関数を提供しなければなりません。

``` rust
{{#include ../ci/exceptions/app2/src/main.rs}}
```

<!-- You can test it in QEMU -->

QEMUでテストできます。

``` console
(gdb) target remote :3333
Remote debugging using :3333
Reset () at /home/japaric/rust/embedonomicon/ci/exceptions/rt/src/lib.rs:7
7       pub unsafe extern "C" fn Reset() -> ! {

(gdb) b HardFault
Breakpoint 1 at 0x44: file src/main.rs, line 18.

(gdb) continue
Continuing.

Breakpoint 1, HardFault () at src/main.rs:18
18          loop {}

(gdb) list
13      }
14
15      #[no_mangle]
16      pub extern "C" fn HardFault() -> ! {
17          // ここで何か面白いことをして下さい
18          loop {}
19      }
```

<!-- 
The program now executes the user defined `HardFault` function instead of the
`DefaultExceptionHandler` in the `rt` crate.
 -->

今回は、プログラムは、`rt`クレートの`DefaultExceptionHandler`ではなく、ユーザーが定義した`HardFault`関数を実行します。

<!-- 
Like our first attempt at a `main` interface, this first implementation has the problem of having no
type safety. It's also easy to mistype the name of the exception, but that doesn't produce an error
or warning. Instead the user defined handler is simply ignored. Those problems can be fixed using a
macro like the [`exception!`] macro defined in `cortex-m-rt` v0.5.x or the
[`exception`] attribute in `cortex-m-rt` v0.6.x.
 -->

`main`インタフェースでの最初の試みのように、最初の実装は型安全でないという問題があります。
簡単に、例外の名前を間違ってしまいますが、エラーも警告も発しません。
代わりに、ユーザー定義のハンドラは単に無視されます。
これらの問題は、`cortex-m-rt` v0.5.xの[`exception!`]マクロか、`cortex-m-rt` v0.6.x.の[`exception`]アトリビュートにより解決できます。

[`exception!`]: https://github.com/japaric/cortex-m-rt/blob/v0.5.1/src/lib.rs#L792
[`exception`]: https://github.com/rust-embedded/cortex-m-rt/blob/v0.6.3/macros/src/lib.rs#L254
