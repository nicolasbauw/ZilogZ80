# Fine-tuning log

Improvement points addressed after the initial XF/YF/MEMPTR work (see
`REPRISE-drapeaux-non-documentes.md` for that part). This file is a running
log of correctness passes, not a design document — new entries go at the
bottom.

## Crash on loading Cauldron.cdt (overflow panic in adc_16)

The P/V flag calculation in `adc_16` computed `n + c` on `u16`: panics in
debug builds as soon as the operand was `0xFFFF` with the carry set. `sbc_16`
had the same defect in a silent form — an `as i16` cast that truncated
`n + c` to zero, producing a wrong P/V instead of a panic.

Both now use the canonical XOR formulas, the ones the 8-bit `adc`/`sbc`
already used:
```
ADC: V = (h ^ r) & (n ^ r) & 0x8000
SBC: V = (h ^ n) & (h ^ r) & 0x8000
```
Regression test: `adc_sbc_16_with_ffff_operand_and_carry`.

## Same bug class found during the audit: backward relative jumps

`JR e`, `JR cc,e` and `DJNZ` computed `pc + 2 - displacement` without
protection on `u16` (6 sites). A backward jump from the very start of memory
panicked in debug builds instead of wrapping around, as the real hardware
does. Rewritten via sign extension through `relative_target`.
Regression test: `backward_relative_jumps_wrap_below_address_zero`.

Note: commit `c8e0814` ("Wrap 16 bit address arithmetic instead of
overflowing") already targeted this bug class, but had left these six sites
untouched.

Rest of the audit, no action needed:
- 8-bit adc/sbc/add/sub: already correct (XOR formulas, widened to `u16` for
  the carry).
- 16-bit half-carries and carries: all sums stay within the type or are
  widened to `u32`.
- `n << 1` shifts: in Rust, only a shift wider than the type panics, not the
  loss of the high bit. Nothing to do.
- `signed_to_abs(n) = !n + 1` would panic for `n = 0`. No caller left since
  the indexed-addressing rework; kept in place as it's part of the public
  API.

## Warnings and lints in test.rs

An unused `CPU` in `dasm_cb`, 113 `assert_eq!(x, true)` turned into
`assert!(x)`, two unnecessary mutable borrows, a no-op `0x00 | z`. No more
warnings or clippy lints on the test side. The three remaining lints are in
the library and are decoder style choices.

## Documented flags of block I/O instructions

`INI`/`IND`/`OUTI`/`OUTD` set neither S, nor H, nor P/V, and always forced N
to 1 although it actually mirrors bit 7 of the transferred byte. See
`REPRISE-drapeaux-non-documentes.md` for the rules.
Covered by `block_io_documented_flags`.

## MEMPTR/WZ

The register is modeled and updated by every relevant instruction family;
`BIT b,(HL)` and `BIT b,(IX+d)` now take their two undocumented flags from
it. The indexed form also failed to set S and P/V. Rule details in
`REPRISE-drapeaux-non-documentes.md`.
Covered by `memptr_update_rules` and `bit_undocumented_flags_come_from_memptr`.

Prerequisite: the 50 `(IX+d)`/`(IY+d)` accesses each repeated a five-line
if/else distinguishing positive and negative displacement, where a sign
extension is enough. Factored into `ix_d`/`iy_d`, which also became the
MEMPTR hook point. 400 fewer lines.

## Passing zexall

`bin/zexall.com` passes all 67 tests, no errors. This is the test suite that
checks every bit of F, undocumented flags included — unlike zexdoc, which
masks them and therefore validated nothing of the XF/YF/MEMPTR work.

Verified with a negative control: reverting `BIT b,(HL)` to its old
approximation (reading the tested value instead of MEMPTR's high byte) makes
zexall flag the corresponding line as an error.

Reservation: zexall doesn't cover I/O or interrupts. The MEMPTR rules for
`IN`/`OUT`, block I/O, and interrupt acknowledgment, as well as the
documented flags of block I/O instructions, still rest only on the tests
written here, hand-derived from the published rules.

Open item, flagged earlier: the repeated block I/O forms (`INIR`/`OTIR`...)
don't follow the "MEMPTR = PC + 1" rule that `LDIR`/`CPIR` do — a choice
matching reference cores, but one zexall doesn't check.
