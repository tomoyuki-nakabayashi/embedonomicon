//! Overlapping use

#![deny(missing_docs, warnings)]

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// 1回のDMA転送です
pub struct Transfer<B> {
    buffer: B,
    // 注記：追加しました
    serial: Serial1,
}

impl<B> Transfer<B> {
    /// 転送が完了するまでブロックし、バッファを返します。
    /// 注記：戻り値が変わっています
    pub fn wait(self) -> (B, Serial1) {
        // 転送が完了するまでビジーウェイトします
        while !self.is_done() {}

        (self.buffer, self.serial)
    }

    // ..
}

impl Serial1 {
    /// 与えられた`buffer`が埋められるまでデータを受信します
    /// 
    /// DMA転送中であることを意味する値を返します
    // 注記 今回は、`self`を値として受け取ります
    pub fn read_exact(mut self, buffer: &'static mut [u8]) -> Transfer<&'static mut [u8]> {
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        // .. 以前と同じです ..

        Transfer {
            buffer,
            // 注記：追加しました
            serial: self,
        }
    }

    /// 与えられた`buffer`を送信します
    /// 
    /// DMA転送中であることを意味する値を返します
    // 注記 今回は、`self`を値として受け取ります
    pub fn write_all(mut self, buffer: &'static [u8]) -> Transfer<&'static [u8]> {
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        // .. 以前と同じです ..

        Transfer {
            buffer,
            // 注記：追加しました
            serial: self,
        }
    }
}

#[allow(dead_code, unused_variables)]
fn read(serial: Serial1, buf: &'static mut [u8; 16]) {
    let t = serial.read_exact(buf);

    // let byte = serial.read(); //~ ERROR: `serial` has been moved

    // .. 何かやります ..

    let (serial, buf) = t.wait();

    // .. さらに何かやります ..
}

#[allow(dead_code, unused_variables)]
fn reorder(serial: Serial1, buf: &'static mut [u8]) {
    // バッファをゼロクリアします（特別な理由はありません）
    buf.iter_mut().for_each(|byte| *byte = 0);

    let t = serial.read_exact(buf);

    // ... 何か別のことをやります ..

    let (buf, serial) = t.wait();

    buf.reverse();

    // .. `buf`で何かやります ..
}

// 変更ありません

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
