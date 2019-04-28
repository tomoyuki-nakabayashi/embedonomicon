<!-- # Global singletons -->

# グローバルシングルトン

<!-- 
In this section we'll cover how to implement a global, shared singleton. The
embedded Rust book covered local, owned singletons which are pretty much unique
to Rust. Global singletons are essentially the singleton pattern you see in C
and C++; they are not specific to embedded development but since they involve
symbols they seemed a good fit for the embedonomicon.
 -->

このセクションでは、グローバルに共有されるシングルトンの実装方法を説明します。
The embedded Rust bookは、Rust特有のローカルで所有されるシングルトンを説明しました。
グローバルシングルトンは、本質的にCやC++で見かけるシングルトンパターンです。
これは、組込み開発固有のものではありませんが、シンボルに関係するため、embedonomiconに相応しい内容のように思えます。

<!-- 内部的なTODOなので、そのまま残してあります。 -->

> **TODO**(resources team) link "the embedded Rust book" to the singletons
> section when it's up

<!-- 
To illustrate this section we'll extend the logger we developed in the last
section to support global logging. The result will be very similar to the
`#[global_allocator]` feature covered in the embedded Rust book.
 -->

グローバルシングルトンを説明するために、このセクションでは、前のセクションで開発したロガーを、
グローバルにログ出力できるように拡張します。
結果は、the embedded Rust bookで説明した`#[global_allocator]`フィーチャと非常に似たものになります。

<!-- 内部的なTODOなので、そのまま残してあります。 -->

> **TODO**(resources team) link `#[global_allocator]` to the collections chapter
> of the book when it's in a more stable location.

<!-- Here's the summary of what we want to: -->

やりたいことを、下記にまとめます。

<!-- 
In the last section we created a `log!` macro to log messages through a specific
logger, a value that implements the `Log` trait. The syntax of the `log!` macro
is `log!(logger, "String")`. We want to extend the macro such that
`log!("String")` also works. Using the `logger`-less version should log the
message through a global logger; this is how `std::println!` works. We'll also
need a mechanism to declare what the global logger is; this is the part that's
similar to `#[global_allocator]`.
 -->

前のセクションでは、`Log`トレイトを実装している特定のロガーを通してログメッセージを出力するために、`log!`マクロを作りました。
`log!`マクロのシンタックスは、`log!(logger, "String")`です。
このマクロを、`log!("String")`でも動くように拡張します。
`logger`なしのバージョンを使うと、グローバルロガーを通してメッセージをログ出力しなければなりません。
これは、`std::println!`が動作する方法と同じです。
また、何がグローバルロガーか、を宣言するための機構が必要です。
これは、`#[global_allocator]`と似ている部分です。

<!-- 
It could be that the global logger is declared in the top crate and it could
also be that the type of the global logger is defined in the top crate. In this
scenario the dependencies can *not* know the exact type of the global logger. To
support this scenario we'll need some indirection.
 -->

グローバルロガーが最上位クレートで宣言される可能性があり、
グローバルロガーの型もまた最上位クレートで定義される可能性があります。
この場合、依存関係から正確なグローバルロガーの型を知ることは*できません*。
この場合をサポートするために、いくらか間接的な方法が必要になります。

<!-- 
Instead of hardcoding the type of the global logger in the `log` crate we'll
declare only the *interface* of the global logger in that crate. That is we'll
add a new trait, `GlobalLog`, to the `log` crate. The `log!` macro will also
have to make use of that trait.
 -->

`log`クレートにグローバルロガーの型をハードコーディングする代わりに、logクレート内で、
グローバルロガーの*インタフェース*だけを宣言します。
そのインタフェースは、`log`クレートに新しく追加する`GlobalLog`というトレイトです。
`log!`マクロもそのトレイトを使うようにします。

``` console
$ cat ../log/src/lib.rs
```

``` rust
{{#include ../ci/singleton/log/src/lib.rs}}
```

<!-- There's quite a bit to unpack here. -->

解説することがたくさんあります。

<!-- Let's start with the trait. -->

トレイトから始めましょう。

``` rust
{{#include ../ci/singleton/log/src/lib.rs:4:6}}
```

