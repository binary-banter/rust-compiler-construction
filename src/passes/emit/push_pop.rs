use crate::language::x86var::Arg;
use crate::passes::emit;

pub struct PushPopInfo {
    pub op_reg: u8,
    pub op_deref: u8,
    pub op_imm: u8,
    pub imm_as_src: u8,
}

pub fn encode_push_pop(op_info: PushPopInfo, reg: &Arg) -> Vec<u8> {
    match reg {
        Arg::Imm { val } => {
            let val = *val as i32;

            let mut v = vec![op_info.op_imm];
            v.extend(val.to_le_bytes());
            v
        }
        Arg::Reg { reg } => {
            let (r, rrr) = emit::encode_reg(reg);
            if r == 0 {
                vec![op_info.op_reg | rrr]
            } else {
                vec![0x41, op_info.op_reg | rrr]
            }
        }
        Arg::Deref { reg, off } => {
            let (r, rrr) = emit::encode_reg(reg);
            let off = *off as i32;

            let mut v = if r == 0 {
                vec![
                    op_info.op_deref,
                    0b10_000_000 | op_info.imm_as_src << 3 | rrr,
                ]
            } else {
                vec![
                    0x41,
                    op_info.op_deref,
                    0b10_000_000 | op_info.imm_as_src << 3 | rrr,
                ]
            };
            v.extend(off.to_le_bytes());
            v
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::language::x86var::{Arg, Instr, Reg};
    use crate::*;

    mod push {
        use super::*;

        check!(reg1, pushq!(reg!(RAX)), vec![0x50]);
        check!(reg2, pushq!(reg!(R14)), vec![0x41, 0x56]);

        check!(
            deref1,
            pushq!(deref!(RDX, i32::MAX as i64)),
            vec![0xFF, 0xB2, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref2,
            pushq!(deref!(R11, i32::MAX as i64)),
            vec![0x41, 0xFF, 0xB3, 0xFF, 0xFF, 0xFF, 0x7F]
        );

        check!(
            imm,
            pushq!(imm!(i32::MAX as i64)),
            vec![0x68, 0xFF, 0xFF, 0xFF, 0x7F]
        );
    }

    mod pop {
        use super::*;

        check!(reg1, popq!(reg!(RAX)), vec![0x58]);
        check!(reg2, popq!(reg!(R14)), vec![0x41, 0x5E]);

        check!(
            deref1,
            popq!(deref!(RDX, i32::MAX as i64)),
            vec![0x8F, 0x82, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref2,
            popq!(deref!(R11, i32::MAX as i64)),
            vec![0x41, 0x8F, 0x83, 0xFF, 0xFF, 0xFF, 0x7F]
        );
    }
}
