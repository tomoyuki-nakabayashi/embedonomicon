//! `'static` bound

#![deny(missing_docs, warnings)]

use core::{
    marker::Unpin,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::atomic::{self, Ordering},
};

use as_slice::{AsMutSlice, AsSlice};
use shared::{Dma1Channel1, USART1_RX, USART1_TX};

impl Serial1 {
    /// 与えられた`buffer`が埋められるまでデータを受信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn read_exact<B>(mut self, mut buffer: Pin<B>) -> Transfer<B>
    where
        // 注記：'static境界を追加しました
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
            buffer,
            serial: self,
        }
    }

    /// 与えられた`buffer`を送信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn write_all<B>(mut self, buffer: Pin<B>) -> Transfer<B>
    where
        // 注記：'static境界を追加しました
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
            buffer,
            serial: self,
        }
    }
}

// UNCHANGED

fn main() {}

/// A singleton that represents serial port #1
pub struct Serial1 {
    dma: Dma1Channel1,
    // ..
}

/// A DMA transfer
pub struct Transfer<B> {
    buffer: Pin<B>,
    serial: Serial1,
}

impl<B> Transfer<B> {
    /// Returns `true` if the DMA transfer has finished
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_progress()
    }

    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(self) -> (Pin<B>, Serial1) {
        while !self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        (self.buffer, self.serial)
    }
}
