/* LM3S6965マイクロコントローラのメモリレイアウト */
/* 1K = 1 KiBi = 1024バイト */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

/* エントリポイントはリセットハンドラです */
ENTRY(Reset);

EXTERN(RESET_VECTOR);
EXTERN(EXCEPTIONS); /* <- 追加 */

SECTIONS
{
  .vector_table ORIGIN(FLASH) :
  {
    /* 1つ目のエントリ。スタックポインタの初期値 */
    LONG(ORIGIN(RAM) + LENGTH(RAM));

    /* 2つ目のエントリ。リセットベクタ */
    KEEP(*(.vector_table.reset_vector));

    /* 続く14エントリは例外ベクタです */
    KEEP(*(.vector_table.exceptions)); /* <- 追加 */
  } > FLASH

  .text :
  {
    *(.text .text.*);
  } > FLASH

  /* CHANGED! */
  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  .bss :
  {
    _sbss = .;
    *(.bss .bss.*);
    _ebss = .;
  } > RAM

  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
  {
    _sdata = .;
    *(.data .data.*);
    _edata = .;
  } > RAM

  _sidata = LOADADDR(.data);

  /DISCARD/ :
  {
    *(.ARM.exidx.*);
  }
}

PROVIDE(NMI = DefaultExceptionHandler);
PROVIDE(HardFault = DefaultExceptionHandler);
PROVIDE(MemManage = DefaultExceptionHandler);
PROVIDE(BusFault = DefaultExceptionHandler);
PROVIDE(UsageFault = DefaultExceptionHandler);
PROVIDE(SVCall = DefaultExceptionHandler);
PROVIDE(PendSV = DefaultExceptionHandler);
PROVIDE(SysTick = DefaultExceptionHandler);
