# Direct Memory Access (DMA)

<!-- 
This section covers the core requirements for building a memory safe API around
DMA transfers.
 -->

このセクションは、DMA転送周りのメモリ安全なAPI構築における主要な要件について、説明します。

<!-- 
The DMA peripheral is used to perform memory transfers in parallel to the work
of the processor (the execution of the main program). A DMA transfer is more or
less equivalent to spawning a thread (see [`thread::spawn`]) to do a `memcpy`.
We'll use the fork-join model to illustrate the requirements of a memory safe
API.
 -->

DMAペリフェラルは、プロセッサの動作（メインプログラムの実行）と並行してメモリ転送を行うために使用されます。
DMA転送は、`memcpy`を実行するためにスレッドを生成すること（[`thread::spawn`]を参照）とほぼ同等です。
メモリ安全なAPIの要件を説明するために、fork-joinのモデルを使用します。

[`thread::spawn`]: https://doc.rust-lang.org/std/thread/fn.spawn.html

<!-- Consider the following DMA primitives: -->

次のDMAプリミティブを考えます。

``` rust
{{#include ../ci/dma/src/lib.rs:6:57}}
{{#include ../ci/dma/src/lib.rs:59:60}}
```

<!-- 
Assume that the `Dma1Channel1` is statically configured to work with serial port
(AKA UART or USART) #1, `Serial1`, in one-shot mode (i.e. not circular mode).
`Serial1` provides the following *blocking* API:
 -->

`Dma1Channel1`は、`Serial1`というシリアルポート（別名UARTまたはUSART）#1と1ショットモード（つまりサーキュラーモードでない）でやり取りするように、
静的に設定されていると想定して下さい。
`Serial1`は次のような*ブロッキングする*APIを提供します。

``` rust
{{#include ../ci/dma/src/lib.rs:62:72}}
{{#include ../ci/dma/src/lib.rs:74:80}}
{{#include ../ci/dma/src/lib.rs:82:83}}
```

<!-- 
Let's say we want to extend `Serial1` API to (a) asynchronously send out a
buffer and (b) asynchronously fill a buffer.
 -->

例えば、(a)非同期にバッファを送信し、(b)非同期にバッファを埋めるように、`Serial1` APIを拡張したいとしましょう。

<!-- 
We'll start with a memory unsafe API and we'll iterate on it until it's
completely memory safe. On each step we'll show you how the API can be broken to
make you aware of the issues that need to be addressed when dealing with
asynchronous memory operations.
 -->

メモリアンセーフなAPIから出発し、完全にメモリ安全になるまで繰り返し改善していきます。
各ステップで、非同期メモリ操作を扱う際に対処すべき問題を理解するために、
APIがどのように壊れる可能性があるか、を説明します。

<!-- ## A first stab -->

## 最初の挑戦

<!-- 
For starters, let's try to use the [`Write::write_all`] API as a reference. To
keep things simple let's ignore all error handling.
 -->

初心者向けに、[`Write::write_all`]を参考に使ってみましょう。
単純化のため、全てのエラー処理を無視します。

[`Write::write_all`]: https://doc.rust-lang.org/std/io/trait.Write.html#method.write_all

``` rust
{{#include ../ci/dma/examples/one.rs:7:47}}
```

<!-- 
> **NOTE:** `Transfer` could expose a futures or generator based API instead of
> the API shown above. That's an API design question that has little bearing on
> the memory safety of the overall API so we won't delve into it in this text.
 -->

> **注記** `Transfer`は、上述のAPIの代わりに、フューチャーやジェネレータベースのAPIとして公開できるでしょう。
> それは、API設計の問題で、API全体のメモリ安全性にはほとんど関係がありません。
> そのため、このテキストでは、詳しく説明しません。

<!-- 
We can also implement an asynchronous version of [`Read::read_exact`].
 -->

[`Read::read_exact`]の非同期バージョンも実装できます。

