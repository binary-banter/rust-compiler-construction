use crate::interpreter::IO;
use crate::passes::conclude::X86Concluded;
use crate::passes::select::{
    Block, Cnd, Instr, Reg, VarArg, X86Selected, CALLEE_SAVED, CALLER_SAVED,
};
use crate::utils::gen_sym::UniqueSym;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;

#[derive(Default)]
pub struct Status {
    /// CF
    carry: bool,
    /// PF
    parity_even: bool,
    /// ZF = 1 (is zero)
    zero: bool,
    /// SF
    sign: bool,
    /// OF
    overflow: bool,
}

/// Stats gathered by the interpreter.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct IStats {
    pub branches_taken: usize,
    pub instructions_executed: usize,
}

pub struct X86Interpreter<'p, I: IO> {
    /// Maps labels to entry points.
    pub entries: &'p HashMap<UniqueSym<'p>, UniqueSym<'p>>,
    pub blocks: &'p HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
    pub block_ids: HashMap<usize, UniqueSym<'p>>,
    pub io: &'p mut I,
    /// Registers (`Reg`) mapped to their values in memory.
    pub regs: HashMap<Reg, i64>,
    /// Variables (`XVars`) mapped to their values in memory.
    pub vars: HashMap<UniqueSym<'p>, i64>,
    pub var_stack: Vec<HashMap<UniqueSym<'p>, i64>>,
    pub memory: HashMap<i64, i64>,
    /// Status register.
    pub status: Status,
    /// Stats for bencher.
    pub stats: IStats,
}

impl<'p, I: IO> X86Interpreter<'p, I> {
    fn new(
        entries: &'p HashMap<UniqueSym<'p>, UniqueSym<'p>>,
        blocks: &'p HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
        io: &'p mut I,
    ) -> Self {
        let block_ids = blocks.keys().map(|sym| (sym.id, *sym)).collect();

        let mut regs = HashMap::from_iter(
            CALLEE_SAVED
                .into_iter()
                .chain(CALLER_SAVED)
                .map(|reg| (reg, 0)),
        );

        regs.insert(Reg::RBP, i64::MAX - 7);
        regs.insert(Reg::RSP, i64::MAX - 7);

        Self {
            entries,
            blocks,
            block_ids,
            io,
            regs,
            vars: Default::default(),
            var_stack: Default::default(),
            memory: Default::default(),
            status: Default::default(),
            stats: Default::default(),
        }
    }
}

impl<'p> X86Concluded<'p> {
    pub fn interpret_with_stats(&self, io: &mut impl IO) -> (i64, IStats) {
        let entries = self.blocks.iter().map(|(&k, _v)| (k, k)).collect();

        let blocks = self
            .blocks
            .clone()
            .into_iter()
            .map(|(sym, block)| (sym, block.fmap(Into::into)))
            .collect();

        let mut state = X86Interpreter::new(&entries, &blocks, io);

        let val = state.run(self.entry);
        (val, state.stats)
    }

    pub fn interpret(&self, io: &mut impl IO) -> i64 {
        self.interpret_with_stats(io).0
    }
}

impl<'p> X86Selected<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> i64 {
        let entries = self
            .fns
            .iter()
            .map(|(&fun_sym, fun)| (fun_sym, fun.entry))
            .collect();

        let blocks: HashMap<_, _> = self
            .fns
            .values()
            .flat_map(|fun| fun.blocks.iter().map(|(&k, v)| (k, v.clone())))
            .collect();

        let mut state = X86Interpreter::new(&entries, &blocks, io);

        state.run(self.entry)
    }
}