<!-- 
Both `GlobalLog` and `Log` have a `log` method. The difference is that
`GlobalLog.log` takes a shared reference to the receiver (`&self`). This is
necessary because the global logger will be a `static` variable. More on that
later.
 -->

`GlobalLog`と`Log`とは、`log`メソッドを持っています。違いは、`GlobalLog.log`がレシーバの共有参照(`&self`)を取ることです。
グローバルロガーは`static`変数なので、これが必要です。後ほど、詳しく見ます。

<!-- 
The other difference is that `GlobalLog.log` doesn't return a `Result`. This
means that it can *not* report errors to the caller. This is not a strict
requirement for traits used to implement global singletons. Error handling in
global singletons is fine but then all users of the global version of the `log!`
macro have to agree on the error type. Here we are simplifying the interface a
bit by having the `GlobalLog` implementer deal with the errors.
 -->

もう1つの違う点は、`GlobalLog.log`は`Result`を返さないことです。
これは、呼び出し側にエラーを報告*できない*ことを意味します。
これはグローバルシングルトンを実装するトレイトを使うための必要条件ではありません。
グローバルシングルトンでエラー処理をすることは良いことですが、グローバルバージョンの`log!`マクロの全てのユーザーが、
エラー型に同意する必要があります。
ここでは、`GlobalLog`実装者がエラーを処理するようにして、インタフェースを少し簡略化します。

<!-- 
Yet another difference is that `GlobalLog` requires that the implementer is
`Sync`, that is that it can be shared between threads. This is a requirement for
values placed in `static` variables; their types must implement the `Sync`
trait.
 -->

さらに別の違いは、`GlobalLog`が実装者に、スレッド間で共有できるようにするための`Sync`を要求する点です。
これは、`static`変数内の値への要求です。それらの値の型は、`Sync`を実装しなければなりません。

<!-- 
At this point it may not be entirely clear why the interface has to look this
way. The other parts of the crate will make this clearer so keep reading.
 -->

現時点では、インタフェースがこのようになっていなければならない理由は、完全には明らかではないかもしれません。
クレートの他の部分を見ることで、より明らかになっていきますので、読み進めて下さい。

<!-- Next up is the `log!` macro: -->

次は`log!`マクロです。

``` rust
{{#include ../ci/singleton/log/src/lib.rs:17:29}}
```

<!-- 
When called without a specific `$logger` the macros uses an `extern` `static`
variable called `LOGGER` to log the message. This variable *is* the global
logger that's defined somewhere else; that's why we use the `extern` block. We
saw this pattern in the [main interface] chapter.
 -->

特定の`$logger`なしでマクロを呼び出すと、マクロはメッセージをログ出力するために`LOGGER`と呼ばれる`extern` `static`変数を使います。
この変数*は*どこかで定義されたグローバルロガーです。そのため、`extern`ブロックを使っています。
このパターンは[メインインタフェース]の章で見ました。

<!-- [main interface]: /main.html -->

[メインインタフェース]: /main.html

<!-- 
We need to declare a type for `LOGGER` or the code won't type check. We don't
know the concrete type of `LOGGER` at this point but we know, or rather require,
that it implements the `GlobalLog` trait so we can use a trait object here.
 -->

`LOGGER`の型を宣言する必要があります。そうでなければ、コードは型チェックを行いません。
`LOGGER`の具体的な型はここではわかりませんが、
その型が`GlobalLog`トレイトを実装していることを知っています（むしろ必要としています）。
そこで、トレイトオブジェクトを使うことができます。

<!-- 
The rest of the macro expansion looks very similar to the expansion of the local
version of the `log!` macro so I won't explain it here as it's explained in the
[previous] chapter.
 -->

残りのマクロ拡張は、`log!`マクロのローカルバージョンの拡張ととてもよく似ています。
そのため、[前の]章で説明したことは、ここでは説明しません。

<!-- [previous]: /logging.html -->

[前の]: /logging.html

<!-- 
Now that we know that `LOGGER` has to be a trait object it's clearer why we
omitted the associated `Error` type in `GlobalLog`. If we had not omitted then
we would have need to pick a type for `Error` in the type signature of `LOGGER`.
This is what I earlier meant by "all users of `log!` would need to agree on the
error type".
 -->