[`Read::read_exact`]: https://doc.rust-lang.org/std/io/trait.Read.html#method.read_exact

``` rust
{{#include ../ci/dma/examples/one.rs:49:63}}
```

<!-- Here's how to use the `write_all` API: -->

`write_all` APIの使い方は次のとおりです。

``` rust
{{#include ../ci/dma/examples/one.rs:66:71}}
```

<!-- And here's an example of using the `read_exact` API: -->

そして、`read_exact` APIの使用例です。

``` rust
{{#include ../ci/dma/examples/one.rs:74:86}}
```

## `mem::forget`

<!-- 
[`mem::forget`] is a safe API. If our API is truly safe then we should be able
to use both together without running into undefined behavior. However, that's
not the case; consider the following example:
 -->

[`mem::forget`]は安全なAPIです。もし私達のAPIが本当に安全なら、
未定義動作を起こさずに両方のAPIを同時に使えるはずです。
しかしながら、そうではありません。次の例を考えます。

[`mem::forget`]: https://doc.rust-lang.org/std/mem/fn.forget.html

``` rust
{{#include ../ci/dma/examples/one.rs:91:103}}
{{#include ../ci/dma/examples/one.rs:105:112}}
```

<!-- 
Here we start a DMA transfer, in `foo`, to fill an array allocated on the stack
and then `mem::forget` the returned `Transfer` value. Then we proceed to return
from `foo` and execute the function `bar`.
 -->

ここで、スタック上に確保された配列を埋めるために、`foo`からDMA転送を開始します。
そして、戻り値の`Transfer`を`mem::forget`します。
その後、`foo`から戻り、`bar`関数を実行します。

<!-- 
This series of operations results in undefined behavior. The DMA transfer writes
to stack memory but that memory is released when `foo` returns and then reused
by `bar` to allocate variables like `x` and `y`. At runtime this could result in
variables `x` and `y` changing their value at random times. The DMA transfer
could also overwrite the state (e.g. link register) pushed onto the stack by the
prologue of function `bar`.
 -->

この一連の操作は、未定義動作を引き起こします。DMA転送はスタックメモリに書き込みますが、
そのメモリは`foo`から戻った時に解放され、`bar`で`x`と`y`のような変数を確保するために再利用されます。
実行時、`x`と`y`の値は、ランダムなタイミングで書き換わる可能性があります。
DMA転送は`bar`関数のプロローグによりスタックにプッシュされた状態（例えば、リンクレジスタ）を上書きする可能性もあります。

<!-- 
Note that if we had not use `mem::forget`, but `mem::drop`, it would have been
possible to make `Transfer`'s destructor stop the DMA transfer and then the
program would have been safe. But one can *not* rely on destructors running to
enforce memory safety because `mem::forget` and memory leaks (see RC cycles) are
safe in Rust.
 -->

`mem::forget`を使わずに、`mem::drop`を使うと、`Transfer`のデストラクタはDMA転送を停止し、
プログラムを安全にすることができることに留意して下さい。
しかし、メモリ安全性を強制するためにデストラクタの実行に頼ることは*できません*。
なぜなら、`mem::forget`とメモリリーク（RCサイクルを参照）はRustでは安全だからです。

<!-- 
We can fix this particular problem by changing the lifetime of the buffer from
`'a` to `'static` in both APIs.
 -->

両APIのバッファのライフタイムを`'a`から`'static`に変更することで、この問題を解決できます。

``` rust
{{#include ../ci/dma/examples/two.rs:7:12}}
{{#include ../ci/dma/examples/two.rs:21:27}}
{{#include ../ci/dma/examples/two.rs:35:36}}
```

<!-- 
If we try to replicate the previous problem we note that `mem::forget` no longer
causes problems.
 -->

もし前と同じ問題を再現しようとすると、`mem::forget`はもはや問題になりません。

``` rust
{{#include ../ci/dma/examples/two.rs:40:52}}
{{#include ../ci/dma/examples/two.rs:54:61}}
```

