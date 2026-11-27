//! Sub-dword (`buffer_store_byte*`, `buffer_store_short*`) buffer store lowering.

use super::super::bitcode::{Attr, Bin, V, pred};
use super::super::shader::flags;
use super::{Lower, Why};
use crate::gfx10::Operand;
use crate::ir::Inst;
use crate::resource::{Site, Source};

pub(super) struct SubwordStore<'a> {
    pub site: Site,
    pub src: &'a Source,
    pub stride: u32,
    pub index: Option<V>,
    pub off: V,
    pub record_off: V,
    pub has_base_off: bool,
    pub may_cross_dword: bool,
    pub width: u32,
    pub hi16: bool,
    pub guard: Option<V>,
}

impl<'a> Lower<'a> {
    /// Lowers `buffer_store_byte*` and `buffer_store_short*` (raw or indexed
    /// structured UAV) as clear+set `atomicBinOp.i32` pairs. Halfword stores
    /// whose byte offset lands at `off & 3 == 3` split their upper byte into
    /// the next dword instead of silently truncating.
    pub(super) fn subword_buffer_store(
        &mut self,
        inst: &Inst,
        data: Option<&Operand>,
        st: SubwordStore<'a>,
    ) -> Result<(), Why> {
        let c = self.cur;
        let iu = self.s.m.undef(self.s.t.i32);
        let d = data.ok_or("buffer store data")?;
        let mut raw_val = self.vgprs(inst, d, 1)?[0];
        if st.hi16 {
            let k16 = self.s.m.i32(16);
            raw_val = self.s.m.bin(c, Bin::LShr, raw_val, k16);
        }
        let structured = match st.index {
            Some(index) => {
                let stride = st.stride;
                if stride < 4 || !stride.is_multiple_of(4) {
                    return Err("a structured buffer of unknown stride");
                }
                let h = self.handle(st.site, st.src, false, (12 | 0x1000, stride))?;
                Some((index, stride, h))
            }
            None => None,
        };

        let (not3, three) = (self.s.m.i32(!3), self.s.m.i32(3));
        let at = self.s.m.bin(c, Bin::And, st.off, not3);
        let byte = self.s.m.bin(c, Bin::And, st.off, three);
        let sh = self.s.m.bin(c, Bin::Shl, byte, three);
        let field = if st.width == 8 {
            self.s.m.i32(0xFF)
        } else if let Some((_, stride, _)) = structured {
            // When `record_off == stride - 1`, only byte 0 of the halfword lies
            // inside the record; clamp the upper byte even if an unaligned
            // `soffset` shifts `st.off` onto a 4-byte boundary.
            let last = self.s.m.i32(stride - 1);
            let at_edge = self.s.m.cmp(self.cur, pred::EQ, st.record_off, last);
            let (f8, f16) = (self.s.m.i32(0xFF), self.s.m.i32(0xFFFF));
            self.s.m.select(self.cur, at_edge, f8, f16)
        } else {
            self.s.m.i32(0xFFFF)
        };
        let x = self.s.m.bin(self.cur, Bin::And, raw_val, field);
        let x = self.s.m.bin(self.cur, Bin::Shl, x, sh);
        let mask = self.s.m.bin(self.cur, Bin::Shl, field, sh);
        let all = self.s.m.i32(u32::MAX);
        let keep = self.s.m.bin(self.cur, Bin::Xor, mask, all);

        let (h, coords, byte_at, inside) = match structured {
            Some((index, stride, h)) => {
                let s = self.s.m.i32(stride);
                // RDNA2 checks in-record `voffset + inst_offset < stride`;
                // `soffset` and descriptor base offsets displace across records.
                let rec_at = self.s.m.bin(self.cur, Bin::And, st.record_off, not3);
                let inside = self.s.m.cmp(self.cur, pred::IULT, rec_at, s);
                let (eff_idx, eff_at, inside) = if st.has_base_off {
                    let d_idx = self.s.m.bin(self.cur, Bin::UDiv, at, s);
                    let eff_idx = self.s.m.bin(self.cur, Bin::Add, index, d_idx);
                    // Drop writes whose `index + at / stride` wraps 32 bits.
                    let no_idx_wrap = self.s.m.cmp(self.cur, pred::IUGE, eff_idx, index);
                    let inside = self.s.m.bin(self.cur, Bin::And, inside, no_idx_wrap);
                    let eff_at = self.s.m.bin(self.cur, Bin::URem, at, s);
                    (eff_idx, eff_at, inside)
                } else {
                    (index, at, inside)
                };
                let row = self.s.m.bin(self.cur, Bin::Mul, eff_idx, s);
                let byte_at = self.s.m.bin(self.cur, Bin::Add, row, eff_at);
                (h, [eff_idx, eff_at, iu], byte_at, Some(inside))
            }
            None => {
                let (h, at) =
                    self.raw_view(st.site, st.src, at, 4, (11 | 0x1000, 0), true, st.guard)?;
                (h, [at, iu, iu], at, None)
            }
        };
        let store_guard = match (st.guard, inside) {
            (Some(g), Some(in_b)) => Some(self.s.m.bin(self.cur, Bin::And, g, in_b)),
            (g, in_b) => g.or(in_b),
        };
        let begin = self.begin_guarded(store_guard);
        let i32t = self.s.t.i32;
        for (op, v) in [(1, keep), (2, x)] {
            let args = [h, self.s.m.i32(op), coords[0], coords[1], coords[2], v];
            self.s
                .op(self.cur, "atomicBinOp.i32", Attr::NoUnwind, i32t, 78, &args);
        }
        self.mark_view_write(h, byte_at, 4, None);
        self.end_guarded(begin, &[]);

        if st.may_cross_dword {
            let crosses = self.s.m.cmp(self.cur, pred::EQ, byte, three);
            // Guard against 32-bit wrap of `at + 4` when `st.off == 0xFFFF_FFFF`.
            let max_at = self.s.m.i32(0xFFFF_FFFC);
            let no_wrap = self.s.m.cmp(self.cur, pred::NE, at, max_at);
            let crosses = self.s.m.bin(self.cur, Bin::And, crosses, no_wrap);
            let hi_guard = match st.guard {
                Some(g) => Some(self.s.m.bin(self.cur, Bin::And, g, crosses)),
                None => Some(crosses),
            };
            let four = self.s.m.i32(4);
            let at2 = self.s.m.bin(self.cur, Bin::Add, at, four);
            let k8 = self.s.m.i32(8);
            let ff = self.s.m.i32(0xFF);
            let hi_byte = self.s.m.bin(self.cur, Bin::LShr, raw_val, k8);
            let hi_byte = self.s.m.bin(self.cur, Bin::And, hi_byte, ff);
            let keep_hi = self.s.m.i32(!0xFFu32);
            let (h2, coords2, byte_at2, store_guard2) = match structured {
                Some((index, stride, h)) => {
                    let s = self.s.m.i32(stride);
                    // With `byte == 3`, `record_off + 1 < stride` iff
                    // `record_off < stride - 1` (avoids wrap on 0xFFFF_FFFF).
                    let limit = self.s.m.i32(stride - 1);
                    let inside_hi = self.s.m.cmp(self.cur, pred::IULT, st.record_off, limit);
                    let (eff_idx, eff_at, inside_hi) = if st.has_base_off {
                        let d_idx = self.s.m.bin(self.cur, Bin::UDiv, at2, s);
                        let eff_idx = self.s.m.bin(self.cur, Bin::Add, index, d_idx);
                        let no_idx_wrap = self.s.m.cmp(self.cur, pred::IUGE, eff_idx, index);
                        let inside_hi = self.s.m.bin(self.cur, Bin::And, inside_hi, no_idx_wrap);
                        let eff_at = self.s.m.bin(self.cur, Bin::URem, at2, s);
                        (eff_idx, eff_at, inside_hi)
                    } else {
                        (index, at2, inside_hi)
                    };
                    let g2 = match hi_guard {
                        Some(g) => Some(self.s.m.bin(self.cur, Bin::And, g, inside_hi)),
                        None => Some(inside_hi),
                    };
                    let row = self.s.m.bin(self.cur, Bin::Mul, eff_idx, s);
                    let byte_at2 = self.s.m.bin(self.cur, Bin::Add, row, eff_at);
                    (h, [eff_idx, eff_at, iu], byte_at2, g2)
                }
                None => {
                    let (h2, at2) =
                        self.raw_view(st.site, st.src, at2, 4, (11 | 0x1000, 0), true, hi_guard)?;
                    (h2, [at2, iu, iu], at2, hi_guard)
                }
            };
            let begin2 = self.begin_guarded(store_guard2);
            for (op, v) in [(1, keep_hi), (2, hi_byte)] {
                let args = [h2, self.s.m.i32(op), coords2[0], coords2[1], coords2[2], v];
                self.s
                    .op(self.cur, "atomicBinOp.i32", Attr::NoUnwind, i32t, 78, &args);
            }
            self.mark_view_write(h2, byte_at2, 4, None);
            self.end_guarded(begin2, &[]);
        }
        Ok(())
    }

