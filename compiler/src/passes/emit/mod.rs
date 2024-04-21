mod binary;
pub mod elf;
mod mul_div;
mod push_pop;
mod special;
mod unary;

use crate::passes::assign::Arg;
use crate::passes::conclude::X86Concluded;
use crate::passes::emit::binary::{
    encode_binary_instr, ADDQ_INFO, ANDQ_INFO, CMPQ_INFO, MOVQ_INFO, ORQ_INFO, SUBQ_INFO, XORQ_INFO,
};
use crate::passes::emit::elf::{ElfFile, PRG_OFFSET};
use crate::passes::emit::mul_div::{encode_muldiv_instr, MulDivOpInfo};
use crate::passes::emit::push_pop::{encode_push_pop, POPQ_INFO, PUSHQ_INFO};
use crate::passes::emit::special::encode_setcc;
use crate::passes::emit::unary::{encode_unary_instr, CALLQ_INDIRECT_INFO, NEGQ_INFO};
use crate::passes::select::{Block, Cnd, Instr, Reg};
use crate::utils::unique_sym::UniqueSym;
use crate::{imm, time};
use std::collections::HashMap;

impl<'p> X86Concluded<'p> {
    #[must_use]
    pub fn emit(self) -> ElfFile {
        let mut machine_code = Vec::new();

        let mut rel_jumps = HashMap::new();
        let mut abs_jumps = HashMap::new();
        let mut addresses = HashMap::new();

        for (name, block) in &self.blocks {
            addresses.insert(name, machine_code.len());
            emit_block(block, &mut machine_code, &mut rel_jumps, &mut abs_jumps);
        }

        for (addr, block) in rel_jumps {
            let src = (addr + 4) as i32;
            let target = addresses[&block] as i32;
            let jump = target - src;

            machine_code[addr..addr + 4].copy_from_slice(&jump.to_le_bytes());
        }

        for (addr, block) in abs_jumps {
            let target = (PRG_OFFSET + addresses[&block]) as i32;

            machine_code[addr..addr + 4].copy_from_slice(&target.to_le_bytes());
        }

        let elf_file = ElfFile::new(addresses[&self.entry], machine_code);

        time!("emit");

        elf_file
    }
}

fn emit_block<'p>(
    block: &Block<'p, Arg>,
    machine_code: &mut Vec<u8>,
    rel_jumps: &mut HashMap<usize, UniqueSym<'p>>,
    abs_jumps: &mut HashMap<usize, UniqueSym<'p>>,
) {
    for instr in &block.instrs {
        emit_instr(instr, machine_code, rel_jumps, abs_jumps);
    }
}