<!-- 
As before, the DMA transfer continues after `mem::forget`-ing the `Transfer`
value. This time that's not an issue because `buf` is statically allocated
(e.g. `static mut` variable) and not on the stack.
 -->

前回同様、`Transfer`の値を`mem::forget`した後も、DMA転送は続いています。
今回は、これは問題になりません。なぜなら`buf`は静的に確保されており（例えば、`static mut`変数）、
スタック上にないからです。

<!-- ## Overlapping use -->

## オーバーラップして使う

<!-- 
Our API doesn't prevent the user from using the `Serial` interface while the DMA
transfer is in progress. This could lead the transfer to fail or data to be
lost.
 -->

私達のAPIは、DMA転送を行っている間、ユーザーが`Serial`インタフェースを使えてしまいます。
これは、DMA転送が失敗するか、データロスを発生させる可能性があります。

<!-- 
There are several ways to prevent overlapping use. One way is to have `Transfer`
take ownership of `Serial1` and return it back when `wait` is called.
 -->

オーバーラップしての利用を防ぐ方法は、いくつかあります。
1つの方法は、`Transfer`が`Serial1`の所有権を取得し、`wait`が呼ばれた時に所有権を返すことです。

``` rust
{{#include ../ci/dma/examples/three.rs:7:32}}
{{#include ../ci/dma/examples/three.rs:40:53}}
{{#include ../ci/dma/examples/three.rs:60:68}}
```
<!-- 
The move semantics statically prevent access to `Serial1` while the transfer is
in progress.
 -->

ムーブセマンティクスは、DMA転送を行っている間、`Serial1`へのアクセスを静的に防ぎます。

``` rust
{{#include ../ci/dma/examples/three.rs:71:81}}
```

<!-- 
There are other ways to prevent overlapping use. For example, a (`Cell`) flag
that indicates whether a DMA transfer is in progress could be added to
`Serial1`. When the flag is set `read`, `write`, `read_exact` and `write_all`
would all return an error (e.g. `Error::InUse`) at runtime. The flag would be
set when `write_all` / `read_exact` is used and cleared in `Transfer.wait`.
 -->

オーバーラップして利用できないようにする方法が他にもいくつかあります。
例えば、`Serial1`にDMA転送中かどうかを示す（`Cell`）フラグを追加できます。
もしフラグがセットされている時は、`read`, `write`, `read_exact`および`write_all`は、
実行時にエラー（例えば、`Error::InUse`）を返します。
このフラグは`write_all` / `read_exact`が使われた時にセットし、`Transfer.wait`でクリアします。

<!-- ## Compiler (mis)optimizations -->

## コンパイラの（誤った）最適化

<!-- 
The compiler is free to re-order and merge non-volatile memory operations to
better optimize a program. With our current API, this freedom can lead to
undefined behavior. Consider the following example:
 -->

コンパイラは、よりプログラムを最適化するため、non-volatileなメモリ操作の順番を入れ替えたり、結合する自由があります。
現在のAPIでは、この自由が未定義動作を引き起こします。
次の例を考えます。

``` rust
{{#include ../ci/dma/examples/three.rs:84:97}}
```

<!-- 
Here the compiler is free to move `buf.reverse()` before `t.wait()`, which would
result in a data race: both the processor and the DMA would end up modifying
`buf` at the same time. Similarly the compiler can move the zeroing operation to
after `read_exact`, which would also result in a data race.
 -->

ここで、コンパイラは、自由に`t.wait()`の前に`buf.reverse()`を移動することができます。
この移動は、プロセッサとDMAが同時に`buf`を修正するデータ競合を起こします。
同様に、コンパイラはゼロクリア操作を`read_exact`の後に移動するかもしれません。
それもデータ競合を起こします。

<!-- 
To prevent these problematic reorderings we can use a [`compiler_fence`]
 -->

これらの問題ある順番の入れ替えを起こさないために、[`compiler_fence`]を使えます。

