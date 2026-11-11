use crate::{
    assembler,
    cpu::{Cpu, ENTRY},
};
use eframe::egui::{
    self, Color32, RichText, Stroke, Vec2,
    text::{LayoutJob, TextFormat},
};

pub struct BladesApp {
    cpu: Cpu,
    source: String,
    assembled: bool,
    running: bool,
    speed: usize,
    message: String,
    memory_address: String,
    breakpoint_address: String,
    file_path: String,
    memory_edit_address: String,
    memory_edit_value: String,
    source_lines: Vec<String>,
    line_addresses: Vec<(usize, u16)>,
    error: Option<String>,
    show_memory: bool,
}
impl BladesApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.window_fill = Color32::from_rgb(18, 22, 29);
        style.visuals.panel_fill = Color32::from_rgb(16, 20, 27);
        style.visuals.extreme_bg_color = Color32::from_rgb(10, 13, 18);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(25, 125, 160);
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        cc.egui_ctx.set_style(style);
        Self {
            cpu: Cpu::new(),
            source: include_str!("../examples/hello.asm").into(),
            assembled: false,
            running: false,
            speed: 32,
            message: "Pronto · BLADE-16 · 64 KB".into(),
            memory_address: "0200".into(),
            breakpoint_address: "0200".into(),
            file_path: String::new(),
            memory_edit_address: "4000".into(),
            memory_edit_value: "00".into(),
            source_lines: vec![],
            line_addresses: vec![],
            error: None,
            show_memory: true,
        }
    }
    fn assemble(&mut self) {
        match assembler::assemble(&self.source) {
            Ok(p) => {
                self.cpu.load(&p.bytes, &p.vectors);
                self.source_lines = p.lines;
                self.line_addresses = p.line_addresses;
                self.assembled = true;
                self.running = false;
                self.error = None;
                self.message = format!(
                    "Montagem concluída · {} bytes · {} labels · entrada {:04X}",
                    p.bytes.len(),
                    p.labels.len(),
                    ENTRY
                );
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.message = format!("Erro de montagem · {e}");
            }
        }
    }
    fn card<R>(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(20, 25, 33))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(42, 51, 63)))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(title.to_uppercase())
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(125, 155, 178)),
                    );
                });
                ui.add_space(6.0);
                add(ui)
            })
            .inner
    }
    fn mono(s: impl Into<String>) -> RichText {
        RichText::new(s.into()).monospace()
    }
    fn pc_source_line(&self) -> Option<usize> {
        if !self.assembled || self.source.trim_end_matches('\n') != self.source_lines.join("\n") {
            return None;
        }
        self.line_addresses
            .iter()
            .find(|(_, address)| *address == self.cpu.pc)
            .map(|(line, _)| *line)
    }
    fn source_at(&self, address: u16) -> Option<(usize, &str)> {
        let line = self.line_addresses.iter().find(|(_, pc)| *pc == address)?.0;
        Some((line, self.source_lines.get(line.saturating_sub(1))?.trim()))
    }
    fn next_source_at(&self, address: u16) -> Option<(usize, &str, u16)> {
        let (_, next_address) = self
            .line_addresses
            .iter()
            .filter(|(_, pc)| *pc > address)
            .min_by_key(|(_, pc)| *pc)?;
        let (line, text) = self.source_at(*next_address)?;
        Some((line, text, *next_address))
    }
    fn assembly_layout(
        ui: &egui::Ui,
        source: &str,
        wrap_width: f32,
        highlighted_line: Option<usize>,
    ) -> std::sync::Arc<egui::Galley> {
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        for (line_number, line) in source.split_inclusive('\n').enumerate() {
            let mut i = 0;
            while i < line.len() {
                let ch = line[i..].chars().next().unwrap_or(' ');
                let start = i;
                let color;
                if ch == ';' {
                    color = Color32::from_rgb(112, 151, 126);
                    i = line.len();
                } else if ch.is_whitespace() {
                    color = Color32::from_rgb(205, 211, 221);
                    i += ch.len_utf8();
                    while i < line.len()
                        && line[i..].chars().next().is_some_and(char::is_whitespace)
                    {
                        i += line[i..].chars().next().map(char::len_utf8).unwrap_or(1);
                    }
                } else if ch == ',' || ch == '[' || ch == ']' {
                    color = Color32::from_rgb(164, 180, 195);
                    i += ch.len_utf8();
                } else {
                    i += ch.len_utf8();
                    while i < line.len() {
                        let c = line[i..].chars().next().unwrap_or(' ');
                        if c.is_whitespace() || matches!(c, ',' | '[' | ']' | ';') {
                            break;
                        }
                        i += c.len_utf8();
                    }
                    let token = line[start..i].trim_end_matches(':').to_ascii_uppercase();
                    color = if line[start..i].ends_with(':') {
                        Color32::from_rgb(210, 170, 98)
                    } else if token.starts_with('R')
                        && token[1..].parse::<u8>().is_ok_and(|r| r < 8)
                    {
                        Color32::from_rgb(90, 190, 218)
                    } else if matches!(
                        token.as_str(),
                        "MOV"
                            | "LOAD"
                            | "STORE"
                            | "PUSH"
                            | "POP"
                            | "ADD"
                            | "SUB"
                            | "MUL"
                            | "DIV"
                            | "INC"
                            | "DEC"
                            | "AND"
                            | "OR"
                            | "XOR"
                            | "NOT"
                            | "SHL"
                            | "SHR"
                            | "CMP"
                            | "JMP"
                            | "JZ"
                            | "JNZ"
                            | "JC"
                            | "JNC"
                            | "CALL"
                            | "RET"
                            | "INT"
                            | "IRET"
                            | "NOP"
                            | "HLT"
                    ) {
                        Color32::from_rgb(197, 139, 241)
                    } else if token.starts_with("0X")
                        || token.chars().next().is_some_and(|c| c.is_ascii_digit())
                    {
                        Color32::from_rgb(224, 171, 105)
                    } else if token.starts_with('.') {
                        Color32::from_rgb(105, 190, 160)
                    } else {
                        Color32::from_rgb(205, 211, 221)
                    };
                }
                job.append(
                    &line[start..i],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        background: if highlighted_line == Some(line_number + 1) {
                            Color32::from_rgb(31, 59, 73)
                        } else {
                            Color32::TRANSPARENT
                        },
                        ..Default::default()
                    },
                );
            }
        }
        ui.fonts(|fonts| fonts.layout_job(job))
    }
    fn registers(&self, ui: &mut egui::Ui) {
        Self::card(ui, "Register File", |ui| {
            egui::Grid::new("regs")
                .striped(true)
                .spacing([14.0, 5.0])
                .show(ui, |ui| {
                    ui.label("REG");
                    ui.label("HEX");
                    ui.label("DEC");
                    ui.label("BIN");
                    ui.end_row();
                    for (i, v) in self.cpu.r.iter().enumerate() {
                        let changed = self.cpu.changed_registers.contains(&format!("R{i}"));
                        ui.label(RichText::new(format!("R{i}")).strong().color(if changed {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::WHITE
                        }));
                        ui.label(Self::mono(format!("{v:04X}")));
                        ui.label(Self::mono(format!("{v}")));
                        ui.label(Self::mono(format!("{v:016b}")).size(9.0));
                        ui.end_row();
                    }
                });
            ui.separator();
            egui::Grid::new("special").num_columns(2).show(ui, |ui| {
                for (n, v) in [
                    ("PC", self.cpu.pc),
                    ("SP", self.cpu.sp),
                    ("IR", self.cpu.ir),
                ] {
                    ui.label(RichText::new(n).strong().color(
                        if self.cpu.changed_registers.iter().any(|x| x == n) {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::WHITE
                        },
                    ));
                    ui.label(Self::mono(format!("{v:04X}")));
                    ui.end_row();
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                for (n, v) in [
                    ("Z", self.cpu.flags.z),
                    ("C", self.cpu.flags.c),
                    ("N", self.cpu.flags.n),
                    ("V", self.cpu.flags.v),
                ] {
                    ui.label(
                        RichText::new(format!("{n} {}", if v { "1" } else { "0" }))
                            .monospace()
                            .color(
                                if self.cpu.changed_registers.contains(&"FLAGS".to_string()) {
                                    Color32::LIGHT_GREEN
                                } else if v {
                                    Color32::from_rgb(81, 200, 169)
                                } else {
                                    Color32::GRAY
                                },
                            ),
                    );
                }
            });
        });
    }
    fn memory(&mut self, ui: &mut egui::Ui) {
        Self::card(ui, "Memory · 64 KiB", |ui| {
            ui.horizontal(|ui| {
                ui.label("Base address");
                ui.add(
                    egui::TextEdit::singleline(&mut self.memory_address)
                        .desired_width(76.0)
                        .font(egui::TextStyle::Monospace),
                );
                ui.separator();
                ui.label("Edit byte");
                ui.add(
                    egui::TextEdit::singleline(&mut self.memory_edit_address)
                        .desired_width(54.0)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("addr"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.memory_edit_value)
                        .desired_width(34.0)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("00"),
                );
                if ui.button("Write").clicked() {
                    let addr =
                        u16::from_str_radix(self.memory_edit_address.trim_start_matches("0x"), 16);
                    let value =
                        u8::from_str_radix(self.memory_edit_value.trim_start_matches("0x"), 16);
                    match (addr, value) {
                        (Ok(a), Ok(v)) => {
                            self.cpu.memory[a as usize] = v;
                            self.cpu.changed_memory = vec![a];
                            self.message = format!("RAM[{a:04X}] ← {v:02X}");
                        }
                        _ => {
                            self.message = "Endereço (16 bits) ou byte hexadecimal inválido".into();
                        }
                    }
                }
            });
            let base = u16::from_str_radix(self.memory_address.trim_start_matches("0x"), 16)
                .unwrap_or(ENTRY)
                & !0x0f;
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    egui::Grid::new("memory")
                        .striped(true)
                        .spacing([8.0, 3.0])
                        .show(ui, |ui| {
                            ui.label("ADDR");
                            for i in 0..16 {
                                ui.label(RichText::new(format!("{i:X}")).monospace().size(10.0));
                            }
                            ui.label("ASCII");
                            ui.end_row();
                            for row in 0..12u16 {
                                let addr = base.wrapping_add(row * 16);
                                ui.label(
                                    RichText::new(format!("{addr:04X}"))
                                        .monospace()
                                        .color(Color32::GRAY),
                                );
                                let mut ascii = String::new();
                                for col in 0..16u16 {
                                    let v = self.cpu.memory[addr.wrapping_add(col) as usize];
                                    ui.label(
                                        RichText::new(format!("{v:02X}"))
                                            .monospace()
                                            .size(10.5)
                                            .color(
                                                if self
                                                    .cpu
                                                    .changed_memory
                                                    .contains(&addr.wrapping_add(col))
                                                {
                                                    Color32::LIGHT_GREEN
                                                } else {
                                                    Color32::from_rgb(200, 210, 220)
                                                },
                                            ),
                                    );
                                    ascii.push(if v.is_ascii_graphic() { v as char } else { '.' });
                                }
                                ui.label(
                                    RichText::new(ascii)
                                        .monospace()
                                        .size(10.0)
                                        .color(Color32::GRAY),
                                );
                                ui.end_row();
                            }
                        });
                });
        });
    }
}
impl eframe::App for BladesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            match self.cpu.run_batch(self.speed) {
                Ok(n) => {
                    if n == 0 && !self.cpu.halted {
                        self.running = false;
                        self.message = format!("Breakpoint em {:04X}", self.cpu.pc);
                    }
                    if n > 0 {
                        self.message = format!("Executando · {n} instruções nesta atualização");
                    }
                    if self.cpu.halted {
                        self.running = false;
                        self.message = "HALT · execução concluída".into();
                    }
                }
                Err(e) => {
                    self.running = false;
                    self.error = Some(e.clone());
                    self.message = e;
                }
            }
            ctx.request_repaint();
        }
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("▰ BLADES")
                        .strong()
                        .size(16.0)
                        .color(Color32::from_rgb(56, 190, 220)),
                );
                ui.label(RichText::new("CPU SIMULATOR").strong());
                ui.separator();
                ui.label(RichText::new("BLADE-16").monospace().color(Color32::GRAY));
                ui.separator();
                ui.add(
                    egui::TextEdit::singleline(&mut self.file_path)
                        .desired_width(150.0)
                        .hint_text("unsaved program.asm"),
                );
                if ui.button("Open").clicked() {
                    let mut dialog = rfd::FileDialog::new().add_filter("Assembly", &["asm"]);
                    if let Some(parent) = std::path::Path::new(&self.file_path)
                        .parent()
                        .filter(|p| p.exists())
                    {
                        dialog = dialog.set_directory(parent);
                    }
                    if let Some(path) = dialog.pick_file() {
                        self.file_path = path.display().to_string();
                        match std::fs::read_to_string(&path) {
                            Ok(text) => {
                                self.source = text;
                                self.assembled = false;
                                self.message = format!("Arquivo carregado: {}", self.file_path);
                                self.error = None;
                            }
                            Err(e) => {
                                self.error = Some(e.to_string());
                                self.message = format!("Falha ao abrir arquivo: {e}");
                            }
                        }
                    }
                }
                if ui.button("Save").clicked() {
                    let path = if self.file_path.trim().is_empty() {
                        rfd::FileDialog::new()
                            .add_filter("Assembly", &["asm"])
                            .set_file_name("program.asm")
                            .save_file()
                    } else {
                        Some(std::path::PathBuf::from(&self.file_path))
                    };
                    if let Some(path) = path {
                        self.file_path = path.display().to_string();
                        match std::fs::write(&path, &self.source) {
                            Ok(()) => {
                                self.message = format!("Arquivo salvo: {}", self.file_path);
                                self.error = None;
                            }
                            Err(e) => {
                                self.error = Some(e.to_string());
                                self.message = format!("Falha ao salvar arquivo: {e}");
                            }
                        }
                    }
                }
                if ui.button("Save As").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Assembly", &["asm"])
                        .set_file_name("program.asm")
                        .save_file()
                    {
                        self.file_path = path.display().to_string();
                        match std::fs::write(&path, &self.source) {
                            Ok(()) => {
                                self.message = format!("Arquivo salvo: {}", self.file_path);
                                self.error = None;
                            }
                            Err(e) => {
                                self.error = Some(e.to_string());
                                self.message = format!("Falha ao salvar arquivo: {e}");
                            }
                        }
                    }
                }
                if ui.button("Assemble").clicked() {
                    self.assemble()
                }
                if ui
                    .add_enabled(
                        self.assembled && !self.cpu.halted,
                        egui::Button::new(if self.running { "Ⅱ Pause" } else { "▶ Run" }),
                    )
                    .clicked()
                {
                    self.running = !self.running
                }
                if ui
                    .add_enabled(
                        self.assembled && !self.cpu.halted,
                        egui::Button::new("Step"),
                    )
                    .clicked()
                {
                    self.running = false;
                    match self.cpu.step() {
                        Ok(instruction) => {
                            self.message = format!("Step · {instruction}");
                            self.error = None;
                        }
                        Err(e) => {
                            self.error = Some(e.clone());
                            self.message = e;
                        }
                    }
                }
                if ui.button("Stop").clicked() {
                    self.running = false
                }
                if ui.button("Reset").clicked() {
                    self.cpu.reset();
                    self.running = false;
                    self.assembled = false;
                    self.message = "CPU reiniciada".into();
                }
                ui.separator();
                ui.label("Rate");
                ui.add(
                    egui::Slider::new(&mut self.speed, 1..=500)
                        .suffix(" instr/frame")
                        .show_value(true),
                );
                ui.label("BP");
                ui.add(
                    egui::TextEdit::singleline(&mut self.breakpoint_address)
                        .desired_width(52.0)
                        .font(egui::TextStyle::Monospace),
                );
                if ui.button("Toggle BP").clicked() {
                    if let Ok(addr) =
                        u16::from_str_radix(self.breakpoint_address.trim_start_matches("0x"), 16)
                    {
                        if self.cpu.breakpoints.contains(&addr) {
                            self.cpu.breakpoints.retain(|x| *x != addr);
                        } else {
                            self.cpu.breakpoints.push(addr);
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(if self.cpu.halted {
                            "HALTED"
                        } else if self.running {
                            "RUNNING"
                        } else {
                            "PAUSED"
                        })
                        .monospace()
                        .color(if self.cpu.halted {
                            Color32::LIGHT_RED
                        } else if self.running {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::LIGHT_BLUE
                        }),
                    );
                    ui.label(RichText::new(format!("{} cycles", self.cpu.cycles)).monospace());
                });
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.message).color(if self.error.is_some() {
                    Color32::LIGHT_RED
                } else {
                    Color32::from_rgb(147, 165, 182)
                }));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("FETCH → DECODE → EXECUTE · single-cycle model")
                            .size(10.0)
                            .color(Color32::GRAY),
                    );
                });
            });
        });
        egui::SidePanel::left("editor")
            .resizable(true)
            .default_width(380.0)
            .min_width(260.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Assembly source");
                    ui.label(RichText::new(".asm").monospace().color(Color32::GRAY));
                    if let Some(line) = self.pc_source_line() {
                        ui.label(
                            RichText::new(format!("PC → line {line}"))
                                .monospace()
                                .color(Color32::LIGHT_GREEN),
                        );
                    }
                });
                ui.separator();
                let current_line = self.pc_source_line();
                egui::ScrollArea::both().show(ui, |ui| {
                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        Self::assembly_layout(ui, text, wrap_width, current_line)
                    };
                    ui.add(
                        egui::TextEdit::multiline(&mut self.source)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(30)
                            .layouter(&mut layouter),
                    );
                });
                if let Some(e) = &self.error {
                    ui.separator();
                    ui.label(RichText::new(e).color(Color32::LIGHT_RED));
                }
                ui.separator();
                ui.collapsing("Source breakpoints", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(120.0)
                        .show(ui, |ui| {
                            for (line, address) in &self.line_addresses {
                                let mut enabled = self.cpu.breakpoints.contains(address);
                                let text = self
                                    .source_lines
                                    .get(line.saturating_sub(1))
                                    .map(String::as_str)
                                    .unwrap_or("")
                                    .trim();
                                let current = *address == self.cpu.pc;
                                let label = RichText::new(format!(
                                    "{} L{line:03} · {address:04X}  {text}",
                                    if current { "▶" } else { " " }
                                ))
                                .color(if current {
                                    Color32::LIGHT_GREEN
                                } else {
                                    Color32::WHITE
                                });
                                if ui.checkbox(&mut enabled, label).changed() {
                                    if enabled {
                                        self.cpu.breakpoints.push(*address);
                                    } else {
                                        self.cpu.breakpoints.retain(|a| a != address);
                                    }
                                }
                            }
                        });
                });
                ui.separator();
                ui.label(
                    RichText::new("ISA: MOV · LOAD/STORE · ALU · CMP · JMP/Jcc · CALL/RET · HLT")
                        .size(10.0)
                        .color(Color32::GRAY),
                );
            });
        egui::SidePanel::right("registers")
            .resizable(true)
            .default_width(370.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Processor state");
                ui.separator();
                self.registers(ui);
                ui.add_space(8.0);
                Self::card(ui, "Stack · descending", |ui| {
                    ui.label(
                        RichText::new(format!("SP = {:04X}", self.cpu.sp))
                            .monospace()
                            .color(Color32::LIGHT_BLUE),
                    );
                    egui::Grid::new("stack")
                        .striped(true)
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("ADDR");
                            ui.label("WORD");
                            ui.end_row();
                            for i in 0..6u16 {
                                let a = self.cpu.sp.wrapping_add(i * 2);
                                let value = u16::from_le_bytes([
                                    self.cpu.memory[a as usize],
                                    self.cpu.memory[a.wrapping_add(1) as usize],
                                ]);
                                ui.label(RichText::new(format!("{a:04X}")).monospace());
                                ui.label(
                                    RichText::new(format!("{value:04X}  /  {value}")).monospace(),
                                );
                                ui.end_row();
                            }
                        });
                });
                ui.add_space(8.0);
                Self::card(ui, "Execution unit", |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("CONTROL UNIT");
                            ui.label(
                                RichText::new("Decode · sequencer")
                                    .size(10.0)
                                    .color(Color32::GRAY),
                            );
                        });
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.label("ALU");
                            ui.label(
                                RichText::new("16-bit · Z C N V")
                                    .size(10.0)
                                    .color(Color32::GRAY),
                            );
                        });
                    });
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "IR {:04X}    PC {:04X}    SP {:04X}",
                            self.cpu.ir, self.cpu.pc, self.cpu.sp
                        ))
                        .monospace()
                        .size(11.0),
                    );
                });
                ui.add_space(8.0);
                Self::card(ui, "Flags", |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Z={}  C={}  N={}  V={}",
                            self.cpu.flags.z as u8,
                            self.cpu.flags.c as u8,
                            self.cpu.flags.n as u8,
                            self.cpu.flags.v as u8
                        ))
                        .monospace(),
                    );
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BLADE-16 · execution monitor");
            ui.add_space(6.0);
            ui.columns(2, |cols| {
                Self::card(&mut cols[0], "Instruction state", |ui| {
                    ui.label(
                        RichText::new("PREVIOUS · RETIRED")
                            .size(10.0)
                            .color(Color32::GRAY),
                    );
                    ui.label(
                        RichText::new(if self.cpu.last_instruction == "—" {
                            "—"
                        } else {
                            &self.cpu.last_instruction
                        })
                        .monospace()
                        .color(Color32::from_rgb(147, 165, 182)),
                    );
                    ui.label(
                        RichText::new("CURRENT · PC")
                            .size(10.0)
                            .color(Color32::GRAY),
                    );
                    if let Some((line, text)) = self.source_at(self.cpu.pc) {
                        ui.label(
                            RichText::new(format!(
                                "{self_pc:04X}  L{line:03}  {text}",
                                self_pc = self.cpu.pc
                            ))
                            .monospace()
                            .strong()
                            .color(Color32::from_rgb(96, 211, 232)),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!("{:04X}  —", self.cpu.pc))
                                .monospace()
                                .color(Color32::GRAY),
                        );
                    }
                    ui.label(
                        RichText::new("NEXT · SEQUENTIAL ADDRESS")
                            .size(10.0)
                            .color(Color32::GRAY),
                    );
                    if let Some((line, text, address)) = self.next_source_at(self.cpu.pc) {
                        ui.label(
                            RichText::new(format!("{address:04X}  L{line:03}  {text}"))
                                .monospace()
                                .color(Color32::from_rgb(180, 193, 207)),
                        );
                    } else {
                        ui.label(RichText::new("—").monospace().color(Color32::GRAY));
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("FETCH");
                        ui.label("→");
                        ui.label("DECODE");
                        ui.label("→");
                        ui.label("EXECUTE");
                    });
                    ui.label(
                        RichText::new(format!(
                            "PC = {:04X}    IR = {:04X}    CYCLES = {}",
                            self.cpu.pc, self.cpu.ir, self.cpu.cycles
                        ))
                        .monospace(),
                    );
                });
                Self::card(&mut cols[1], "Bus activity", |ui| {
                    ui.label("DATA BUS  16-bit");
                    ui.label("ADDRESS BUS  16-bit");
                    ui.label(
                        RichText::new(format!(
                            "LAST WRITE  {}",
                            self.cpu
                                .changed_memory
                                .last()
                                .map(|x| format!("{x:04X}"))
                                .unwrap_or_else(|| "—".into())
                        ))
                        .monospace(),
                    );
                });
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Execution trace");
                ui.label(
                    RichText::new("most recent first · 300 entries")
                        .size(10.0)
                        .color(Color32::GRAY),
                );
            });
            egui::ScrollArea::vertical()
                .max_height(330.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for item in self.cpu.trace.iter().rev() {
                        ui.label(RichText::new(item).monospace().size(12.0));
                    }
                });
            ui.add_space(8.0);
            if self.show_memory {
                self.memory(ui)
            }
        });
    }
}
