//! GFX10 machine code: instruction boundaries, encodings and opcodes.
//!
//! Instructions are found by following control flow from offset 0, not by a
//! linear sweep: compilers place data (constant tables) in the code region
//! after the end of the program. [`Instruction::fields`] decodes the operand
//! fields and [`Instruction::text`] prints them in AMD's SP3 syntax.
//!
//! Encodings by the top bits of the first dword:
//!
//! ```text
//! 0xxxxxxx  VOP2; 0111111 VOP1; 0111110 VOPC
//! 10        SOP2; 1011 SOPK; 101111101 SOP1; 101111110 SOPC; 101111111 SOPP
//! 110010 VINTRP  110011 VOP3P  110101 VOP3  110110 DS  110111 FLAT/GLOBAL/SCRATCH
//! 111000 MUBUF   111010 MTBUF  111100 MIMG  111101 SMEM  111110 EXP
//! ```

mod fields;
pub(crate) mod opcodes;
mod sp3;

pub use fields::{Cache, Dpp, Fields, NULL, Operand, Sdwa, Valu};

use std::collections::{BTreeMap, BTreeSet};

use crate::ShaderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Sop2,
    Sopk,
    Sop1,
    Sopc,
    Sopp,
    Smem,
    Vop2,
    Vop1,
    Vopc,
    Vop3,
    Vop3p,
    Vintrp,
    Ds,
    Mubuf,
    Mtbuf,
    Mimg,
    Exp,
    Flat,
    Global,
    Scratch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// Byte offset in the program.
    pub offset: usize,
    pub encoding: Encoding,
    pub opcode: u16,
    /// Every dword of the instruction, including literals and extra dwords.
    pub words: Vec<u32>,
    /// [`name`] of `encoding` and `opcode`, resolved once at decode time.
    /// The translator asks about every instruction eight times; the table is
    /// a binary search.
    name: Option<&'static str>,
}

impl Instruction {
    /// AMD's name for the opcode, if it is in our table.
    pub fn name(&self) -> Option<&'static str> {
        self.name
    }
}

/// Decodes every instruction reachable from offset 0, sorted by offset.
pub fn decode(code: &[u8]) -> Result<Vec<Instruction>, ShaderError> {
    Ok(decode_with_flows(code)?.0)
}

/// The same walk with its final control-flow decisions. IR must use these
/// rather than resolve a far jump again from a different set of predecessors.
pub(crate) fn decode_with_flows(code: &[u8]) -> Result<(Vec<Instruction>, Vec<Flow>), ShaderError> {
    if !code.len().is_multiple_of(4) {
        return Err(isa(code.len(), "code size is not a multiple of 4"));
    }

    // A later path can enter a chain already followed as a far jump. Keep
    // discovered entries across retries: making a jump indirect can remove
    // the very path that revealed the entry, but cannot make its setup safe.
    let mut entries = BTreeSet::new();
    loop {
        let mut found: BTreeMap<usize, (Instruction, Flow)> = BTreeMap::new();
        let mut errors = BTreeMap::new();
        let mut chains = Vec::new();
        let mut pending = vec![0usize];
        while let Some(mut offset) = pending.pop() {
            while !found.contains_key(&offset) && !errors.contains_key(&offset) {
                let inst = match instruction(code, offset) {
                    Ok(inst) => inst,
                    Err(error) => {
                        errors.insert(offset, error);
                        break;
                    }
                };
                let next = offset + 4 * inst.words.len();
                let evaluated = flow(&inst, next, code.len()).and_then(|flow| {
                    if flow != Flow::Stop || inst.name() != Some("s_setpc_b64") {
                        return Ok(flow);
                    }
                    let jump = far_jump(
                        |at| found.range(..at).next_back().map(|(_, (i, _))| i),
                        &inst,
                        code.len(),
                        &entries,
                    );
                    match jump {
                        Some(jump) => {
                            chains.push((jump.start, offset));
                            jump.target.map(Flow::Jump)
                        }
                        None => Ok(Flow::Stop),
                    }
                });
                let flow = match evaluated {
                    Ok(flow) => flow,
                    Err(error) => {
                        errors.insert(offset, error);
                        Flow::Stop
                    }
                };
                found.insert(offset, (inst, flow));
                match flow {
                    Flow::Next => offset = next,
                    Flow::Stop => break,
                    Flow::Jump(target) => {
                        entries.insert(target);
                        offset = target;
                    }
                    Flow::Branch(target) => {
                        entries.insert(target);
                        pending.push(target);
                        offset = next;
                    }
                }
            }
        }

        if chains
            .iter()
            .any(|&(start, end)| entries.range(start + 1..=end).next().is_some())
        {
            // Every retry has at least one newly discovered entry that
            // blocks a previously accepted chain, so this converges.
            continue;
        }
        // An error on a speculative target must not hide another path
        // which invalidates that jump. Report errors only after the retry.
        if let Some(error) = errors.into_values().next() {
            return Err(error);
        }
        let (list, flows): (Vec<Instruction>, Vec<Flow>) = found.into_values().unzip();
        if let Some(w) = list
            .windows(2)
            .find(|w| w[0].offset + 4 * w[0].words.len() > w[1].offset)
        {
            return Err(isa(w[1].offset, "a branch lands inside an instruction"));
        }
        return Ok((list, flows));
    }
}

