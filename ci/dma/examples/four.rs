//! Compiler (mis)optimizations

#![deny(missing_docs, warnings)]

use core::sync::atomic::{self, Ordering};

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

impl Serial1 {
    /// 与えられた`buffer`が埋められるまでデータを受信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn read_exact(mut self, buffer: &'static mut [u8]) -> Transfer<&'static mut [u8]> {
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        // 注記：追加しました
        atomic::compiler_fence(Ordering::Release);

        // 注記：これはvolatileな*書き込み*です
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }

    /// 与えられた`buffer`を送信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn write_all(mut self, buffer: &'static [u8]) -> Transfer<&'static [u8]> {
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        // 注記：追加しました
        atomic::compiler_fence(Ordering::Release);

        // 注記：これはvolatileな*書き込み*です
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }
}

impl<B> Transfer<B> {
    /// 転送が完了するまでブロックし、バッファを返します。
    pub fn wait(self) -> (B, Serial1) {
        // 注記： これはvolatileな*読み込み*です
        while !self.is_done() {}

        // 注記：追加しました
        atomic::compiler_fence(Ordering::Acquire);

        (self.buffer, self.serial)
    }

    // ..
}

#[allow(dead_code, unused_variables)]
fn reorder(serial: Serial1, buf: &'static mut [u8], x: &mut u32) {
    // バッファをゼロクリアします（特別な理由はありません）
    buf.iter_mut().for_each(|byte| *byte = 0);

    *x += 1;

    let t = serial.read_exact(buf); // compiler_fence(Ordering::Release) ▲

    // 注記：プロセッサはフェンスの間、`buf`にアクセスできません
    // ... 何か別のことをやります ..
    *x += 2;

    let (buf, serial) = t.wait(); // compiler_fence(Ordering::Acquire) ▼

    *x += 3;

    buf.reverse();

    // .. `buf`で何かやります ..
}

// UNCHANGED

fn main() {}

/// A DMA transfer
pub struct Transfer<B> {
    buffer: B,
    serial: Serial1,
}

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
