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