[`compiler_fence`]: https://doc.rust-lang.org/core/sync/atomic/fn.compiler_fence.html

``` rust
{{#include ../ci/dma/examples/four.rs:9:65}}
```

<!-- 
We use `Ordering::Release` in `read_exact` and `write_all` to prevent all
preceding memory operations from being moved *after* `self.dma.start()`, which
performs a volatile write.
 -->

volatileな書き込みをする`self.dma.start()`の*後ろに*先行するメモリ操作が移動されないように、
`read_exact`と`write_all`では`Ordering::Release`を使います。

<!-- 
Likewise, we use `Ordering::Acquire` in `Transfer.wait` to prevent all
subsequent memory operations from being moved *before* `self.is_done()`, which
performs a volatile read.
 -->

同様に、volatileな読み込みをする`self.is_done()`の*前に*後続のメモリ操作が移動されないように、
`Transfer.wait`では`Ordering::Acquire`を使います。

<!-- 
To better visualize the effect of the fences here's a slightly tweaked version
of the example from the previous section. We have added the fences and their
orderings in the comments.
 -->

フェンスの効果をより理解しやすくするために、前回セクションの例を少し修正したバージョンを示します。
フェンスを追加しており、メモリ操作の順序はコメントで記述しています。

``` rust
{{#include ../ci/dma/examples/four.rs:68:87}}
```

<!-- 
The zeroing operation can *not* be moved *after* `read_exact` due to the
`Release` fence. Similarly, the `reverse` operation can *not* be moved *before*
`wait` due to the `Acquire` fence. The memory operations *between* both fences
*can* be freely reordered across the fences but none of those operations
involves `buf` so such reorderings do *not* result in undefined behavior.
 -->

`Release`フェンスのおかげで、ゼロクリアする操作は、`read_exact`より*後ろ*に動かすことが*できません*。
同様に、`Acquire`フェンスのおかげで、`reverse`操作は`wait`より*前に*動かすことが*できません*。
両フェンスの*間*にあるメモリ操作は、フェンスを超えて自由に順序を入れ替えることが*できます*が、
`buf`に関わるような操作はありません。そのため、順序の入れ替えは、未定義動作を*起こしません*。

<!-- 
Note that `compiler_fence` is a bit stronger than what's required. For example,
the fences will prevent the operations on `x` from being merged even though we
know that `buf` doesn't overlap with `x` (due to Rust aliasing rules). However,
there exist no intrinsic that's more fine grained than `compiler_fence`.
 -->

`compiler_fence`は求められているものより少し強いことに注意して下さい。例えば、
このフェンスは、`buf`と`x`とがオーバーラップしない（Rustのエイリアス規則のため）ことが分かっているにも関わらず、
`x`に対する操作が結合されないようにします。しかしながら、
`compiler_fence`より細かい粒度のintrinsicは存在していません。

<!-- ### Don't we need a memory barrier? -->

### メモリバリアは不要なのですか？

<!-- 
That depends on the target architecture. In the case of Cortex M0 to M4F cores,
[AN321] says:
 -->

ターゲットアーキテクチャによります。Cortex M0とM4Fコアについて、[AN321]は次のように言っています。

[AN321]: https://static.docs.arm.com/dai0321/a/DAI0321A_programming_guide_memory_barriers_for_m_profile.pdf

<!-- 
> 3.2 Typical usages
>
> (..)
>
> The use of DMB is rarely needed in Cortex-M processors because they do not
> reorder memory transactions. However, it is needed if the software is to be
> reused on other ARM processors, especially multi-master systems. For example:
>
> - DMA controller configuration. A barrier is required between a CPU memory
>   access and a DMA operation.
>
> (..)
>
> 4.18 Multi-master systems
>
> (..)
>
> Omitting the DMB or DSB instruction in the examples in Figure 41 on page 47
> and Figure 42 would not cause any error because the Cortex-M processors:
>
> - do not re-order memory transfers
> - do not permit two write transfers to be overlapped.
 -->