/// Where execution goes after an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flow {
    Next,
    /// End of the program, or an indirect jump that is not a far jump.
    Stop,
    Jump(usize),
    /// Conditional: to the target or the next instruction.
    Branch(usize),
}

fn flow(inst: &Instruction, next: usize, len: usize) -> Result<Flow, ShaderError> {
    let target = || {
        let delta = 4 * i64::from(inst.words[0] as u16 as i16);
        usize::try_from(next as i64 + delta)
            .ok()
            .filter(|&t| t < len)
            .ok_or(isa(inst.offset, "branch target outside the code"))
    };
    Ok(match (inst.encoding, inst.name().unwrap_or("")) {
        (
            Encoding::Sopp,
            "s_endpgm" | "s_endpgm_saved" | "s_endpgm_ordered_ps_done" | "s_code_end",
        ) => Flow::Stop,
        (Encoding::Sopp, "s_branch") => Flow::Jump(target()?),
        (Encoding::Sopp, name) if name.starts_with("s_cbranch_") => Flow::Branch(target()?),
        (Encoding::Sopk, "s_call_b64") => Flow::Branch(target()?),
        // Indirect jump; `decode` resolves the compiler's far-jump form.
        (Encoding::Sop1, "s_setpc_b64") => Flow::Stop,
        _ => Flow::Next,
    })
}

struct FarJump {
    start: usize,
    target: Result<usize, ShaderError>,
}

