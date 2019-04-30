//! A first stab

#![deny(missing_docs, warnings)]

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// シリアルポート#1を表すシングルトンです
pub struct Serial1 {
    // 注記：DMAチャネルシングルトンを追加することで、このstructを拡張します
    dma: Dma1Channel1,
    // ..
}

impl Serial1 {
    /// 与えられた`buffer`を送信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn write_all<'a>(mut self, buffer: &'a [u8]) -> Transfer<&'a [u8]> {
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        Transfer { buffer }
    }
}

/// 1回のDMA転送です
pub struct Transfer<B> {
    buffer: B,
}

impl<B> Transfer<B> {
    /// DMA転送が完了すると`true`を返します
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_progress()
    }

    /// 転送が完了するまでブロックし、バッファを返します。
    pub fn wait(self) -> B {
        // 転送が完了するまでビジーウェイトします
        while !self.is_done() {}

        self.buffer
    }
}

impl Serial1 {
    /// 与えられた`buffer`が埋められるまでデータを受信します
    /// 
    /// DMA転送中であることを意味する値を返します
    pub fn read_exact<'a>(&mut self, buffer: &'a mut [u8]) -> Transfer<&'a mut [u8]> {
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        Transfer { buffer }
    }
}

#[allow(dead_code)]
fn write(serial: Serial1) {
    // 転送を開始して、忘れます
    serial.write_all(b"Hello, world!\n");

    // 他のことをやります
}

#[allow(dead_code)]
fn read(mut serial: Serial1) {
    let mut buf = [0; 16];
    let t = serial.read_exact(&mut buf);

    // 他のことをやります

    t.wait();

    match buf.split(|b| *b == b'\n').next() {
        Some(b"some-command") => { /* 何かやります */ }
        _ => { /* 何か他のことをやります */ }
    }
}

use core::mem;

#[allow(dead_code)]
fn unsound(mut serial: Serial1) {
    start(&mut serial);
    bar();
}

#[inline(never)]
fn start(serial: &mut Serial1) {
    let mut buf = [0; 16];

    // DMA転送を開始し、戻り値の`Transfer`をforgetします
    mem::forget(serial.read_exact(&mut buf));
}

#[allow(unused_variables, unused_mut)]
#[inline(never)]
fn bar() {
    // スタック変数です
    let mut x = 0;
    let mut y = 0;

    // `x`と`y`を使います
}

fn main() {}
