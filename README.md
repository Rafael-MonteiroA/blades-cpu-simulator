# Blades CPU Simulator

Aplicação desktop para montar, executar e depurar programas Assembly em uma CPU virtual própria de 16 bits. O núcleo de simulação é Rust independente da interface; a aplicação desktop usa egui/eframe.

## BLADE-16 em resumo

- 8 registradores gerais de 16 bits: R0–R7.
- PC, SP, IR e flags Z/C/N/V.
- 64 KiB de RAM endereçável por byte; words e instruções usam little-endian.
- Execução sequencial FETCH → DECODE → EXECUTE, uma instrução por ciclo.
- Programa carregado por padrão em `0x0200`; stack cresce para endereços menores e começa em `SP = 0xFFFE`.
- Interrupções via tabela de vetores: INT salva flags, registradores e PC; IRET restaura o contexto.

## Funcionalidades

- Editor Assembly com realce de instruções, registradores, labels, diretivas, literais e comentários.
- Destaque visual da linha correspondente ao PC e monitor com instrução anterior, atual e próximo endereço sequencial.
- Abrir arquivos `.asm` por diálogo nativo; salvar por diálogo **Save As** ou pelo caminho digitado.
- Montagem de duas passagens, resolução de labels, comentários com `;`, operandos decimais/hexadecimais e erros com linha.
- Diretivas `.ORG`, `.BYTE`, `.WORD` e `.VECTOR`.
- Run, Pause, Stop, Reset, Step e velocidade configurável. A execução contínua é limitada por quadro para manter a janela responsiva.
- Breakpoints por endereço e por linha Assembly.
- Painéis de registradores, flags, PC/SP/IR, ALU, barramentos, RAM hexadecimal, edição manual de bytes, stack e trilha de execução com deltas de estado.
- Exemplos para aritmética, calculadora, Fibonacci, fatorial, memória, sub-rotinas e interrupções.

## Conjunto de instruções

| Categoria | Instruções |
|---|---|
| Controle | `NOP`, `HLT` |
| Dados e stack | `MOV`, `LOAD`, `STORE`, `PUSH`, `POP` |
| Aritmética | `ADD`, `SUB`, `MUL`, `DIV`, `INC`, `DEC` |
| Lógica e shifts | `AND`, `OR`, `XOR`, `NOT`, `SHL`, `SHR` |
| Comparação | `CMP` |
| Fluxo | `JMP`, `JZ`, `JNZ`, `JC`, `JNC`, `CALL`, `RET` |
| Interrupções | `INT`, `IRET` |

Os resultados aritméticos são de 16 bits. DIV é unsigned; divisão por zero e opcodes inválidos são reportados sem avançar o ciclo. A documentação detalhada das flags, formatos e efeitos está em [`docs/instruction_set.md`](docs/instruction_set.md).

## Requisitos

- Rust stable e Cargo.
- Windows 11 é o alvo inicial. O projeto também habilita backends X11/Wayland do eframe, mas Linux ainda não foi validado nesta entrega.

## Compilar, testar e executar

No PowerShell, a partir da pasta do projeto:

```powershell
cargo fmt --check
cargo test
cargo build --release
cargo run --release
```

## Fluxo rápido

1. Abra um arquivo `.asm` ou edite o exemplo no painel esquerdo.
2. Clique **Assemble** para validar e carregar o programa em RAM.
3. Use **Step** para avançar uma instrução ou **Run** para executar em lotes.
4. Acompanhe PC, instrução, registradores, flags, stack, alterações de memória e trilha.
5. Configure breakpoints no endereço ou na lista **Source breakpoints**.

Exemplo mínimo:

```asm
MOV R0, 10
MOV R1, 20
ADD R0, R1
HLT
```

Exemplo de handler:

```asm
.VECTOR 1, HANDLER
INT 1
HLT
HANDLER:
MOV R2, 0xCAFE
STORE [0x4000], R2
IRET
```

## Organização

```text
src/                 Aplicação egui, assembler e motor BLADE-16
examples/            Programas Assembly demonstrativos
tests/               Testes de integração dos exemplos
docs/                Arquitetura, ISA e referência Assembly
```

Consulte [`docs/architecture.md`](docs/architecture.md), [`docs/instruction_set.md`](docs/instruction_set.md) e [`docs/assembly_reference.md`](docs/assembly_reference.md) para os detalhes técnicos.

## Escopo atual

Esta é uma versão funcional inicial. O editor ainda não tem gutter numérico sincronizado nem syntax highlighting com parser completo; a sintaxe é destacada por regras léxicas simples. Não há edição direta de words ou registradores pela tabela, configuração persistente, CLI independente ou benchmarks. O painel apresenta pipeline sequencial e não simula paralelismo.