fn emit_instr<'p>(
    instr: &Instr<Arg, UniqueSym<'p>>,
    machine_code: &mut Vec<u8>,
    rel_jumps: &mut HashMap<usize, UniqueSym<'p>>,
    abs_jumps: &mut HashMap<usize, UniqueSym<'p>>,
) {
    let v = match instr {
        Instr::Add { src, dst } => encode_binary_instr(ADDQ_INFO, src, dst),
        Instr::Sub { src, dst } => encode_binary_instr(SUBQ_INFO, src, dst),
        Instr::Mov { src, dst } => encode_binary_instr(MOVQ_INFO, src, dst),
        Instr::Neg { dst } => encode_unary_instr(NEGQ_INFO, dst),
        Instr::Push { src } => encode_push_pop(PUSHQ_INFO, src),
        Instr::Pop { dst } => encode_push_pop(POPQ_INFO, dst),
        Instr::Cmp { src, dst } => encode_binary_instr(CMPQ_INFO, src, dst),
        Instr::And { src, dst } => encode_binary_instr(ANDQ_INFO, src, dst),
        Instr::Or { src, dst } => encode_binary_instr(ORQ_INFO, src, dst),
        Instr::Xor { src, dst } => encode_binary_instr(XORQ_INFO, src, dst),
        Instr::Not { .. } => todo!(),
        Instr::Setcc { cnd } => encode_setcc(cnd),
        Instr::CallDirect { lbl, .. } => {
            rel_jumps.insert(machine_code.len() + 1, *lbl);
            vec![0xE8, 0x00, 0x00, 0x00, 0x00]
        }
        Instr::CallIndirect { src, .. } => encode_unary_instr(CALLQ_INDIRECT_INFO, src),
        Instr::Jmp { lbl } => {
            rel_jumps.insert(machine_code.len() + 1, *lbl);
            vec![0xE9, 0x00, 0x00, 0x00, 0x00]
        }
        Instr::Jcc { lbl, cnd } => {
            rel_jumps.insert(machine_code.len() + 2, *lbl);
            vec![0x0F, encode_cnd(*cnd), 0x00, 0x00, 0x00, 0x00]
        }
        Instr::Ret => vec![0xC3],
        Instr::Syscall { .. } => vec![0x0F, 0x05],
        Instr::Div { divisor } => encode_muldiv_instr(
            MulDivOpInfo {
                op: 0xF7,
                imm_as_src: 0b110,
            },
            divisor,
        ),
        Instr::Mul { src } => encode_muldiv_instr(
            MulDivOpInfo {
                op: 0xF7,
                imm_as_src: 0b100,
            },
            src,
        ),
        Instr::LoadLbl { lbl, dst } => {
            // This instruction creates a 32-bit placeholder and stores the offset so it can be set to `lbl` later.
            let bytes = encode_binary_instr(MOVQ_INFO, &imm!(0), dst);
            // The label should, regardless of the destination variant, always be stored in the last 4 bytes (we hope).
            abs_jumps.insert(machine_code.len() + bytes.len() - 4, *lbl);
            bytes
        }
        Instr::IDiv { .. } => todo!(),
        Instr::IMul { .. } => todo!(),
        Instr::MovSX { .. } => todo!(),
    };
    machine_code.extend(v);
}

fn encode_reg(reg: &Reg) -> (u8, u8) {
    match reg {
        Reg::RAX => (0, 0),
        Reg::RCX => (0, 1),
        Reg::RDX => (0, 2),
        Reg::RBX => (0, 3),
        Reg::RSP => (0, 4),
        Reg::RBP => (0, 5),
        Reg::RSI => (0, 6),
        Reg::RDI => (0, 7),
        Reg::R8 => (1, 0),
        Reg::R9 => (1, 1),
        Reg::R10 => (1, 2),
        Reg::R11 => (1, 3),
        Reg::R12 => (1, 4),
        Reg::R13 => (1, 5),
        Reg::R14 => (1, 6),
        Reg::R15 => (1, 7),
    }
}

fn encode_cnd(cnd: Cnd) -> u8 {
    match cnd {
        Cnd::Above => 0x87,
        Cnd::AboveOrEqual | Cnd::NotCarry => 0x83,
        Cnd::Below | Cnd::Carry => 0x82,
        Cnd::BelowOrEqual => 0x86,
        Cnd::EQ => 0x84,
        Cnd::GT => 0x8F,
        Cnd::GE => 0x8D,
        Cnd::LT => 0x8C,
        Cnd::LE => 0x8E,
        Cnd::NE => 0x85,
        Cnd::NotOverflow => 0x81,
        Cnd::ParityOdd => 0x8B,
        Cnd::NotSign => 0x89,
        Cnd::Overflow => 0x80,
        Cnd::ParityEven => 0x8A,
        Cnd::Sign => 0x88,
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! check {
    ($name: ident, $instr: expr, $expected: expr) => {
        #[test]
        fn $name() {
            let mut output = vec![];
            use std::collections::HashMap;
            use $crate::passes::emit::emit_instr;
            emit_instr(
                &$instr,
                &mut output,
                &mut HashMap::new(),
                &mut HashMap::new(),
            );

            assert_eq!(output, $expected);
        }
    };
}
