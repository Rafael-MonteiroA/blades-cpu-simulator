use std::collections::{BTreeMap, VecDeque};

pub const MEM_SIZE: usize = 65_536;
pub const ENTRY: u16 = 0x0200;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub z: bool,
    pub c: bool,
    pub n: bool,
    pub v: bool,
}

#[derive(Clone, Debug)]
pub struct Cpu {
    pub r: [u16; 8],
    pub pc: u16,
    pub sp: u16,
    pub ir: u16,
    pub flags: Flags,
    pub memory: Vec<u8>,
    pub cycles: u64,
    pub halted: bool,
    pub changed_registers: Vec<String>,
    pub changed_memory: Vec<u16>,
    pub last_instruction: String,
    pub trace: VecDeque<String>,
    pub breakpoints: Vec<u16>,
    pub interrupt_depth: u16,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
impl Cpu {
    pub fn new() -> Self {
        Self {
            r: [0; 8],
            pc: ENTRY,
            sp: 0xfffe,
            ir: 0,
            flags: Flags::default(),
            memory: vec![0; MEM_SIZE],
            cycles: 0,
            halted: false,
            changed_registers: vec![],
            changed_memory: vec![],
            last_instruction: "—".into(),
            trace: VecDeque::new(),
            breakpoints: vec![],
            interrupt_depth: 0,
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn load(&mut self, bytes: &[u8], vectors: &[(u8, u16)]) {
        self.reset();
        let end = (ENTRY as usize + bytes.len()).min(MEM_SIZE);
        self.memory[ENTRY as usize..end].copy_from_slice(&bytes[..end - ENTRY as usize]);
        for (number, address) in vectors {
            self.memory[*number as usize * 2..*number as usize * 2 + 2]
                .copy_from_slice(&address.to_le_bytes());
        }
    }
    fn word(&self, addr: u16) -> u16 {
        u16::from_le_bytes([
            self.memory[addr as usize],
            self.memory[addr.wrapping_add(1) as usize],
        ])
    }
    fn read_word(&self, addr: u16) -> u16 {
        self.word(addr)
    }
    fn write_word(&mut self, addr: u16, v: u16) {
        let b = v.to_le_bytes();
        self.memory[addr as usize] = b[0];
        self.memory[addr.wrapping_add(1) as usize] = b[1];
        self.changed_memory.extend([addr, addr.wrapping_add(1)]);
    }
    fn fetch8(&mut self) -> u8 {
        let v = self.memory[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);
        v
    }
    fn fetch16(&mut self) -> u16 {
        let lo = self.fetch8();
        let hi = self.fetch8();
        u16::from_le_bytes([lo, hi])
    }
    fn reg(&mut self) -> usize {
        (self.fetch8() & 7) as usize
    }
    fn set_zn(&mut self, v: u16) {
        self.flags.z = v == 0;
        self.flags.n = v & 0x8000 != 0;
    }
    pub fn step(&mut self) -> Result<String, String> {
        if self.halted {
            return Ok("CPU parada".into());
        }
        self.changed_registers.clear();
        self.changed_memory.clear();
        let start = self.pc;
        let before_r = self.r;
        let before_pc = self.pc;
        let before_sp = self.sp;
        let before_ir = self.ir;
        let before_flags = self.flags;
        let op = self.fetch8();
        self.ir = op as u16;
        let name = match op {
            0x00 => "NOP".to_string(),
            0x01 => {
                self.halted = true;
                "HLT".into()
            }
            0x10 => {
                let d = self.reg();
                let v = self.fetch16();
                self.r[d] = v;
                self.set_zn(v);
                format!("MOV R{d}, {v}")
            }
            0x11 => {
                let d = self.reg();
                let s = self.reg();
                let v = self.r[s];
                self.r[d] = v;
                self.set_zn(v);
                format!("MOV R{d}, R{s}")
            }
            0x12 => {
                let d = self.reg();
                let a = self.fetch16();
                let v = self.read_word(a);
                self.r[d] = v;
                self.set_zn(v);
                format!("LOAD R{d}, [{a:04X}]")
            }
            0x13 => {
                let s = self.reg();
                let a = self.fetch16();
                let v = self.r[s];
                self.write_word(a, v);
                format!("STORE [{a:04X}], R{s}")
            }
            0x14 => {
                let s = self.reg();
                self.sp = self.sp.wrapping_sub(2);
                self.write_word(self.sp, self.r[s]);
                format!("PUSH R{s}")
            }
            0x15 => {
                let d = self.reg();
                let v = self.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
                self.r[d] = v;
                format!("POP R{d}")
            }
            0x20..=0x25 => {
                let d = self.reg();
                let s = self.reg();
                let a = self.r[d];
                let b = self.r[s];
                let wide = match op {
                    0x20 => (a as u32 + b as u32, format!("ADD R{d}, R{s}")),
                    0x21 => (a.wrapping_sub(b) as u32, format!("SUB R{d}, R{s}")),
                    0x22 => ((a as u32 * b as u32), format!("MUL R{d}, R{s}")),
                    0x23 => {
                        if b == 0 {
                            self.pc = start;
                            return Err(format!("DIVISÃO POR ZERO em {:04X}", start));
                        }
                        (a as u32 / b as u32, format!("DIV R{d}, R{s}"))
                    }
                    0x24 => ((a & b) as u32, format!("AND R{d}, R{s}")),
                    _ => ((a | b) as u32, format!("OR R{d}, R{s}")),
                };
                let v = wide.0 as u16;
                self.r[d] = v;
                self.flags.c = match op {
                    0x20 => a as u32 + b as u32 > 0xffff,
                    0x21 => a < b,
                    0x22 => a as u32 * b as u32 > 0xffff,
                    _ => false,
                };
                self.flags.v = match op {
                    0x20 => ((!(a ^ b) & (a ^ v)) & 0x8000) != 0,
                    0x21 => (((a ^ b) & (a ^ v)) & 0x8000) != 0,
                    _ => false,
                };
                self.set_zn(v);
                wide.1
            }
            0x26 | 0x27 => {
                let d = self.reg();
                let s = self.reg();
                let v = if op == 0x26 {
                    self.r[d] ^ self.r[s]
                } else {
                    !self.r[d]
                };
                self.r[d] = v;
                self.set_zn(v);
                self.flags.c = false;
                self.flags.v = false;
                if op == 0x26 {
                    format!("XOR R{d}, R{s}")
                } else {
                    format!("NOT R{d}")
                }
            }
            0x28 | 0x29 => {
                let d = self.reg();
                let n = self.fetch8() & 15;
                let v = if op == 0x28 {
                    self.r[d].wrapping_shl(n as u32)
                } else {
                    self.r[d] >> n
                };
                self.flags.c = if n == 0 {
                    false
                } else if op == 0x28 {
                    self.r[d] >> (16 - n) != 0
                } else {
                    (self.r[d] >> (n - 1)) & 1 != 0
                };
                self.r[d] = v;
                self.set_zn(v);
                format!("{} R{d}, {n}", if op == 0x28 { "SHL" } else { "SHR" })
            }
            0x2a | 0x2b => {
                let d = self.reg();
                let before = self.r[d];
                let v = before.wrapping_add(if op == 0x2a { 1 } else { u16::MAX });
                self.r[d] = v;
                self.set_zn(v);
                self.flags.c = if op == 0x2a {
                    before == u16::MAX
                } else {
                    before == 0
                };
                self.flags.v = if op == 0x2a {
                    before == 0x7fff
                } else {
                    before == 0x8000
                };
                format!("{} R{d}", if op == 0x2a { "INC" } else { "DEC" })
            }
            0x30 => {
                let a = self.reg();
                let b = self.reg();
                let x = self.r[a];
                let y = self.r[b];
                let z = x.wrapping_sub(y);
                self.flags.z = x == y;
                self.flags.n = z & 0x8000 != 0;
                self.flags.c = x < y;
                self.flags.v = (((x ^ y) & (x ^ z)) & 0x8000) != 0;
                format!("CMP R{a}, R{b}")
            }
            0x40..=0x45 => {
                let addr = self.fetch16();
                let take = match op {
                    0x40 => true,
                    0x41 => self.flags.z,
                    0x42 => !self.flags.z,
                    0x43 => self.flags.c,
                    0x44 => !self.flags.c,
                    0x45 => {
                        let ret = self.pc;
                        self.sp = self.sp.wrapping_sub(2);
                        self.write_word(self.sp, ret);
                        true
                    }
                    _ => false,
                };
                if take {
                    self.pc = addr;
                }
                format!(
                    "{} {addr:04X}",
                    match op {
                        0x40 => "JMP",
                        0x41 => "JZ",
                        0x42 => "JNZ",
                        0x43 => "JC",
                        0x44 => "JNC",
                        0x45 => "CALL",
                        _ => "JNC",
                    }
                )
            }
            0x46 => {
                self.pc = self.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
                "RET".into()
            }
            0x50 => {
                let vector = self.fetch16();
                if vector > 255 {
                    self.pc = start;
                    return Err(format!("Vetor de interrupção inválido: {vector}"));
                }
                let target = self.read_word(vector * 2);
                if target == 0 {
                    self.pc = start;
                    return Err(format!("Vetor {vector} sem handler"));
                }
                let f = (self.flags.z as u16)
                    | ((self.flags.c as u16) << 1)
                    | ((self.flags.n as u16) << 2)
                    | ((self.flags.v as u16) << 3);
                self.sp = self.sp.wrapping_sub(2);
                self.write_word(self.sp, f);
                for value in self.r {
                    self.sp = self.sp.wrapping_sub(2);
                    self.write_word(self.sp, value);
                }
                self.sp = self.sp.wrapping_sub(2);
                self.write_word(self.sp, self.pc);
                self.pc = target;
                self.interrupt_depth += 1;
                format!("INT {vector}")
            }
            0x51 => {
                if self.interrupt_depth == 0 {
                    self.pc = start;
                    return Err("IRET sem interrupção ativa".into());
                }
                self.pc = self.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
                for reg in (0..8).rev() {
                    self.r[reg] = self.read_word(self.sp);
                    self.sp = self.sp.wrapping_add(2);
                }
                let f = self.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
                self.flags = Flags {
                    z: f & 1 != 0,
                    c: f & 2 != 0,
                    n: f & 4 != 0,
                    v: f & 8 != 0,
                };
                self.interrupt_depth -= 1;
                "IRET".into()
            }
            _ => {
                self.pc = start;
                return Err(format!("Opcode inválido {op:02X} em {start:04X}"));
            }
        };
        let mut changes = Vec::new();
        for i in 0..8 {
            if before_r[i] != self.r[i] {
                changes.push(format!("R{i}"));
            }
        }
        if before_pc != self.pc {
            changes.push("PC".into());
        }
        if before_sp != self.sp {
            changes.push("SP".into());
        }
        if before_ir != self.ir {
            changes.push("IR".into());
        }
        if before_flags != self.flags {
            changes.push("FLAGS".into());
        }
        self.changed_registers = changes;
        self.cycles += 1;
        self.last_instruction = name.clone();
        let effects = self.changed_registers.join(",");
        let writes = self
            .changed_memory
            .iter()
            .map(|a| format!("M[{a:04X}]"))
            .collect::<Vec<_>>()
            .join(",");
        let summary = match (effects.is_empty(), writes.is_empty()) {
            (true, true) => String::new(),
            (false, true) => format!("  Δ {effects}"),
            (true, false) => format!("  Δ {writes}"),
            (false, false) => format!("  Δ {effects} {writes}"),
        };
        self.trace
            .push_back(format!("{:04X}  {}{}", start, name, summary));
        if self.trace.len() > 300 {
            self.trace.pop_front();
        }
        Ok(name)
    }
    pub fn run_batch(&mut self, limit: usize) -> Result<usize, String> {
        let mut n = 0;
        while n < limit && !self.halted {
            if self.breakpoints.contains(&self.pc) {
                break;
            }
            self.step()?;
            n += 1;
        }
        Ok(n)
    }
}

pub fn register_names() -> BTreeMap<&'static str, usize> {
    [
        ("R0", 0),
        ("R1", 1),
        ("R2", 2),
        ("R3", 3),
        ("R4", 4),
        ("R5", 5),
        ("R6", 6),
        ("R7", 7),
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;
    #[test]
    fn executes_arithmetic_and_flags() {
        let p = assemble("MOV R0, 65535\nMOV R1, 1\nADD R0, R1\nHLT").unwrap();
        let mut c = Cpu::new();
        c.load(&p.bytes, &p.vectors);
        c.run_batch(10).unwrap();
        assert_eq!(c.r[0], 0);
        assert!(c.flags.z);
        assert!(c.flags.c);
        assert!(c.halted);
    }
    #[test]
    fn executes_stack_and_subroutine() {
        let p = assemble(
            "MOV R0, 5\nCALL DOUBLE\nHLT\nDOUBLE:\nPUSH R1\nMOV R1, R0\nADD R0, R1\nPOP R1\nRET",
        )
        .unwrap();
        let mut c = Cpu::new();
        c.load(&p.bytes, &p.vectors);
        c.run_batch(30).unwrap();
        assert_eq!(c.r[0], 10);
        assert_eq!(c.sp, 0xfffe);
        assert!(c.halted);
    }
    #[test]
    fn division_by_zero_is_reported() {
        let p = assemble("MOV R0, 8\nMOV R1, 0\nDIV R0, R1").unwrap();
        let mut c = Cpu::new();
        c.load(&p.bytes, &p.vectors);
        c.run_batch(2).unwrap();
        assert!(c.step().unwrap_err().contains("ZERO"));
        assert_eq!(c.cycles, 2);
        assert_eq!(c.pc, ENTRY + 8);
    }
    #[test]
    fn breakpoint_stops_before_executing_its_address() {
        let p = assemble("MOV R0, 7\nHLT").unwrap();
        let mut c = Cpu::new();
        c.load(&p.bytes, &p.vectors);
        c.breakpoints.push(ENTRY);
        assert_eq!(c.run_batch(10).unwrap(), 0);
        assert_eq!(c.pc, ENTRY);
        assert_eq!(c.cycles, 0);
        c.breakpoints.clear();
        assert_eq!(c.run_batch(1).unwrap(), 1);
        assert_eq!(c.r[0], 7);
        assert_eq!(c.cycles, 1);
    }
    #[test]
    fn invalid_opcode_does_not_advance_execution_state() {
        let mut c = Cpu::new();
        c.memory[ENTRY as usize] = 0xff;
        let error = c.step().unwrap_err();
        assert!(error.contains("FF"));
        assert_eq!(c.pc, ENTRY);
        assert_eq!(c.cycles, 0);
        assert!(c.trace.is_empty());
        assert_eq!(c.last_instruction, "—");
    }
    #[test]
    fn interrupt_saves_and_restores_context() {
        let p = assemble(".VECTOR 1, HANDLER\nMOV R0, 7\nMOV R3, 1\nINT 1\nHLT\nHANDLER:\nMOV R2, 0xCAFE\nSTORE [0x4000], R2\nIRET").unwrap();
        let mut c = Cpu::new();
        c.load(&p.bytes, &p.vectors);
        c.run_batch(20).unwrap();
        assert_eq!(c.r[0], 7);
        assert_eq!(
            u16::from_le_bytes([c.memory[0x4000], c.memory[0x4001]]),
            0xcafe
        );
        assert_eq!(c.sp, 0xfffe);
        assert_eq!(c.interrupt_depth, 0);
        assert!(c.halted);
    }
    #[test]
    fn reset_clears_ram_and_machine_state() {
        let mut c = Cpu::new();
        c.memory[0x4000] = 0xaa;
        c.r[3] = 9;
        c.cycles = 4;
        c.reset();
        assert_eq!(c.memory[0x4000], 0);
        assert_eq!(c.r[3], 0);
        assert_eq!(c.cycles, 0);
        assert_eq!(c.pc, ENTRY);
    }
    #[test]
    fn step_records_real_register_flag_and_pc_deltas() {
        let p = assemble("MOV R0, 65535\nMOV R1, 1\nADD R0, R1").unwrap();
        let mut c = Cpu::new();
        c.load(&p.bytes, &p.vectors);
        c.run_batch(2).unwrap();
        c.step().unwrap();
        assert!(c.changed_registers.contains(&"R0".to_string()));
        assert!(c.changed_registers.contains(&"PC".to_string()));
        assert!(c.changed_registers.contains(&"FLAGS".to_string()));
        assert!(
            c.trace
                .back()
                .is_some_and(|entry| entry.contains("Δ R0") && entry.contains("FLAGS"))
        );
    }
    #[test]
    fn shipped_programs_assemble_and_run() {
        for (source, expected) in [
            (include_str!("../examples/calculator.asm"), (0, 150)),
            (include_str!("../examples/fibonacci.asm"), (1, 21)),
            (include_str!("../examples/factorial.asm"), (1, 120)),
            (include_str!("../examples/memory.asm"), (1, 0xbeef)),
            (include_str!("../examples/interrupts.asm"), (0, 7)),
        ] {
            let p = assemble(source).unwrap();
            let mut c = Cpu::new();
            c.load(&p.bytes, &p.vectors);
            c.run_batch(500).unwrap();
            assert!(c.halted, "program did not halt: {source}");
            if source.contains("STORE [0x4000]") {
                assert_eq!(
                    u16::from_le_bytes([c.memory[0x4000], c.memory[0x4001]]),
                    if source.contains(".VECTOR") {
                        0xcafe
                    } else {
                        expected.1
                    }
                );
            } else {
                assert_eq!(c.r[expected.0], expected.1);
            }
        }
    }
}