/// Resolves the far jump compilers emit beyond the reach of a branch offset:
/// `s_getpc_b64 s[n:n+1]`, `s_add_u32|s_sub_u32 sn, sn, literal`, a carry
/// into s(n+1), `s_setpc_b64 s[n:n+1]`, looking back over the straight-line
/// run just decoded. Only NOPs and waits may separate those operations:
/// another instruction may overwrite the pair or its carry. Any other
/// `s_setpc_b64` (e.g. a handover to the next merged stage) gives `None`.
/// A branch may enter at getpc, but must not bypass any of the setup.
fn far_jump<'a>(
    before: impl Fn(usize) -> Option<&'a Instruction>,
    setpc: &Instruction,
    len: usize,
    entries: &BTreeSet<usize>,
) -> Option<FarJump> {
    let n = setpc.words[0] & 0xFF;
    let mut carry = None;
    let mut delta = None;
    let mut at = setpc.offset;
    for _ in 0..4 {
        let Some(prev) = before(at) else {
            break;
        };
        if prev.offset + 4 * prev.words.len() != at {
            break;
        }
        let w = prev.words[0];
        let (sdst, ssrc0, ssrc1) = ((w >> 16) & 0x7F, w & 0xFF, (w >> 8) & 0xFF);
        match (prev.encoding, prev.name()) {
            (Encoding::Sop2, Some(op @ ("s_addc_u32" | "s_subb_u32")))
                if sdst == n + 1 && ssrc0 == n + 1 && carry.is_none() && delta.is_none() =>
            {
                let high = match ssrc1 {
                    128..=192 => ssrc1 - 128,
                    193..=208 => (192i32 - ssrc1 as i32) as u32,
                    0xFF => prev.words[1],
                    _ => return None,
                };
                carry = Some((op, high));
            }
            (Encoding::Sop2, Some(op @ ("s_add_u32" | "s_sub_u32")))
                if sdst == n && ssrc0 == n && ssrc1 == 0xFF && delta.is_none() =>
            {
                let (high_op, high) = carry?;
                if !matches!(
                    (op, high_op),
                    ("s_add_u32", "s_addc_u32") | ("s_sub_u32", "s_subb_u32")
                ) {
                    return None;
                }
                // Both words form the signed offset. In particular, a low
                // word with bit 31 set is not negative when the high is 0.
                // i128 also permits negating i64::MIN for subtraction.
                let literal =
                    i128::from(((u64::from(high) << 32) | u64::from(prev.words[1])) as i64);
                delta = Some(if op == "s_add_u32" { literal } else { -literal });
            }
            (Encoding::Sop1, Some("s_getpc_b64")) if sdst == n => {
                let Some(delta) = delta else { break };
                if entries
                    .range(prev.offset + 1..=setpc.offset)
                    .next()
                    .is_some()
                {
                    return None;
                }
                // s_getpc_b64 yields the address of the instruction after it.
                let target = usize::try_from(prev.offset as i128 + 4 + delta)
                    .ok()
                    .filter(|&t| t < len)
                    .ok_or(isa(setpc.offset, "far jump target outside the code"));
                return Some(FarJump {
                    start: prev.offset,
                    target,
                });
            }
            (Encoding::Sopp, Some("s_nop" | "s_waitcnt" | "s_waitcnt_depctr")) => {}
            _ => return None,
        }
        at = prev.offset;
    }
    None
}

fn instruction(code: &[u8], offset: usize) -> Result<Instruction, ShaderError> {
    let word = |i: usize| {
        code.get(offset + 4 * i..offset + 4 * i + 4)
            .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
            .ok_or(isa(offset, "instruction runs past the end of the code"))
    };
    let w = word(0)?;
    let (encoding, opcode, len) = if w >> 31 == 0 {
        let src0 = w & 0x1FF;
        // Literal constant, SDWA, DPP16, DPP8, DPP8 with fetch-inactive.
        let extra = usize::from(matches!(src0, 0xFF | 0xF9 | 0xFA | 0xE9 | 0xEA));
        match w >> 25 {
            0x3F => (Encoding::Vop1, (w >> 9) & 0xFF, 1 + extra),
            0x3E => (Encoding::Vopc, (w >> 17) & 0xFF, 1 + extra),
            op => {
                let mk_ak = name(Encoding::Vop2, op as u16).is_some_and(|n| {
                    matches!(
                        n,
                        "v_madmk_f32"
                            | "v_madak_f32"
                            | "v_fmamk_f32"
                            | "v_fmaak_f32"
                            | "v_madmk_f16"
                            | "v_madak_f16"
                            | "v_fmamk_f16"
                            | "v_fmaak_f16"
                    )
                });
                (Encoding::Vop2, op, 1 + extra.max(usize::from(mk_ak)))
            }
        }
    } else if w >> 30 == 0b10 {
        let literal = |srcs: &[u32]| 1 + usize::from(srcs.contains(&0xFF));
        match w >> 23 {
            0x17D => (Encoding::Sop1, (w >> 8) & 0xFF, literal(&[w & 0xFF])),
            0x17E => (
                Encoding::Sopc,
                (w >> 16) & 0x7F,
                literal(&[w & 0xFF, (w >> 8) & 0xFF]),
            ),
            0x17F => (Encoding::Sopp, (w >> 16) & 0x7F, 1),
            _ if w >> 28 == 0b1011 => {
                let op = (w >> 23) & 0x1F;
                let imm32 = name(Encoding::Sopk, op as u16) == Some("s_setreg_imm32_b32");
                (Encoding::Sopk, op, 1 + usize::from(imm32))
            }
            _ => (
                Encoding::Sop2,
                (w >> 23) & 0x7F,
                literal(&[w & 0xFF, (w >> 8) & 0xFF]),
            ),
        }
    } else {
        let vop3 =
            |w1: u32| 2 + usize::from([w1, w1 >> 9, w1 >> 18].iter().any(|s| s & 0x1FF == 0xFF));
        match w >> 26 {
            0x32 => (Encoding::Vintrp, (w >> 16) & 0x3, 1),
            0x33 => (Encoding::Vop3p, (w >> 16) & 0x7F, vop3(word(1)?)),
            0x35 => (Encoding::Vop3, (w >> 16) & 0x3FF, vop3(word(1)?)),
            0x36 => (Encoding::Ds, (w >> 18) & 0xFF, 2),
            0x37 => {
                let encoding = match (w >> 14) & 3 {
                    0 => Encoding::Flat,
                    1 => Encoding::Scratch,
                    2 => Encoding::Global,
                    _ => return Err(isa(offset, "reserved FLAT segment")),
                };
                (encoding, (w >> 18) & 0x7F, 2)
            }
            0x38 => (Encoding::Mubuf, (w >> 18) & 0x7F | (w >> 25 & 1) << 7, 2),
            0x3A => (
                Encoding::Mtbuf,
                (w >> 16) & 0x7 | (word(1)? >> 21 & 1) << 3,
                2,
            ),
            // Bits 2:1 count extra dwords of non-sequential address registers.
            0x3C => (
                Encoding::Mimg,
                (w >> 18) & 0x7F | (w & 1) << 7,
                2 + (w as usize >> 1 & 3),
            ),
            0x3D => (Encoding::Smem, (w >> 18) & 0xFF, 2),
            0x3E => (Encoding::Exp, 0, 2),
            _ => return Err(isa(offset, "unknown encoding")),
        }
    };
    let opcode = opcode as u16;
    Ok(Instruction {
        offset,
        encoding,
        opcode,
        words: (0..len).map(word).collect::<Result<_, _>>()?,
        name: name(encoding, opcode),
    })
}