ここで、`LOGGER`がトレイトオブジェクトでなければならないことを知っているので、
`GlobalLog`で関連型の`Error`を除去する理由はより明白です。もし除去しなければ、
`LOGGER`の型シグネチャの中で`Error`の型を1つ選ばなければなりません。
これが先程、「`log!`マクロの全てのユーザーが、エラー型に同意する必要があります。」と書いた意味です。

<!-- 
Now the final piece: the `global_logger!` macro. It could have been a proc macro
attribute but it's easier to write a `macro_rules!` macro.
 -->

そして、最後のピースの`global_logger!`マクロです。
これは、手続きマクロアトリビュートにもできますが、`macro_rules!`でマクロを書くほうが簡単です。

``` rust
{{#include ../ci/singleton/log/src/lib.rs:41:47}}
```

<!-- 
This macro creates the `LOGGER` variable that `log!` uses. Because we need a
stable ABI interface we use the `no_mangle` attribute. This way the symbol name
of `LOGGER` will be "LOGGER" which is what the `log!` macro expects.
 -->

このマクロは、`log!`が使用する`LOGGER`変数を作ります。安定したABIインタフェースが必要なので、
`no_mangle`アトリビュートを使用します。
この方法により、`LOGGER`のシンボル名は`log!`マクロが期待する「LOGGER」になります。

<!-- 
The other important bit is that the type of this static variable must exactly
match the type used in the expansion of the `log!` macro. If they don't match
Bad Stuff will happen due to ABI mismatch.
 -->

他の重要な点は、このstatic変数の型は、`log!`マクロの展開で使用される型と正確に一致しなければなりません。
もし一致しない場合、ABIの不一致により、良くないことが起こるでしょう。

<!-- 
Let's write an example that uses this new global logger functionality.
 -->

新しいグローバルロガーの機能を使う例を書いてみましょう。

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/singleton/app/src/main.rs}}
```

<!-- 内部的なTODOなので、そのまま残してあります。 -->

> **TODO**(resources team) use `cortex_m::Mutex` instead of a `static mut`
> variable when `const fn` is stabilized.

<!-- We had to add `cortex-m` to the dependencies. -->

依存関係に`cortex-m`を追加する必要があります。

``` console
$ tail -n5 Cargo.toml
```

``` text
{{#include ../ci/singleton/app/Cargo.toml:11:15}}
```

<!-- 
This is a port of one of the examples written in the [previous] section. The
output is the same as what we got back there.
 -->

これは、[前の]セクションで書いた例を移植したものです。
出力は、以前のものと同じです。

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/singleton/app/dev.out}}
```

``` console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/singleton/app/dev.objdump}}
```

---

<!-- 
Some readers may be concerned about this implementation of global singletons not
being zero cost because it uses trait objects which involve dynamic dispatch,
that is method calls are performed through a vtable lookup.
 -->

このグローバルシングルトンの実装がゼロコストでないことが気になる読者も居るかと思います。
なぜなら、トレイトオブジェクトを使用しており、vtableを参照してメソッド呼び出しを行う動的ディスパッチになるためです。

<!-- 
However, it appears that LLVM is smart enough to eliminate the dynamic dispatch
when compiling with optimizations / LTO. This can be confirmed by searching for
`LOGGER` in the symbol table.
 -->

しかし、LLVMは十分に賢く、この動的ディスパッチをコンパイラの最適化 / LTOで消去してくれます。
このことは、シンボルテーブル内の`LOGGER`を探すことで確認できます。

``` console
$ cargo objdump --bin app --release -- -t | grep LOGGER
```

``` text
{{#include ../ci/singleton/app/release.objdump}}
```

<!-- 
If the `static` is missing that means that there is no vtable and that LLVM was
capable of transforming all the `LOGGER.log` calls into `Logger.log` calls.
 -->

もし`static`が見つからない場合、vtableがないことと、
LLVMが`LOGGER.log`の呼び出しを`Logger.log`の呼び出しに変換できたことを意味します。