> 3.2 一般的な使い方
> 
> (..)
> 
> DMBの使用はCortex-Mプロセッサではほとんど必要ありません。なぜならCortex-Mプロセッサは
> メモリトランザクションの順序を変更しないからです。しかし、ソフトウェアが他のARMプロセッサ、
> 特に複数のマスターがあるシステム、で再利用される場合は必要です。例えば、
> 
> - DMAコントローラ設定。バリアは、CPUのメモリアクセスとDMA操作との間で必要です。
> 
> (..)
> 
> 4.18 複数のマスターがあるシステム
> 
> (..)
> 
> 47ページの図41や図42でDMBやDSB命令を除去すると、何らかのエラーが発生します。なぜなら、Cortex-Mプロセッサは
> 
> - メモリ転送の順序を入れ替えない
> - オーバーラップした2つの書き込み転送を許可しない

<!-- 
Where Figure 41 shows a DMB (memory barrier) instruction being used before
starting a DMA transaction.
 -->

ここで、図41は、DMAトランザクションを開始する前に使用されるDMB（メモリバリア）命令を示しています。

<!-- 
In the case of Cortex-M7 cores you'll need memory barriers (DMB/DSB) if you are
using the data cache (DCache), unless you manually invalidate the buffer used by
the DMA.
 -->

Cortex-M7コアの場合、データキャッシュ（DCache）を使っていれば、
DMAで使用されるバッファを手動で無効化しない限り、メモリバリア（DMB/DSB）が必要になります。

<!-- 
If your target is a multi-core system then it's very likely that you'll need
memory barriers.
 -->

もしターゲットがマルチコアシステムの場合、メモリバリアが必要になる可能性が非常に高いです。

<!-- 
If you do need the memory barrier then you need to use [`atomic::fence`] instead
of `compiler_fence`. That should generate a DMB instruction on Cortex-M devices.
 -->

もしメモリバリアが必要な場合、`compiler_fence`の代わりに[`atomic::fence`]を使わなければなりません。
これは、Cortex-MデバイスではDMB命令を生成するはずです。

[`atomic::fence`]: https://doc.rust-lang.org/core/sync/atomic/fn.fence.html

<!-- ## Generic buffer -->

## ジェネリックバッファ

<!-- 
Our API is more restrictive that it needs to be. For example, the following
program won't be accepted even though it's valid.
 -->

私達のAPIは要件よりも制約が強いです。例えば、
次のプログラムは正しいですが、対応できません

``` rust
{{#include ../ci/dma/examples/five.rs:67:85}}
```

<!-- 
To accept such program we can make the buffer argument generic.
 -->

このようなプログラムに対応するため、バッファの引数をジェネリックにできます。

``` rust
{{#include ../ci/dma/examples/five.rs:9:65}}
```

<!-- 
> **NOTE:** `AsRef<[u8]>` (`AsMut<[u8]>`) could have been used instead of
> `AsSlice<Element = u8>` (`AsMutSlice<Element = u8`).
 -->

> **注記**：`AsSlice<Element = u8>` (`AsMutSlice<Element = u8`)の代わりに、
> `AsRef<[u8]>` (`AsMut<[u8]>`)を使うことができます。

<!-- Now the `reuse` program will be accepted. -->

これで、`reuse`プログラムに対応できます。

<!-- ## Immovable buffers -->

## 固定バッファ

<!-- 
With this modification the API will also accept arrays by value (e.g. `[u8;
16]`). However, using arrays can result in pointer invalidation. Consider the
following program.
 -->

この修正でAPIは値として配列（例えば、`[u8; 16]`）を受け取れるようになります。
しかし、配列を使用すると、ポインタが不正になる可能性があります。
次のプログラムを考えます。

``` rust
{{#include ../ci/dma/examples/five.rs:88:103}}
{{#include ../ci/dma/examples/five.rs:105:112}}
```