impl<'p, I: IO> X86Interpreter<'p, I> {
    fn run(&mut self, entry: UniqueSym<'p>) -> i64 {
        let mut block_sym = self.entries[&entry];
        let mut instr_id = 0;

        loop {
            match &self.blocks[&block_sym].instrs[instr_id] {
                Instr::Addq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) + self.get_arg(dst));
                }
                Instr::Subq { src, dst } => {
                    self.set_arg(dst, self.get_arg(dst) - self.get_arg(src));
                }
                Instr::Negq { dst } => self.set_arg(dst, -self.get_arg(dst)),
                Instr::Movq { src, dst } => self.set_arg(dst, self.get_arg(src)),
                Instr::Pushq { src } => {
                    let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                    assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                    *rsp -= 8;
                    self.memory.insert(*rsp, self.get_arg(src));
                }
                Instr::Popq { dst } => {
                    let rsp = self.regs[&Reg::RSP];
                    assert_eq!(rsp % 8, 0, "Misaligned stack pointer.");
                    self.set_arg(dst, self.memory[&rsp]);
                    *self.regs.get_mut(&Reg::RSP).unwrap() += 8;
                }
                Instr::Jmp { lbl } => {
                    block_sym = *lbl;
                    instr_id = 0;
                    continue;
                }
                Instr::Retq => {
                    let rsp = self.regs[&Reg::RSP];
                    assert_eq!(rsp % 8, 0, "Misaligned stack pointer.");
                    let addr = self.memory[&rsp];
                    *self.regs.get_mut(&Reg::RSP).unwrap() += 8;

                    // Pop var context
                    self.vars = self.var_stack.pop().expect(
                        "Found more returns than we have had calls so far, ur program is weird m8",
                    );

                    (block_sym, instr_id) = self.destruct_addr(addr);
                    continue;
                }
                Instr::Syscall { .. } => match self.regs[&Reg::RAX] {
                    0x00 => self.syscall_read(),
                    0x01 => self.syscall_write(),
                    0x3C => return self.regs[&Reg::RDI],
                    _ => unreachable!(),
                },
                Instr::Divq { divisor } => {
                    let rax = self.regs[&Reg::RAX];
                    let rdx = self.regs[&Reg::RDX];
                    let dividend = (i128::from(rdx) << 64) | i128::from(rax);
                    let divisor = i128::from(self.get_arg(divisor));

                    self.regs.insert(Reg::RAX, (dividend / divisor) as i64);
                    self.regs.insert(Reg::RDX, (dividend % divisor) as i64);
                }
                Instr::Mulq { src } => {
                    let rax = self.regs[&Reg::RAX] as i128;
                    let src = self.get_arg(src) as i128;

                    let res = rax * src;

                    self.regs.insert(Reg::RAX, (res & (-1i64 as i128)) as i64);
                    self.regs.insert(Reg::RDX, (res >> 64) as i64);
                }
                Instr::Jcc { lbl, cnd } => {
                    self.stats.branches_taken += 1;
                    if self.evaluate_cnd(*cnd) {
                        block_sym = *lbl;
                        instr_id = 0;
                        continue;
                    }
                }
                Instr::Cmpq { src, dst } => {
                    assert!(
                        !matches!(dst, VarArg::Imm { .. }),
                        "Destination cannot be an immediate."
                    );

                    let src = self.get_arg(src);
                    let dst = self.get_arg(dst);

                    let (res, overflow) = dst.overflowing_sub(src);

                    // Maybe this can be done "prettier", but honestly it works.
                    let src = u64::from_ne_bytes(src.to_ne_bytes());
                    let dst = u64::from_ne_bytes(dst.to_ne_bytes());

                    self.status = Status {
                        carry: src > dst,
                        parity_even: res % 2 == 0,
                        zero: res == 0,
                        sign: res < 0,
                        overflow,
                    }
                }
                Instr::Andq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) & self.get_arg(dst));
                }
                Instr::Orq { src, dst } => self.set_arg(dst, self.get_arg(src) | self.get_arg(dst)),
                Instr::Xorq { src, dst } => {
                    self.set_arg(dst, self.get_arg(src) ^ self.get_arg(dst));
                }
                Instr::Notq { dst } => self.set_arg(dst, !self.get_arg(dst)),
                Instr::Setcc { cnd } => {
                    let rax = self.regs[&Reg::RAX];
                    let cnd = i64::from(self.evaluate_cnd(*cnd));
                    self.regs.insert(Reg::RAX, rax & !0xFF | cnd);
                }
                Instr::LoadLbl { lbl: sym, dst } => {
                    let val = self.fn_to_addr(*sym);
                    self.set_arg(dst, val);
                }
                Instr::CallqDirect { lbl, .. } => {
                    let ret_addr = self.instr_to_addr(block_sym, instr_id + 1);

                    let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                    assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                    *rsp -= 8;
                    self.memory.insert(*rsp, ret_addr);

                    //Push old var context
                    self.var_stack.push(mem::take(&mut self.vars));

                    (block_sym, instr_id) = self.destruct_addr(self.fn_to_addr(*lbl));
                    continue;
                }
                Instr::CallqIndirect { src, .. } => {
                    let ret_addr = self.instr_to_addr(block_sym, instr_id + 1);

                    let rsp = self.regs.get_mut(&Reg::RSP).unwrap();
                    assert_eq!(*rsp % 8, 0, "Misaligned stack pointer.");
                    *rsp -= 8;
                    self.memory.insert(*rsp, ret_addr);

                    let target = self.get_arg(src);

                    //Push old var context
                    self.var_stack.push(mem::take(&mut self.vars));

                    (block_sym, instr_id) = self.destruct_addr(target);
                    continue;
                }
            }
            instr_id += 1;
        }
    }

    /// Turns an instruction in a specified block into its address.
    fn instr_to_addr(&self, block_name: UniqueSym, instr_id: usize) -> i64 {
        // Please do not make more than 2^32 blocks or blocks with more than 2^32 instructions!
        ((block_name.id << 32) | instr_id) as i64
    }

    /// Turns a function label into its address.
    fn fn_to_addr(&self, entry: UniqueSym) -> i64 {
        self.instr_to_addr(self.entries[&entry], 0)
    }

    /// Destructures into a block label and instruction index.
    fn destruct_addr(&self, addr: i64) -> (UniqueSym<'p>, usize) {
        let block_id = (addr >> 32) as usize;
        let instr_id = (addr & 0xFF_FF_FF_FF) as usize;
        (self.block_ids[&block_id], instr_id)
    }

    /// Evaluates whether a condition holds.
    fn evaluate_cnd(&self, cnd: Cnd) -> bool {
        match cnd {
            Cnd::Above => !self.status.carry && !self.status.zero,
            Cnd::AboveOrEqual | Cnd::NotCarry => !self.status.carry,
            Cnd::Below | Cnd::Carry => self.status.carry,
            Cnd::BelowOrEqual => self.status.carry || self.status.zero,
            Cnd::EQ => self.status.zero,
            Cnd::GT => !self.status.zero && self.status.sign == self.status.overflow,
            Cnd::GE => self.status.sign == self.status.overflow,
            Cnd::LT => self.status.sign != self.status.overflow,
            Cnd::LE => self.status.zero || self.status.sign != self.status.overflow,
            Cnd::NE => !self.status.zero,
            Cnd::NotOverflow => !self.status.overflow,
            Cnd::NotSign => !self.status.sign,
            Cnd::Overflow => self.status.overflow,
            Cnd::ParityEven => self.status.parity_even,
            Cnd::ParityOdd => !self.status.parity_even,
            Cnd::Sign => self.status.sign,
        }
    }

    /// Retrieves the argument from memory. This can be from `regs`, `memory` or `vars`.
    fn get_arg(&self, a: &'p VarArg<UniqueSym<'p>>) -> i64 {
        match a {
            VarArg::Imm { val } => *val,
            VarArg::Reg { reg } => self.regs[reg],
            VarArg::Deref { reg, off } => self.memory[&(self.regs[reg] + off)],
            VarArg::XVar { sym } => *self
                .vars
                .get(sym)
                .unwrap_or_else(|| panic!("Expected to find variable {sym}")),
        }
    }

    /// Sets the argument in memory. This can be in `regs`, `memory` or `vars`.
    fn set_arg(&mut self, a: &'p VarArg<UniqueSym<'p>>, v: i64) {
        match a {
            VarArg::Imm { .. } => panic!("Tried to write to immediate, are u insane?"),
            VarArg::Reg { reg } => {
                self.regs.insert(*reg, v);
            }
            VarArg::Deref { reg, off } => {
                self.memory.insert(self.regs[reg] + off, v);
            }
            VarArg::XVar { sym } => {
                self.vars.insert(*sym, v);
            }
        }
    }

    /// Emulates a syscall read on Linux.
    fn syscall_read(&mut self) {
        let file = self.regs[&Reg::RDI];
        let buffer = self.regs[&Reg::RSI];
        let buffer_len = self.regs[&Reg::RDX];

        assert_eq!(file, 0, "Only reading from stdin is supported right now.");

        if buffer_len == 0 {
            self.memory.insert(buffer, 0); // This memory insert guarantees the interpreter finds a key.
            self.regs.insert(Reg::RAX, 0); // 0 bytes read
            return;
        }

        if let Some(read) = self.io.read() {
            self.memory.insert(buffer, read as i64);
            self.regs.insert(Reg::RAX, 1);
        } else {
            self.memory.insert(buffer, 0); // This memory insert guarantees the interpreter finds a key.
            self.regs.insert(Reg::RAX, 0); // 0 bytes read
        }
    }

    /// Emulates a syscall write on Linux.
    fn syscall_write(&mut self) {
        let file = self.regs[&Reg::RDI];
        let buffer = self.regs[&Reg::RSI];
        let buffer_len = self.regs[&Reg::RDX];

        assert_eq!(file, 1, "Only writing to stdout is supported right now.");

        assert_eq!(
            buffer_len, 1,
            "Only writing 1 byte at a time is supported right now."
        );

        let val = *self.memory.get(&buffer).unwrap();
        self.io.print(val as u8);
        self.regs.insert(Reg::RAX, 1);
    }
}
