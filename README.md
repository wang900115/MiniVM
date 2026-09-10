# MiniVM

A lightweight Virtual Machine (VM) implemented in Rust for learning how bytecode execution, opcodes, stack, memory, and program execution work.This project is designed as a simple foundation for understanding how blockchain and smart contract virtual machines work internally.
## Run

Make sure Rust and Cargo are installed:

```bash
rustc --version
cargo --version
```

Clone the repository and enter the project directory:

```bash
git clone https://github.com/wang900115/MiniVM.git
cd minivm
```

Run the project:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Format the code:

```bash
cargo fmt
```

## Example

The MiniVM executes bytecode using a stack-based execution model.

Example program:

```text
PUSH 10
PUSH 20
ADD
RETURN
```

Execution:

```text
PUSH 10
Stack: [10]

PUSH 20
Stack: [10, 20]

ADD
Stack: [30]

RETURN
Result: 30
```

Another example:

```text
PUSH 10
PUSH 20
ADD
PUSH 2
MUL
RETURN
```

The result is:

```text
60
```

The VM processes each opcode sequentially and uses the stack and memory to maintain execution state.

## Future Features

Planned improvements include:

- More arithmetic and comparison opcodes
- Conditional and unconditional jumps
- Function calls
- Better memory management
- VM error handling
- Gas / execution cost system
- Bytecode assembler
- Bytecode disassembler
- Persistent contract storage
- Contract state
- Message handling
- More advanced smart contract execution features
- Unit and integration tests
- Improved compatibility with blockchain VM concepts