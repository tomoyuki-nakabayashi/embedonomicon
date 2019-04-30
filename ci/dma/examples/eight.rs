//! Destructors

#![deny(missing_docs, warnings)]

use core::{
    hint,
    marker::Unpin,
    mem,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
    sync::atomic::{self, Ordering},
};

use as_slice::{AsMutSlice, AsSlice};
use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// 1回のDMA転送です
pub struct Transfer<B> {
    // 注記：常に`Some`ヴァリアントです
    inner: Option<Inner<B>>,
}

// 注記：以前は、`Transfer<B>という名前でした
struct Inner<B> {
    buffer: Pin<B>,
    serial: Serial1,
}

impl<B> Transfer<B> {
    /// 転送が完了するまでブロックし、バッファを返します。
    pub fn wait(mut self) -> (Pin<B>, Serial1) {
        while !self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { hint::unreachable_unchecked() });
        (inner.buffer, inner.serial)
    }
}

impl<B> Drop for Transfer<B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            // 注記：これはvolatileな書き込みです
            inner.serial.dma.stop();

            // Acquireフェンスを有効化するため、ここで読み込みが必要です
            // `dma.stop`がRMW操作をするのであれば、これは*不要*です
            unsafe {
                ptr::read_volatile(&0);
            }

            // `Transfer.wait`と同じ理由でフェンスが必要です。
            atomic::compiler_fence(Ordering::Acquire);
        }
    }
}

impl Serial1 {
    /// 与えられた`buffer`が埋められるまでデータを受信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn read_exact<B>(mut self, mut buffer: Pin<B>) -> Transfer<B>
    where
        B: DerefMut + 'static,
        B::Target: AsMutSlice<Element = u8> + Unpin,
    {
        // .. 以前と同じです ..
        let slice = buffer.as_mut_slice();
        let (ptr, len) = (slice.as_mut_ptr(), slice.len());

        self.dma.set_source_address(USART1_RX, false);
        self.dma.set_destination_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);
        self.dma.start();

        Transfer {
            inner: Some(Inner {
                buffer,
                serial: self,
            }),
        }
    }

    /// 与えられた`buffer`を送信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn write_all<B>(mut self, buffer: Pin<B>) -> Transfer<B>
    where
        B: Deref + 'static,
        B::Target: AsSlice<Element = u8>,
    {
        // .. 以前と同じです ..
        let slice = buffer.as_slice();
        let (ptr, len) = (slice.as_ptr(), slice.len());

        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);
        self.dma.start();

        Transfer {
            inner: Some(Inner {
                buffer,
                serial: self,
            }),
        }
    }
}

#[allow(dead_code, unused_mut, unused_variables)]
fn reuse(serial: Serial1) {
    let buf = Pin::new(Box::new([0; 16]));

    let t = serial.read_exact(buf); // compiler_fence(Ordering::Release) ▲

    // ..

    // これはDMA転送を中断し、メモリを解放します
    mem::drop(t); // compiler_fence(Ordering::Acquire) ▼

    // これは、前のメモリ割り当てを再利用する可能性が高いです
    let mut buf = Box::new([0; 16]);

    // `buf`で何かやります
}

// UNCHANGED

fn main() {}

/// A singleton that represents serial port #1
pub struct Serial1 {
    dma: Dma1Channel1,
    // ..
}

impl<B> Transfer<B> {
    /// Returns `true` if the DMA transfer has finished
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_progress()
    }
}