fn name(encoding: Encoding, opcode: u16) -> Option<&'static str> {
    use opcodes as t;
    let (table, op) = match encoding {
        Encoding::Sop2 => (t::SOP2, opcode),
        Encoding::Sopk => (t::SOPK, opcode),
        Encoding::Sop1 => (t::SOP1, opcode),
        Encoding::Sopc => (t::SOPC, opcode),
        Encoding::Sopp => (t::SOPP, opcode),
        Encoding::Smem => (t::SMEM, opcode),
        Encoding::Vop2 => (t::VOP2, opcode),
        Encoding::Vop1 => (t::VOP1, opcode),
        Encoding::Vopc => (t::VOPC, opcode),
        // VOP3 also encodes the VOPC, VOP2 and VOP1 opcodes at fixed bases.
        Encoding::Vop3 => match opcode {
            0x000..0x100 => (t::VOPC, opcode),
            0x100..0x140 => (t::VOP2, opcode - 0x100),
            0x180..0x200 => (t::VOP1, opcode - 0x180),
            _ => (t::VOP3, opcode),
        },
        Encoding::Vop3p => (t::VOP3P, opcode),
        Encoding::Vintrp => (t::VINTRP, opcode),
        Encoding::Ds => (t::DS, opcode),
        Encoding::Mubuf => (t::MUBUF, opcode),
        Encoding::Mtbuf => (t::MTBUF, opcode),
        Encoding::Mimg => (t::MIMG, opcode),
        Encoding::Exp => return Some("exp"),
        Encoding::Flat => (t::FLAT, opcode),
        Encoding::Global => (t::GLOBAL, opcode),
        Encoding::Scratch => (t::SCRATCH, opcode),
    };
    table
        .binary_search_by_key(&op, |&(o, _)| o)
        .ok()
        .map(|i| table[i].1)
}

