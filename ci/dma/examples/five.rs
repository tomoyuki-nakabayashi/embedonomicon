//! Generic buffer

#![deny(missing_docs, warnings)]

use core::sync::atomic::{self, Ordering};

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

// as-slice = "0.1.0"
use as_slice::{AsMutSlice, AsSlice};

impl Serial1 {
    /// 与えられた`buffer`が埋められるまでデータを受信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn read_exact<B>(mut self, mut buffer: B) -> Transfer<B>
    where
        B: AsMutSlice<Element = u8>,
    {
        // 注記：追加しました
        let slice = buffer.as_mut_slice();
        let (ptr, len) = (slice.as_mut_ptr(), slice.len());

        self.dma.set_source_address(USART1_RX, false);

        // 注記：微妙に変更しました
        self.dma.set_destination_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }

    /// 与えられた`buffer`を送信します
    /// 
    /// DMA転送中であることを意味する値を返します
    fn write_all<B>(mut self, buffer: B) -> Transfer<B>
    where
        B: AsSlice<Element = u8>,
    {
        // 注記：追加しました
        let slice = buffer.as_slice();
        let (ptr, len) = (slice.as_ptr(), slice.len());

        self.dma.set_destination_address(USART1_TX, false);

        // 注記：微妙に変更しました
        self.dma.set_source_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }
}

#[allow(dead_code, unused_variables)]
fn reuse(serial: Serial1, msg: &'static mut [u8]) {
    // メッセージを送信します
    let t1 = serial.write_all(msg);

    // ..

    let (msg, serial) = t1.wait(); // `msg`は現在`&'static [u8]`です

    msg.reverse();

    // 今度は、逆順に送ります
    let t2 = serial.write_all(msg);

    // ..

    let (buf, serial) = t2.wait();

    // ..
}

#[allow(dead_code, unused_variables)]
fn invalidate(serial: Serial1) {
    let t = start(serial);

    bar();

    let (buf, serial) = t.wait();
}

#[inline(never)]
fn start(serial: Serial1) -> Transfer<[u8; 16]> {
    // このフレームで確保された配列です
    let buffer = [0; 16];

    serial.read_exact(buffer)
}

#[allow(unused_mut, unused_variables)]
#[inline(never)]
fn bar() {
    // スタック変数です
    let mut x = 0;
    let mut y = 0;

    // `x`と`y`を使います
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

    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(self) -> (B, Serial1) {
        while !self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        (self.buffer, self.serial)
    }
}