    /// Lowers partial-channel (`count < 4`) typed buffer stores via a guarded
    /// load-modify-store (`bufferLoad` + `bufferStore` with mask `0xF`), using
    /// an integer-typed handle `(10 | u, 0x405)` when `VIEW_INTEGER` is set so
    /// untouched channels on wider integer and float views are preserved bit-exact.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn partial_typed_buffer_store(
        &mut self,
        site: Site,
        src: &'a Source,
        u: u32,
        h: V,
        element: V,
        vals: &[V],
        count: u32,
        guard: Option<V>,
    ) -> Result<(), Why> {
        let iu = self.s.m.undef(self.s.t.i32);
        let (byte, bytes) = match self.s.view_index.get(&h).copied() {
            Some(at) if self.x.view_marks => {
                let stride = self.stride_at(at);
                (self.s.m.bin(self.cur, Bin::Mul, element, stride), 16)
            }
            _ => (element, 16),
        };
        self.s.flags |= flags::TYPED_UAV_LOADS;
        let void = self.s.t.void;
        let mask_all = self.s.m.i8(0xF);
        if let Some((int, _)) = self.integer_view(h) {
            let h_int = self.handle(site, src, false, (10 | u, 0x405))?;
            let c = self.cur;
            let one = self.s.m.bool(true);
            let not_int = self.s.m.bin(c, Bin::Xor, int, one);
            let int_guard = match guard {
                Some(g) => Some(self.s.m.bin(c, Bin::And, g, int)),
                None => Some(int),
            };
            let flt_guard = match guard {
                Some(g) => Some(self.s.m.bin(c, Bin::And, g, not_int)),
                None => Some(not_int),
            };
            let i32t = self.s.t.i32;
            let res_i32 = self.s.m.named_struct("dx.types.ResRet.i32", vec![i32t; 5]);
            let at = self.begin_guarded(int_guard);
            let prev_i = self.s.op(
                self.cur,
                "bufferLoad.i32",
                Attr::ReadOnly,
                res_i32,
                68,
                &[h_int, element, iu],
            );
            let mut iargs = vec![h_int, element, iu];
            iargs.extend(vals);
            for ch in count..4 {
                iargs.push(self.s.m.extract(self.cur, prev_i, ch));
            }
            iargs.push(mask_all);
            self.s.op(
                self.cur,
                "bufferStore.i32",
                Attr::NoUnwind,
                void,
                69,
                &iargs,
            );
            self.mark_view_write(h, byte, bytes, None);
            self.end_guarded(at, &[]);
            let at = self.begin_guarded(flt_guard);
            let prev_f = self.s.op(
                self.cur,
                "bufferLoad.f32",
                Attr::ReadOnly,
                self.s.t.res_ret,
                68,
                &[h, element, iu],
            );
            let mut fargs = vec![h, element, iu];
            for &x in vals {
                fargs.push(self.as_float(x));
            }
            for ch in count..4 {
                fargs.push(self.s.m.extract(self.cur, prev_f, ch));
            }
            fargs.push(mask_all);
            self.s.op(
                self.cur,
                "bufferStore.f32",
                Attr::NoUnwind,
                void,
                69,
                &fargs,
            );
            self.mark_view_write(h, byte, bytes, None);
            self.end_guarded(at, &[]);
            return Ok(());
        }
        let at = self.begin_guarded(guard);
        let prev = self.s.op(
            self.cur,
            "bufferLoad.f32",
            Attr::ReadOnly,
            self.s.t.res_ret,
            68,
            &[h, element, iu],
        );
        let mut fargs = vec![h, element, iu];
        for &x in vals {
            fargs.push(self.as_float(x));
        }
        for ch in count..4 {
            fargs.push(self.s.m.extract(self.cur, prev, ch));
        }
        fargs.push(mask_all);
        self.s.op(
            self.cur,
            "bufferStore.f32",
            Attr::NoUnwind,
            void,
            69,
            &fargs,
        );
        self.mark_view_write(h, byte, bytes, None);
        self.end_guarded(at, &[]);
        Ok(())
    }
}
