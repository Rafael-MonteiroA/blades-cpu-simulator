# Blades CPU Simulator

Simulador desktop Rust/egui da arquitetura virtual BLADE-16. O núcleo é independente da interface e executa bytecode determinístico em RAM de 64 KiB.

## Requisitos

- Rust stable e Cargo (verificado nesta máquina: rustc 1.98.1 e cargo 1.98.1).
- Windows 11; eframe também compila para Linux com X11/Wayland.

## Compilar e executar

```powershell
cd outputs\blades-cpu-simulator
cargo run
```

## Uso

Edite o exemplo no painel Assembly, escolha **Assemble**, depois **Step** ou **Run**. O editor destaca instruções, registradores, labels, diretivas, literais e comentários. O monitor mostra registradores, flags, PC/SP/IR, barramento, RAM e trilha de execução. Stop pausa; Reset limpa CPU e RAM. Run processa um lote limitado por quadro para manter a janela responsiva.

## ISA implementada nesta versão

`NOP`, `HLT`; `MOV` imediato/registrador; `LOAD`, `STORE`, `PUSH`, `POP`; `ADD`, `SUB`, `MUL`, `DIV`, `INC`, `DEC`; `AND`, `OR`, `XOR`, `NOT`, `SHL`, `SHR`; `CMP`; `JMP`, `JZ`, `JNZ`, `JC`, `JNC`, `CALL`, `RET`, `INT`, `IRET`.

Assembler inclui labels, comentários com `;`, literais decimais/hexadecimais (`0x` ou sufixo `h`), `.ORG`, `.BYTE`, `.WORD`, `.VECTOR` e erros com linha. Os words são little-endian. Programa inicia em `0x0200`. A pilha cresce para baixo a partir de `0xFFFE`.

## Estado e limitações atuais

Esta versão inclui abrir/salvar por caminho informado, edição de byte da RAM, breakpoints por endereço e linha, stack viewer, interrupções com salvamento/restauração de todo o contexto, examples executáveis e realce lexical de Assembly. Ainda faltam diálogos nativos de arquivo, edição de palavras/registradores na UI, gutter com numeração de linhas sincronizada e configuração persistente. A interface mostra os estágios sequenciais do modelo, sem alegar execução paralela.

Consulte [docs/architecture.md](docs/architecture.md), [docs/instruction_set.md](docs/instruction_set.md) e [docs/assembly_reference.md](docs/assembly_reference.md).
