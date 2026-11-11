use crate::cpu::ENTRY;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("linha {line}: {message}")]
pub struct AsmError {
    pub line: usize,
    pub message: String,
}
#[derive(Clone, Debug)]
pub struct Program {
    pub bytes: Vec<u8>,
    pub lines: Vec<String>,
    pub labels: HashMap<String, u16>,
    pub vectors: Vec<(u8, u16)>,
    pub line_addresses: Vec<(usize, u16)>,
}
fn reg(s: &str, line: usize) -> Result<u8, AsmError> {
    s.trim()
        .to_uppercase()
        .strip_prefix('R')
        .and_then(|x| x.parse::<u8>().ok())
        .filter(|x| *x < 8)
        .ok_or_else(|| AsmError {
            line,
            message: format!("registrador inválido: {s}"),
        })
}
fn is_indirect_register(s: &str) -> bool {
    s.trim()
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .is_some_and(|inner| reg(inner, 0).is_ok())
}
fn indirect_reg(s: &str, line: usize) -> Result<u8, AsmError> {
    let inner = s
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| AsmError {
            line,
            message: format!("esperado endereço indireto [Rn]: {s}"),
        })?;
    reg(inner, line)
}
fn value(s: &str, labels: &HashMap<String, u16>, line: usize) -> Result<u16, AsmError> {
    let s = s.trim().trim_matches(|c| c == '[' || c == ']');
    if let Some(v) = labels.get(&s.to_uppercase()) {
        return Ok(*v);
    }
    let n = parse_number(s).map_err(|_| AsmError {
        line,
        message: format!("valor ou label inválido: {s}"),
    })?;
    if !(-32768..=65535).contains(&n) {
        return Err(AsmError {
            line,
            message: format!("valor fora da faixa de 16 bits: {s}"),
        });
    }
    Ok(n as u16)
}
fn parse_number(s: &str) -> Result<i32, ()> {
    let (negative, unsigned) = s.strip_prefix('-').map_or((false, s), |x| (true, x));
    let parsed = if let Some(x) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        i32::from_str_radix(x, 16)
    } else if let Some(x) = unsigned
        .strip_suffix('h')
        .or_else(|| unsigned.strip_suffix('H'))
    {
        i32::from_str_radix(x, 16)
    } else {
        unsigned.parse::<i32>()
    }
    .map_err(|_| ())?;
    Ok(if negative { -parsed } else { parsed })
}
fn byte_value(s: &str, labels: &HashMap<String, u16>, line: usize) -> Result<u8, AsmError> {
    let raw = labels
        .get(&s.trim().to_uppercase())
        .map(|v| *v as i32)
        .or_else(|| parse_number(s.trim()).ok())
        .ok_or_else(|| AsmError {
            line,
            message: format!("byte inválido: {s}"),
        })?;
    if !(-128..=255).contains(&raw) {
        return Err(AsmError {
            line,
            message: format!(".BYTE fora da faixa -128..255: {s}"),
        });
    }
    Ok(raw as u8)
}
fn split_ops(s: &str) -> Vec<String> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',').map(|x| x.trim().to_string()).collect()
}
fn valid_label(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn size(code: &str, line: usize) -> Result<usize, AsmError> {
    let mut s = code.trim();
    if s.is_empty() {
        return Ok(0);
    }
    if s.contains(':') {
        s = s.split_once(':').map(|(_, b)| b).unwrap_or("").trim()
    }
    if s.is_empty() {
        return Ok(0);
    }
    let (mn, rest) = s.split_once(char::is_whitespace).unwrap_or((s, ""));
    let m = mn.to_uppercase();
    let o = split_ops(rest);
    let n = match m.as_str() {
        "NOP" | "HLT" | "RET" | "IRET" => 1,
        "PUSH" | "POP" => 2,
        "MOV" if o.len() == 2 && o[1].to_uppercase().starts_with('R') => 3,
        "MOV" => 4,
        "LOAD"
            if o.get(1)
                .is_some_and(|operand| is_indirect_register(operand)) =>
        {
            3
        }
        "STORE"
            if o.first()
                .is_some_and(|operand| is_indirect_register(operand)) =>
        {
            3
        }
        "LOAD" | "STORE" => 4,
        "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "OR" | "XOR" | "CMP" => 3,
        "NOT" | "INC" | "DEC" => 2,
        "SHL" | "SHR" => 3,
        "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "CALL" | "INT" => 3,
        ".ORG" => 0,
        ".VECTOR" => 0,
        ".BYTE" => o.len(),
        ".WORD" => 2 * o.len(),
        _ => {
            return Err(AsmError {
                line,
                message: format!("instrução desconhecida: {m}"),
            });
        }
    };
    let expected = match m.as_str() {
        "NOP" | "HLT" | "RET" | "IRET" => Some(0),
        "PUSH" | "POP" | "NOT" | "INC" | "DEC" => Some(1),
        "MOV" | "LOAD" | "STORE" | "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "OR" | "XOR" | "CMP"
        | "SHL" | "SHR" | ".VECTOR" => Some(2),
        "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "CALL" | "INT" => Some(1),
        ".ORG" => Some(1),
        ".BYTE" | ".WORD" => {
            if o.is_empty() {
                return Err(AsmError {
                    line,
                    message: format!("{m} requer pelo menos um valor"),
                });
            }
            None
        }
        _ => None,
    };
    if let Some(count) = expected {
        if o.len() != count {
            return Err(AsmError {
                line,
                message: format!("{m} requer {count} operando(s), recebeu {}", o.len()),
            });
        }
    }
    Ok(n)
}
pub fn assemble(source: &str) -> Result<Program, AsmError> {
    let mut labels = HashMap::new();
    let mut pc = ENTRY as usize;
    for (idx, raw) in source.lines().enumerate() {
        let line = idx + 1;
        let code = raw.split(';').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        let mut body = code;
        if let Some((label, rest)) = code.split_once(':') {
            let l = label.trim().to_uppercase();
            if !valid_label(&l) || labels.insert(l.clone(), pc as u16).is_some() {
                return Err(AsmError {
                    line,
                    message: format!("label duplicada ou vazia: {l}"),
                });
            }
            body = rest.trim();
            if body.is_empty() {
                continue;
            }
        }
        let (m, r) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        if m.eq_ignore_ascii_case(".ORG") {
            if split_ops(r).len() != 1 {
                return Err(AsmError {
                    line,
                    message: ".ORG requer um endereço".into(),
                });
            }
            let target = value(r, &labels, line)? as usize;
            if target < ENTRY as usize || target < pc {
                return Err(AsmError {
                    line,
                    message: ".ORG deve avançar dentro da área de programa".into(),
                });
            }
            pc = target;
            continue;
        }
        if m.eq_ignore_ascii_case(".VECTOR") {
            if split_ops(r).len() != 2 {
                return Err(AsmError {
                    line,
                    message: ".VECTOR requer número e label/endereço".into(),
                });
            }
            continue;
        }
        pc = pc
            .checked_add(size(body, line)?)
            .filter(|n| *n <= 65536)
            .ok_or_else(|| AsmError {
                line,
                message: "programa excede os 64 KB".into(),
            })?;
    }
    let mut bytes = Vec::new();
    let mut vectors = Vec::new();
    let mut vector_numbers = HashSet::new();
    let mut line_addresses = Vec::new();
    let mut pc = ENTRY;
    for (idx, raw) in source.lines().enumerate() {
        let line = idx + 1;
        let code = raw.split(';').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        let body = code.split_once(':').map(|(_, b)| b.trim()).unwrap_or(code);
        if body.is_empty() {
            continue;
        }
        let (mn, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        let m = mn.to_uppercase();
        let o = split_ops(rest);
        if m == ".ORG" {
            let target = value(rest, &labels, line)?;
            if (target as usize) < ENTRY as usize || target < pc {
                return Err(AsmError {
                    line,
                    message: ".ORG deve avançar dentro da área de programa".into(),
                });
            }
            while pc < target {
                bytes.push(0);
                pc = pc.wrapping_add(1);
            }
            continue;
        }
        if m == ".VECTOR" {
            if o.len() != 2 {
                return Err(AsmError {
                    line,
                    message: ".VECTOR requer número e label/endereço".into(),
                });
            }
            let number = value(&o[0], &labels, line)?;
            if number > 255 {
                return Err(AsmError {
                    line,
                    message: "número do vetor deve estar entre 0 e 255".into(),
                });
            }
            if !vector_numbers.insert(number as u8) {
                return Err(AsmError {
                    line,
                    message: format!("vetor {number} definido mais de uma vez"),
                });
            }
            vectors.push((number as u8, value(&o[1], &labels, line)?));
            continue;
        }
        if !m.starts_with('.') {
            line_addresses.push((line, pc));
        }
        if m == ".BYTE" {
            for x in o {
                bytes.push(byte_value(&x, &labels, line)?);
                pc = pc.wrapping_add(1);
            }
            continue;
        }
        if m == ".WORD" {
            for x in o {
                bytes.extend(value(&x, &labels, line)?.to_le_bytes());
                pc = pc.wrapping_add(2);
            }
            continue;
        }
        let opcode = match m.as_str() {
            "NOP" => 0,
            "HLT" => 1,
            "MOV" if o.get(1).is_some_and(|x| x.to_uppercase().starts_with('R')) => 0x11,
            "MOV" => 0x10,
            "LOAD"
                if o.get(1)
                    .is_some_and(|operand| is_indirect_register(operand)) =>
            {
                0x16
            }
            "STORE"
                if o.first()
                    .is_some_and(|operand| is_indirect_register(operand)) =>
            {
                0x17
            }
            "LOAD" => 0x12,
            "STORE" => 0x13,
            "PUSH" => 0x14,
            "POP" => 0x15,
            "ADD" => 0x20,
            "SUB" => 0x21,
            "MUL" => 0x22,
            "DIV" => 0x23,
            "AND" => 0x24,
            "OR" => 0x25,
            "XOR" => 0x26,
            "NOT" => 0x27,
            "SHL" => 0x28,
            "SHR" => 0x29,
            "INC" => 0x2a,
            "DEC" => 0x2b,
            "CMP" => 0x30,
            "JMP" => 0x40,
            "JZ" => 0x41,
            "JNZ" => 0x42,
            "JC" => 0x43,
            "JNC" => 0x44,
            "CALL" => 0x45,
            "RET" => 0x46,
            "INT" => 0x50,
            "IRET" => 0x51,
            _ => {
                return Err(AsmError {
                    line,
                    message: format!("instrução desconhecida: {m}"),
                });
            }
        };
        bytes.push(opcode);
        pc = pc.wrapping_add(1);
        let need = |i: usize| {
            o.get(i).ok_or_else(|| AsmError {
                line,
                message: format!("faltando operando para {m}"),
            })
        };
        match m.as_str() {
            "NOP" | "HLT" | "RET" | "IRET" => {}
            "PUSH" | "POP" => {
                bytes.push(reg(need(0)?, line)?);
                pc = pc.wrapping_add(1)
            }
            "MOV" if opcode == 0x10 => {
                let d = reg(need(0)?, line)?;
                bytes.push(d);
                bytes.extend(value(need(1)?, &labels, line)?.to_le_bytes());
                pc = pc.wrapping_add(3)
            }
            "MOV" => {
                let d = reg(need(0)?, line)?;
                let s = reg(need(1)?, line)?;
                bytes.extend([d, s]);
                pc = pc.wrapping_add(2)
            }
            "LOAD" if opcode == 0x16 => {
                bytes.extend([reg(need(0)?, line)?, indirect_reg(need(1)?, line)?]);
                pc = pc.wrapping_add(2)
            }
            "STORE" if opcode == 0x17 => {
                bytes.extend([indirect_reg(need(0)?, line)?, reg(need(1)?, line)?]);
                pc = pc.wrapping_add(2)
            }
            "LOAD" | "STORE" => {
                let r = reg(need(if m == "LOAD" { 0 } else { 1 })?, line)?;
                let addr = value(need(if m == "LOAD" { 1 } else { 0 })?, &labels, line)?;
                bytes.extend([r]);
                bytes.extend(addr.to_le_bytes());
                pc = pc.wrapping_add(3)
            }
            "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "OR" | "XOR" | "CMP" => {
                bytes.extend([reg(need(0)?, line)?, reg(need(1)?, line)?]);
                pc = pc.wrapping_add(2)
            }
            "NOT" | "INC" | "DEC" => {
                bytes.push(reg(need(0)?, line)?);
                pc = pc.wrapping_add(1)
            }
            "SHL" | "SHR" => {
                bytes.push(reg(need(0)?, line)?);
                bytes.push(value(need(1)?, &labels, line)? as u8);
                pc = pc.wrapping_add(2)
            }
            "JMP" | "JZ" | "JNZ" | "JC" | "JNC" | "CALL" | "INT" => {
                bytes.extend(value(need(0)?, &labels, line)?.to_le_bytes());
                pc = pc.wrapping_add(2)
            }
            _ => {}
        }
    }
    let lines = source.lines().map(str::to_string).collect();
    Ok(Program {
        bytes,
        lines,
        labels,
        vectors,
        line_addresses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assembles_and_resolves_labels() {
        let p = assemble("MOV R0, 7\nJMP done\nNOP\ndone: HLT").unwrap();
        assert_eq!(p.bytes[0..4], [0x10, 0, 7, 0]);
        assert_eq!(p.labels["DONE"], ENTRY + 8);
    }
    #[test]
    fn reports_line() {
        assert!(assemble("MOV R9, 1").unwrap_err().line == 1);
    }
    #[test]
    fn accepts_signed_words_and_checks_byte_range() {
        let p = assemble("MOV R0, -1\n.BYTE -1, 255").unwrap();
        assert_eq!(&p.bytes[2..4], &[0xff, 0xff]);
        assert_eq!(&p.bytes[4..6], &[0xff, 0xff]);
        assert!(assemble(".BYTE 256").unwrap_err().message.contains("faixa"));
    }
    #[test]
    fn validates_arity_vectors_and_origin() {
        assert!(assemble("NOP R0").unwrap_err().message.contains("operando"));
        assert!(
            assemble(".VECTOR 1, H\n.VECTOR 1, H\nH: IRET")
                .unwrap_err()
                .message
                .contains("mais de uma vez")
        );
        assert!(assemble(".ORG 0x0300\n.ORG 0x0200").is_err());
        assert!(assemble("1BAD: HLT").is_err());
    }
    #[test]
    fn validates_program_end_at_memory_boundary() {
        let last_byte = assemble(".ORG 0xFFFF\n.BYTE 0xAA").unwrap();
        assert_eq!(last_byte.bytes.len(), 0x10000 - ENTRY as usize);
        assert_eq!(last_byte.bytes.last(), Some(&0xaa));
        assert!(assemble(".ORG 0xFFFF\n.WORD 1").is_err());
    }
    #[test]
    fn accepts_case_insensitive_labels_and_hex_suffix() {
        let p = assemble("JMP end\n.BYTE 0Ah\nEND: HLT").unwrap();
        assert_eq!(p.labels["END"], ENTRY + 4);
        assert_eq!(p.bytes[3], 10);
        assert_eq!(p.bytes[4], 1);
    }
}