<!-- 
The `read_exact` operation will use the address of the `buffer` local to the
`start` function. That local `buffer` will be freed when `start` returns and the
pointer used in `read_exact` will become invalidated. You'll end up with a
situation similar to the [`unsound`](#dealing-with-memforget) example.
 -->

`read_exact`操作は、`start`関数にある`buffer`のアドレスを使います。
このローカル`buffer`は、`start`から戻った時に解放され、`read_exact`で使われているポインタは不正になります。
[`unsound`](#dealing-with-memforget)の例と似たような状況になるでしょう。

<!-- 
To avoid this problem we require that the buffer used with our API retains its
memory location even when it's moved. The [`Pin`] newtype provides such
guarantee. We can update our API to required that all buffers are "pinned"
first.
 -->

この問題を避けるため、APIで使用するバッファに、ムーブされてもメモリの位置を保ち続けることを要求します。
[`Pin`]ニュータイプは、このような保証を提供します。
全てのバッファがあらかじめ「pin」されていることを要求するように、APIを更新します。

[`Pin`]: https://doc.rust-lang.org/nightly/std/pin/index.html

<!-- 
> **NOTE:** To compile all the programs below this point you'll need Rust
> `>=1.33.0`. As of time of writing (2019-01-04) that means using the nightly
> channel.
 -->

> **注記：** 以降のプログラムをコンパイルするためには、Rust `1.33.0以上`が必要です。
> 執筆時点（2019-01-04）では、nightlyチャネルの使用を意味します。

``` rust
{{#include ../ci/dma/examples/six.rs:16:33}}
{{#include ../ci/dma/examples/six.rs:48:59}}
{{#include ../ci/dma/examples/six.rs:74:75}}
```

<!-- 
> **NOTE:** We could have used the [`StableDeref`] trait instead of the `Pin`
> newtype but opted for `Pin` since it's provided in the standard library.
 -->

> **注記：** `Pin`ニュータイプの代わりに[`StableDeref`]トレイトを使うことができますが、
> `Pin`は標準ライブラリで提供されるため、Pinを選びました。

[`StableDeref`]: https://crates.io/crates/stable_deref_trait

<!-- 
With this new API we can use `&'static mut` references, `Box`-ed slices, `Rc`-ed
slices, etc.
 -->

この新しいAPIでは、`&'static mut`参照、`Box`化したスライス、`Rc`化されたスライスなどを使えます。

``` rust
{{#include ../ci/dma/examples/six.rs:78:89}}
{{#include ../ci/dma/examples/six.rs:91:101}}
```

<!-- ## `'static` bound -->

## `'static`境界

<!-- 
Does pinning let us safely use stack allocated arrays? The answer is *no*.
Consider the following example.
 -->

Pinを使うことで、スタックに割り当てられた配列を安全に使えるのでしょうか？
答えは、*ノー*です。次の例を考えます。

``` rust
{{#include ../ci/dma/examples/six.rs:104:123}}
{{#include ../ci/dma/examples/six.rs:125:132}}
```

<!-- 
As seen many times before, the above program runs into undefined behavior due to
stack frame corruption.
 -->

これまで何回も見た通り、スタックフレームの破壊により、上記のプログラムは未定義動作に陥ります。

<!-- 
The API is unsound for buffers of type `Pin<&'a mut [u8]>` where `'a` is *not*
`'static`. To prevent the problem we have to add a `'static` bound in some
places.
 -->

このAPIは、`Pin<&'a mut [u8]>`（ここで`'a`は`static`では*ありません*）の型を持つバッファに対して、
安全ではありません。
この問題を解決するため、どこかに`'static`境界を追加しなければなりません。

``` rust
{{#include ../ci/dma/examples/seven.rs:15:25}}
{{#include ../ci/dma/examples/seven.rs:40:51}}
{{#include ../ci/dma/examples/seven.rs:66:67}}
```

<!-- Now the problematic program will be rejected. -->

これで問題のプログラムは拒絶されます。

<!-- ## Destructors -->

## デストラクタ

<!-- 
Now that the API accepts `Box`-es and other types that have destructors we need
to decide what to do when `Transfer` is early-dropped.
 -->

これでAPIは`Box`やデストラクタを持つ型を受け入れることができます。
`Transfer`が早めにドロップされたときに何をすべきか決める必要があります。

<!-- 
Normally, `Transfer` values are consumed using the `wait` method but it's also
possible to, implicitly or explicitly, `drop` the value before the transfer is
over. For example, dropping a `Transfer<Box<[u8]>>` value will cause the buffer
to be deallocated. This can result in undefined behavior if the transfer is
still in progress as the DMA would end up writing to deallocated memory.
 -->

通常、`Transfer`の値は、`wait`メソッドを使って消費されます。しかし、転送が完了する前に、
暗黙的もしくは明示的に、値を`drop`することも可能です。
例えば、`Transfer<Box<[u8]>>`の値をドロップすると、バッファは解放されます。
これは、まだ転送中であれば、DMAが解放済みのメモリに書き込むため、未定義動作を引き起こします。

<!-- 
In such scenario one option is to make `Transfer.drop` stop the DMA transfer.
The other option is to make `Transfer.drop` wait for the transfer to finish.
We'll pick the former option as it's cheaper.
 -->

このような状況では、`Transfer.drop`でDMA転送を止めることが1つの選択肢です。
他の選択肢は、`Transfer.drop`が転送完了を待つことです。
より簡単なので、前者を選びます。

``` rust
{{#include ../ci/dma/examples/eight.rs:18:72}}
{{#include ../ci/dma/examples/eight.rs:82:99}}
{{#include ../ci/dma/examples/eight.rs:109:117}}
```

<!-- 
Now the DMA transfer will be stopped before the buffer is deallocated.
 -->

これで、バッファが解放される前にDMA転送が中断されます。

``` rust
{{#include ../ci/dma/examples/eight.rs:120:134}}
```

<!-- ## Summary -->

## まとめ

<!-- 
To sum it up, we need to consider all the following points to achieve  memory
safe DMA transfers:
 -->

まとめると、メモリ安全なDMA転送を行うために、これら全てを考えなければなりません。

<!-- 
- Use immovable buffers plus indirection: `Pin<B>`. Alternatively, you can use
  the `StableDeref` trait.
 -->

- `Pin<B>`という固定バッファと間接参照を使います。あるいは、`StableDeref`トレイトを使用できます。

<!-- 
- The ownership of the buffer must be passed to the DMA : `B: 'static`.
 -->

- `B: 'static`というバッファの所有権をDMAに渡す必要があります。

<!-- 
- Do *not* rely on destructors running for memory safety. Consider what happens
  if `mem::forget` is used with your API.
 -->

- メモリ安全性をデストラクタの実行に頼っては*いけません*。
  APIと`mem::forget`が一緒に使われるとどうなるか、考えて下さい。

<!-- 
- *Do* add a custom destructor that stops the DMA transfer, or waits for it to
  finish. Consider what happens if `mem::drop` is used with your API.
 -->

- DMA転送を中断するカスタムデストラクタを*追加*、もしくは、転送完了まで待機、*するようにして下さい*。
  APIと`mem::drop`が一緒に使われるとどうなるか、考えて下さい。
  

---

<!-- 
This text leaves out up several details required to build a production grade
DMA abstraction, like configuring the DMA channels (e.g. streams, circular vs
one-shot mode, etc.), alignment of buffers, error handling, how to make the
abstraction device-agnostic, etc. All those aspects are left as an exercise for
the reader / community (`:P`).
 -->

このテキストでは製品レベルのDMA抽象を構築するために要求される詳細を省略しています。
例えば、DMAチャネルの設定（ストリーム、サーキュラー vs ワンショットモードなど）、バッファのアライメント、
エラー処置、デバイスに依存しない抽象の作り方などについてです。
これらの点は、読者 / コミュニティの演習とします (`:P`)。