fn isa(offset: usize, reason: &'static str) -> ShaderError {
    ShaderError::Isa { offset, reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S_ENDPGM: u32 = 0xBF81_0000;
    /// s_branch / s_cbranch_scc0 with a dword offset relative to the next instruction.
    const fn sopp(op: u32, simm16: i16) -> u32 {
        0xBF80_0000 | op << 16 | simm16 as u16 as u32
    }

    fn bytes(words: &[u32]) -> Vec<u8> {
        words.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    fn summary(code: &[u32]) -> Vec<(usize, usize, &'static str)> {
        decode(&bytes(code))
            .unwrap()
            .iter()
            .map(|i| (i.offset, i.words.len(), i.name().unwrap_or("?")))
            .collect()
    }

    #[test]
    fn flat_offsets_ignore_bit_eleven() {
        let offset = |word0: u32| {
            let code = bytes(&[word0, 0x0000_0001, S_ENDPGM]);
            match decode(&code).unwrap()[0].fields(false) {
                Fields::Flat { offset, .. } => offset,
                fields => panic!("{fields:?}"),
            }
        };
        // flat_load_dword v0, v[1:2]: the FLAT offset is 11-bit and
        // unsigned, so bit 11 is ignored (0x800 -> 0, 0xfff -> 0x7ff).
        assert_eq!(offset(0xDC30_0800), 0);
        assert_eq!(offset(0xDC30_0FFF), 0x7FF);
    }

    #[test]
    fn tables_are_sorted() {
        for t in [
            opcodes::SOP2,
            opcodes::SOPK,
            opcodes::SOP1,
            opcodes::SOPC,
            opcodes::SOPP,
            opcodes::SMEM,
            opcodes::VOP2,
            opcodes::VOP1,
            opcodes::VOPC,
            opcodes::VOP3,
            opcodes::VOP3P,
            opcodes::VINTRP,
            opcodes::DS,
            opcodes::MUBUF,
            opcodes::MTBUF,
            opcodes::MIMG,
            opcodes::FLAT,
            opcodes::GLOBAL,
            opcodes::SCRATCH,
        ] {
            assert!(t.windows(2).all(|w| w[0].0 < w[1].0));
        }
    }

    #[test]
    fn lengths_and_names() {
        let code = [
            0xBE80_03FF,
            0x3F80_0000, // s_mov_b32 s0, literal
            0x0600_0000, // v_add_f32 v0, s0, v0
            0xD54B_0000,
            0x03FE_0000,
            0x4000_0000, // v_fma_f32 with a literal src2
            0xF080_0F0A,
            0,
            0, // image_sample (2D) with one extra address dword
            0xF800_080F,
            0, // exp
            S_ENDPGM,
        ];
        assert_eq!(
            summary(&code),
            [
                (0, 2, "s_mov_b32"),
                (8, 1, "v_add_f32"),
                (12, 3, "v_fma_f32"),
                (24, 3, "image_sample"),
                (36, 2, "exp"),
                (44, 1, "s_endpgm"),
            ]
        );
    }

    #[test]
    fn follows_branches_and_skips_data() {
        let code = [
            sopp(0x04, 1), // s_cbranch_scc0 -> 8
            sopp(0x02, 1), // s_branch -> 12
            S_ENDPGM,
            S_ENDPGM,
            S_ENDPGM, // data follows the program end
            0xFFFF_FFFF,
        ];
        let found: Vec<_> = summary(&code).iter().map(|i| i.0).collect();
        assert_eq!(found, [0, 4, 8, 12]);
    }

    #[test]
    fn follows_far_jumps() {
        let code = [
            0xBE88_1F00, // s_getpc_b64 s[8:9]
            0x8008_FF08,
            20,          // s_add_u32 s8, s8, 20  -> 4 + 20 = 24
            0x8209_8009, // s_addc_u32 s9, s9, 0
            0xBE80_2008, // s_setpc_b64 s[8:9]
            0xFFFF_FFFF, // not code
            S_ENDPGM,
        ];
        let found: Vec<_> = summary(&code).iter().map(|i| (i.0, i.2)).collect();
        assert_eq!(
            found,
            [
                (0, "s_getpc_b64"),
                (4, "s_add_u32"),
                (12, "s_addc_u32"),
                (16, "s_setpc_b64"),
                (24, "s_endpgm"),
            ]
        );
        // Through other registers (a handover to another stage): the path ends.
        assert_eq!(summary(&[0xBE80_2018, S_ENDPGM]).len(), 1);
    }

    #[test]
    fn follows_far_jumps_backward() {
        // The compiler's backward far jump adds a negative offset: the low
        // dword as a 32-bit literal, the high dword as -1 with the carry.
        let code = [
            sopp(0x02, 1), // s_branch -> 8, past the jump target
            S_ENDPGM,      // the far jump's target: only it reaches here
            0xBE88_1F00,   // s_getpc_b64 s[8:9]
            0x8008_FF08,
            0xFFFF_FFF8, // s_add_u32 s8, s8, -8 -> 12 - 8 = 4
            0x8209_C109, // s_addc_u32 s9, s9, -1
            0xBE80_2008, // s_setpc_b64 s[8:9]
        ];
        let found: Vec<_> = summary(&code).iter().map(|i| (i.0, i.2)).collect();
        assert_eq!(
            found,
            [
                (0, "s_branch"),
                (4, "s_endpgm"),
                (8, "s_getpc_b64"),
                (12, "s_add_u32"),
                (20, "s_addc_u32"),
                (24, "s_setpc_b64"),
            ]
        );
    }

    #[test]
    fn far_jumps_require_the_matching_high_word_operation() {
        for high in [
            0xBF80_0000, // s_nop: no high-word operation
            0xBE88_0380, // s_mov_b32 s8, 0: overwrites the low word
            0xBE89_0380, // s_mov_b32 s9, 0: overwrites the high word
            0x8289_8009, // s_subb_u32 s9, s9, 0: wrong arithmetic
            0x820A_8009, // s_addc_u32 s10, s9, 0: wrong destination
            0x8209_8007, // s_addc_u32 s9, s7, 0: wrong source
            0x8209_0A09, // s_addc_u32 s9, s9, s10: unknown high immediate
        ] {
            let code = [
                0xBE88_1F00, // s_getpc_b64 s[8:9]
                0x8008_FF08,
                20, // s_add_u32 s8, s8, 20
                high,
                0xBE80_2008, // s_setpc_b64 s[8:9]
                0xFFFF_FFFF, // data, not reached by a recognized far jump
                S_ENDPGM,
            ];
            let found = summary(&code);
            assert_eq!(found.last().unwrap().0, 16, "high word {high:#x}");
            assert_eq!(found.last().unwrap().2, "s_setpc_b64");
            let ir = crate::ir::build(&bytes(&code), false, 1).unwrap();
            assert_eq!(ir.blocks[0].exit, crate::ir::Exit::Indirect);
        }
    }

    #[test]
    fn far_jumps_do_not_skip_register_or_carry_clobbers() {
        for (at, clobber) in [
            (1, 0xBE89_0380), // overwrite the high word before the add
            (3, 0x800A_800A), // s_add_u32 s10, s10, 0 overwrites SCC
            (4, 0xBE88_0380), // overwrite the low word after the carry
            (4, 0xBE89_0380), // overwrite the high word after the carry
        ] {
            let mut code = vec![
                0xBE88_1F00, // s_getpc_b64 s[8:9]
                0x8008_FF08,
                24,          // s_add_u32 s8, s8, 24 -> stale target 28
                0x8209_8009, // s_addc_u32 s9, s9, 0
                0xBE80_2008, // s_setpc_b64 s[8:9]
                0xFFFF_FFFF,
                S_ENDPGM,
            ];
            code.insert(at, clobber);
            let found = summary(&code);
            assert_eq!(found.last().unwrap().0, 20, "clobber {clobber:#x} at {at}");
            assert_eq!(found.last().unwrap().2, "s_setpc_b64");
            let ir = crate::ir::build(&bytes(&code), false, 1).unwrap();
            assert_eq!(ir.blocks[0].exit, crate::ir::Exit::Indirect);
        }
    }

    #[test]
    fn far_jumps_use_both_halves_of_the_offset() {
        for high in [0x8209_8109, 0x8209_C109] {
            // A high word of 1 or -1 makes this a roughly +/-4 GiB offset,
            // not the in-range +20 suggested by the low word alone.
            let code = [
                0xBE88_1F00,
                0x8008_FF08,
                20,
                high,
                0xBE80_2008,
                0xFFFF_FFFF,
                S_ENDPGM,
            ];
            assert!(matches!(
                decode(&bytes(&code)),
                Err(ShaderError::Isa {
                    reason: "far jump target outside the code",
                    ..
                })
            ));
        }
        // A low word with its sign bit set is positive when its high word
        // is zero. It must not be mistaken for the -8 backward-jump form.
        let code = [
            sopp(0x02, 1),
            S_ENDPGM,
            0xBE88_1F00,
            0x8008_FF08,
            0xFFFF_FFF8,
            0x8209_8009,
            0xBE80_2008,
        ];
        assert!(matches!(
            decode(&bytes(&code)),
            Err(ShaderError::Isa {
                reason: "far jump target outside the code",
                ..
            })
        ));
        // Subtracting the most negative signed 64-bit offset cannot fit
        // this program either, but must not overflow the host arithmetic.
        let code = [
            0xBE88_1F00,
            0x8088_FF08,
            0,
            0x8289_FF09,
            0x8000_0000,
            0xBE80_2008,
        ];
        assert!(matches!(
            decode(&bytes(&code)),
            Err(ShaderError::Isa {
                reason: "far jump target outside the code",
                ..
            })
        ));
    }

    #[test]
    fn far_jump_arithmetic_keeps_forward_and_backward_add_and_subtract() {
        for (low_op, low, high, target) in [
            (0x8008_FF08, 20, 0x8209_8009, 32),          // add +20
            (0x8008_FF08, 0xFFFF_FFF8, 0x8209_C109, 4),  // add -8
            (0x8088_FF08, 0xFFFF_FFEC, 0x8289_C109, 32), // subtract -20
            (0x8088_FF08, 8, 0x8289_8009, 4),            // subtract +8
        ] {
            let code = [
                sopp(0x02, 1),
                S_ENDPGM,
                0xBE88_1F00,
                low_op,
                low,
                high,
                0xBE80_2008,
                0xFFFF_FFFF,
                S_ENDPGM,
            ];
            let ends: Vec<_> = summary(&code)
                .into_iter()
                .filter(|i| i.2 == "s_endpgm")
                .map(|i| i.0)
                .collect();
            assert_eq!(ends, [target], "low {low:#x}, high {high:#x}");
        }
    }

    #[test]
    fn far_jumps_allow_literal_high_words_and_scheduling() {
        let forward = [
            0xBE88_1F00,
            0x8008_FF08,
            24,
            0x8209_FF09,
            0,
            0xBE80_2008,
            0xFFFF_FFFF,
            S_ENDPGM,
        ];
        assert_eq!(summary(&forward).last().unwrap().0, 28);
        let backward = [
            sopp(0x02, 1),
            S_ENDPGM,
            0xBE88_1F00,
            0x8008_FF08,
            0xFFFF_FFF8,
            0x8209_FF09,
            0xFFFF_FFFF,
            0xBE80_2008,
        ];
        assert!(summary(&backward).contains(&(4, 1, "s_endpgm")));

        for wait in [0xBF80_0000, 0xBF8C_0000, 0xBFA3_FFFF] {
            for at in [1, 3, 4] {
                let mut code = vec![
                    0xBE88_1F00,
                    0x8008_FF08,
                    24,
                    0x8209_8009,
                    0xBE80_2008,
                    0xFFFF_FFFF,
                    S_ENDPGM,
                ];
                code.insert(at, wait);
                assert_eq!(
                    summary(&code).last().unwrap().0,
                    28,
                    "wait {wait:#x} at {at}"
                );
            }
        }
    }

    fn assert_indirect(code: &[u32], setpc: usize, absent: usize) {
        let code = bytes(code);
        let decoded = decode(&code).unwrap();
        assert!(decoded.iter().any(|i| i.offset == setpc));
        assert!(decoded.iter().all(|i| i.offset != absent));
        for wave64 in [false, true] {
            let ir = crate::ir::build(&code, wave64, 1).unwrap();
            let block = ir
                .blocks
                .iter()
                .find(|b| b.insts.last().is_some_and(|i| i.machine.offset == setpc))
                .unwrap();
            assert_eq!(block.exit, crate::ir::Exit::Indirect);
        }
    }

    #[test]
    fn far_jump_setup_cannot_be_entered_after_getpc() {
        for target in [8, 16, 20] {
            let code = [
                sopp(0x05, (target - 4) / 4), // conditional entry to low, high, or setpc
                0xBE88_1F00,                  // getpc at 4, next PC 8
                0x8008_FF08,
                24, // low offset: candidate target 32
                0x8209_8009,
                0xBE80_2008,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                S_ENDPGM,
            ];
            assert_indirect(&code, 20, 32);
        }
    }

    #[test]
    fn far_jump_control_is_independent_of_decode_order() {
        let code = [
            sopp(0x05, 1), // queue getpc at 8
            sopp(0x02, 4), // visit setpc at 24 before its setup
            0xBE88_1F00,
            0x8008_FF08,
            20, // next PC 12 + 20 = candidate target 32
            0x8209_8009,
            0xBE80_2008,
            0xFFFF_FFFF,
            S_ENDPGM,
        ];
        assert_indirect(&code, 24, 32);
    }

    #[test]
    fn far_jump_setup_rechecks_entries_found_on_another_path() {
        for (low, absent) in [(24, 32), (36, 44), (0x1000, 32)] {
            let code = [
                sopp(0x05, 8), // queue the branch at 36, then visit the setup
                0xBE88_1F00,
                0x8008_FF08,
                low,
                0x8209_8009,
                0xBE80_2008,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,    // candidate target 32 is data, not code
                sopp(0x02, -5), // late entry to setpc at 20
                0xFFFF_FFFF,
                S_ENDPGM, // a valid speculative target at 44 must also be removed
            ];
            // Neither the invalid instruction at 32 nor an out-of-bounds
            // candidate may fail decoding before the late entry is known.
            assert_indirect(&code, 20, absent);
        }
    }

    #[test]
    fn far_jump_setup_rechecks_entries_from_far_jumps() {
        let code = [
            sopp(0x05, 8), // queue the other getpc at 36
            0xBE88_1F00,
            0x8008_FF08,
            24, // candidate target 32
            0x8209_8009,
            0xBE80_2008,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
            0xFFFF_FFFF,
            0xBE8A_1F00, // getpc s[10:11], next PC 40
            0x808A_FF0A,
            20, // subtract 20 -> entry to the first setpc at 20
            0x828B_800B,
            0xBE80_200A,
        ];
        assert_indirect(&code, 20, 32);
    }

    #[test]
    fn far_jump_control_survives_removal_of_speculative_code() {
        let code = [
            0xBE88_1F00,
            0x8008_FF08,
            20,
            0x8209_8009,
            0xBE80_2008, // setpc at 16, candidate target 24
            0xFFFF_FFFF,
            sopp(0x02, -3), // the candidate itself enters setpc at 16
        ];
        // Once that entry invalidates the chain, the branch at 24 is no
        // longer statically reachable. IR must retain the indirect exit.
        assert_indirect(&code, 16, 24);
    }

    #[test]
    fn far_jump_setup_allows_getpc_entries_and_ignores_data_branches() {
        let code = [
            sopp(0x05, 0), // entering getpc itself is safe
            0xBE88_1F00,
            0x8008_FF08,
            20, // next PC 8 + 20 = target 28
            0x8209_8009,
            0xBE80_2008,
            sopp(0x02, -2), // data at 24 looks like a branch to setpc
            S_ENDPGM,
        ];
        assert_eq!(summary(&code).last().unwrap().0, 28);
        let ir = crate::ir::build(&bytes(&code), false, 1).unwrap();
        assert!(ir.blocks.iter().any(|b| b.offset == 28));
        assert!(
            !ir.blocks
                .iter()
                .any(|b| b.exit == crate::ir::Exit::Indirect)
        );
    }

    #[test]
    fn rejects_bad_code() {
        assert!(decode(&[0; 3]).is_err(), "size not a multiple of 4");
        assert!(
            decode(&bytes(&[sopp(0x02, 100)])).is_err(),
            "branch out of range"
        );
        assert!(
            decode(&bytes(&[0xBE80_03FF])).is_err(),
            "literal past the end"
        );
        assert!(
            decode(&bytes(&[0xE400_0000, 0])).is_err(),
            "unknown encoding"
        );
        // s_cbranch_scc0 into the literal of the s_mov_b32 at 4.
        assert!(decode(&bytes(&[sopp(0x04, 1), 0xBE80_03FF, S_ENDPGM, S_ENDPGM])).is_err());
    }
}
