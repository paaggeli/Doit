# Doit App

A simple, beginner-friendly command-line todo list application written in Rust. Perfect for learning Rust concepts like ownership, borrowing, serialization, and CLI argument parsing.

## Features

- ✅ Add new tasks
- 📝 List all tasks
- ✔️ Mark tasks as completed
- 🗑️ Remove tasks
- 💾 Persistent storage (saves to JSON file)

## Prerequisites

Before you begin, make sure you have Rust installed on your system.

### Installing Rust

**Linux/macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**
Download and run the installer from [rustup.rs](https://rustup.rs/)

Verify installation:
```bash
rustc --version
cargo --version
```

## Installation

### Option 1: Clone from GitHub

```bash
# Clone the repository
git clone https://github.com/paaggeli/Doit.git

# Navigate to the project directory
cd Doit 

# Build the project
cargo build --release

# The executable will be in target/release/
```

### Option 2: Install directly with Cargo

```bash
cargo install --path .
```

## Usage

### Basic Commands

**Add a new task:**
```bash
todo add "Buy groceries"
todo add "Finish Rust tutorial"
```

**List all tasks:**
```bash
todo list
```

Output example:
```
🗒️  Todo List:
  ⬜ [1] Buy groceries
  ⬜ [2] Finish Rust tutorial
```

**Mark a task as done:**
```bash
todo done 1
```

Output:
```
✔️  Marked task #1 as done
```

**Remove a task:**
```bash
todo remove 2
```

Output:
```
🗑️  Removed task #2
```

**Get help:**
```bash
todo --help
```

**Check version:**
```bash
todo --version
```

## How It Works

The application stores tasks in a `tasks.json` file in the same directory where you run the program. Each task has:
- **ID**: A unique identifier
- **Description**: What the task is about
- **Completed**: Whether it's done or not

Example `tasks.json`:
```json
[
  {
    "id": 1,
    "description": "Buy groceries",
    "completed": false
  },
  {
    "id": 2,
    "description": "Finish Rust tutorial",
    "completed": true
  }
]
```

## Project Structure

```
todo-cli/
├── src/
│   └── main.rs          # Main application code
├── Cargo.toml           # Project dependencies
├── tasks.json           # Task storage (created automatically)
├── README.md            # This file
├── LICENSE              # License information
└── CONTRIBUTING.md      # Contribution guidelines
```

## Dependencies

This project uses the following crates:
- **clap** (v4.0+) - Command-line argument parsing
- **serde** (v1.0+) - Serialization/deserialization framework
- **serde_json** (v1.0+) - JSON support for serde

## Learning Resources

This project demonstrates several Rust concepts:
- **Ownership and Borrowing**: See how references (`&`) are used throughout
- **Pattern Matching**: Check the `match` statement in `main()`
- **Error Handling**: See `Result`, `Option`, `unwrap()`, and `unwrap_or()`
- **Traits**: `Serialize`, `Deserialize`, `Parser`, `Subcommand`
- **Iterators**: `iter()`, `iter_mut()`, `map()`, `max()`, `find()`, `retain()`

The code is heavily commented to help beginners understand each concept.

**📖 Detailed Tutorial**: For a comprehensive guide building this project from scratch, check out my blog post:
[Building Doit: A Command-Line Todo Application in Rust](https://opensourceodyssey.com/building-doit-a-simple-todo-app-in-rust/)
## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

### Ways to Contribute

- 🐛 Report/Fix bugs
- 💡 Suggest new features
- 📝 Improve documentation
- 🖼 Improve UX/UI

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

- GitHub: [@paaggeli](https://github.com/paaggeli)

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- CLI parsing by [clap](https://github.com/clap-rs/clap)
- JSON handling by [serde](https://github.com/serde-rs/serde)
---

**Happy coding! 🦀**
