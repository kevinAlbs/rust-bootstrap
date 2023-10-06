https://bugzmanov.github.io/nes_ebook/chapter_3_2.html
https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L246
Book suggest setting `status` to 0. Code sets status BREAK and BREAK2 flags to 1 on reset.
Code adds BREAK2 flags twice:
https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L481
https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L488
I expect these are unnecessary. The BREAK2 flag is set to 1 reset and appears not to ever be set to 0.

https://www.nesdev.org/obelisk-6502-guide/reference.html#PHP
Does not suggest setting bit 4 on value pushed onto the stack.
However, bit 4 is set here: https://github.com/bugzmanov/nes_ebook/blob/c4f905346b27e3ab17277e9651d191ff310f480b/code/ch3.3/src/cpu.rs#L484
And described here: https://www.nesdev.org/wiki/Status_flags#The_B_flag
> B is 0 when pushed by interrupts (/IRQ and /NMI) and 1 when pushed by instructions (BRK and PHP).

https://www.nesdev.org/obelisk-6502-guide/reference.html#LSR
Under `LSR`:
> N	Negative Flag	Set if bit 7 of the result is set
I expect Negative Flag is never set.

I expect STACK_RESET could be 0xFF. https://github.com/bugzmanov/nes_ebook/blob/master/code/ch3.3/src/cpu.rs#L346 shows STACK_RESET as 0xFD.

https://www.nesdev.org/obelisk-6502-guide/reference.html#ASL
ASL `Zero Flag` may be incorrectly documented. Says `Set if A = 0`. I expect this is meant to be `Set if result = 0`, since ASL may apply to accumulator or memory contents.

https://bugzmanov.github.io/nes_ebook/chapter_3_2.html
Does not refer to the "Indexed" addressing mode described on https://www.nesdev.org/obelisk-6502-guide/addressing.html#IND

https://bugzmanov.github.io/nes_ebook/chapter_3_2.html
`(hi << 8) | (lo as u16)` can be `(hi << 8) | lo` since `lo` is already `u16`.

`let ptr: u8 = (base as u8).wrapping_add(self.register_x);`
`base as u8` is not necessary. `base` is already `u8`

`fn lda` needs one more indent.

Has typo: "Lastly, all we do is hard-coding"

--
https://bugzmanov.github.io/nes_ebook/chapter_3_2.html
Has typo: "We've discusses"

--
https://bugzmanov.github.io/nes_ebook/chapter_3.html

Says:
> Access to [0x8000 … 0x10000] is mapped to Program ROM (PRG ROM) space on a cartridge.

The image shows address range 0x8000 to 0xFFFF for PRG ROM.