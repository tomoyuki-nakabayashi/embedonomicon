#![allow(unused_variables)]

pub const USART1_TX: usize = 0x4000_0000;
pub const USART1_RX: usize = 0x4000_0004;

/// 1つのDMAチャネル（ここではチャネル1）を表すシングルトンです
/// 
/// このシングルトンは、DMAチャネル１のレジスタへの排他アクセスを持ちます
pub struct Dma1Channel1 {
    // ..
}

impl Dma1Channel1 {
    /// データは`address`に書かれます
    /// 
    /// `inc`は、各転送の後にアドレスをインクリメントするかどうか、を意味します
    /// 
    /// 注記 この関数はvolatileな書き込みを行います
    pub fn set_destination_address(&mut self, address: usize, inc: bool) {
        // ..
    }

    /// データは`address`から読まれます
    /// 
    /// `inc`は、各転送の後にアドレスをインクリメントするかどうか、を意味します
    /// 
    /// 注記 この関数はvolatileな書き込みを行います
    pub fn set_source_address(&mut self, address: usize, inc: bool) {
        // ..
    }

    /// 転送するバイト数です
    /// 
    /// 注記 この関数はvolatileな書き込みを行います
    pub fn set_transfer_length(&mut self, len: usize) {
        // ..
    }

    /// DMA転送を開始します
    /// 
    /// 注記 この関数はvolatileな書き込みを行います
    pub fn start(&mut self) {
        // ..
    }

    /// DMA転送を停止します
    /// 
    /// 注記 この関数はvolatileな書き込みを行います
    pub fn stop(&mut self) {
        // ..
    }

    /// 転送中なら`true`を返します
    /// 
    ///  注記 この関数はvolatileな読み込みを行います
    pub fn in_progress() -> bool {
        // ..
        false
    }
}

/// シリアルポート#1を表すシングルトンです
pub struct Serial1 {
    // ..
}

impl Serial1 {
    /// 1バイト読み込みます
    /// 
    /// 注記：読み込めるバイトがないとブロックします
    pub fn read(&mut self) -> Result<u8, Error> {
        // ..
        Ok(0)
    }

    /// １バイト送信します
    /// 
    /// 注記：出力FIFOバッファに空きがなければブロックします
    pub fn write(&mut self, byte: u8) -> Result<(), Error> {
        // ..
        Ok(())
    }
}

pub enum Error {}
