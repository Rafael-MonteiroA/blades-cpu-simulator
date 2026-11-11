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

- Editor Assembly com gutter numérico, breakpoints clicáveis por linha e realce de instruções, registradores, labels, diretivas, literais e comentários.
- Destaque visual da linha correspondente ao PC e monitor com instrução anterior, atual e próximo endereço sequencial.
- Criar um Assembly novo, abrir arquivos `.asm` por diálogo nativo e salvar por diálogo **Save As** ou pelo caminho digitado; alterações pendentes são identificadas e protegidas antes de substituir a fonte.
- Montagem de duas passagens, resolução de labels, comentários com `;`, operandos decimais/hexadecimais e erros com linha.
- Diretivas `.ORG`, `.BYTE`, `.WORD` e `.VECTOR`.
- Run, Pause, Stop, Reset, Step e velocidade configurável. A execução contínua é limitada por quadro para manter a janela responsiva.
- Breakpoints por endereço e por linha Assembly.
- Painéis de registradores em HEX/DEC/BIN, flags, PC/SP/IR, ALU, barramentos, RAM com modos HEX/DEC/BIN/ASCII, mapa de regiões, edição de bytes, histórico de gravações com valores anterior/novo, stack e trilha com deltas de estado.
- Diagrama ao vivo do datapath BLADE-16 com PC, RAM, IR, controle, banco de registradores, ALU, flags, stack e barramentos; realça os blocos alterados durante a execução.
- Console com eventos de montagem, execução, breakpoints, alterações de memória e erros; histórico limitado aos 500 eventos mais recentes.
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
- Windows 11 é o alvo principal. O CI compila e testa também em Ubuntu Linux com X11/Wayland habilitados; a execução gráfica Linux ainda requer validação manual em uma sessão desktop.

## Compilar, testar e executar

No PowerShell, a partir da pasta do projeto:

```powershell
cargo fmt --check
cargo test
cargo build --release
cargo run --release
```

O workflow de CI verifica formatação, executa testes e compila os binários em Windows e Linux a cada push e pull request.

## Pacote portátil para Windows

Para gerar um `.zip` executável com o aplicativo, documentação e exemplos:

```powershell
.\scripts\package-windows.ps1
```

O arquivo é criado em `dist/`. O GitHub Actions também publica o pacote como artefato baixável em cada execução de CI bem-sucedida.

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

O editor usa destaque lexical leve, sem parser incremental, autocomplete ou refatoração de labels. Registradores e words são inspecionáveis pelos painéis, mas sua edição manual ainda não está disponível; bytes de RAM podem ser editados. Velocidade e preferências do Inspector são persistidas; ainda não há CLI independente, empacotador de instalador ou benchmarks. A execução é sequencial, sem pipeline paralelo. O suporte em Linux está habilitado no eframe, mas ainda precisa de validação em CI e uso manual.
