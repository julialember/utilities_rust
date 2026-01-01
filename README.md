# 🐚 Shu - A Unix Shell Written in Rust

> Because reinventing the wheel is the best way to learn how it spins!


**Shu** is a fully-featured Unix shell implementation in 100% safe Rust. Started as a learning project, it evolved into a production-ready shell with modern features and safety guarantees.

## ✨ Features

### 🔧 Core Shell Features
- **Full pipeline support** - `cmd1 | cmd2 | cmd3` with zero-copy parsing
- **Brace expansion** - `mkdir dir{A,B,C}` creates 3 directories (better than bash!)
- **Quoting & escaping** - `grep "Hello 'World" text.txt` works as expected
- **Background jobs** - `long_task &` runs in background
- **History persistence** - Commands saved between sessions

### 🛡️ Safety First
- **Zero `unsafe` code** - All Rust safety guarantees
- **Protected `rm` command** - Prevents accidental deletion of system directories
- **Memory safe** - No buffer overflows, use-after-free, or segfaults

### 📦 Built-in Commands
| Command    | Description                             |
|------------|-----------------------------------------|
| `cat`      | Concatenate files with Unix options     |
| `grep`     | Search patterns with regex-like matching|
| `ls`       | List files with permissions, ownership  |
| `head-tail`| Combined head/tail utility              |
| `mkdir`    | Create directories with brace expansion |
| `rm`       | Safe removal with protection checks     |
| `cd`, `pwd`, `history` | Standard shell builtins     |

## 🚀 Quick Start

### Installation
```bash
# Clone and build
cd shu
cargo build.
