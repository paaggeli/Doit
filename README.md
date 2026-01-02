# Doit App

A simple, beginner-friendly command-line todo list application written in Rust. Perfect for learning Rust concepts like ownership, borrowing, serialization, and CLI argument parsing.

Doit also includes an AI feature powered by Ollama, which allows you to ask questions about your tasks or chat with an AI model directly from the CLI.

## Features

- ✅ Add new tasks
- 📝 List all tasks
- ✔️ Mark tasks as completed
- 🗑️ Remove tasks
- 💾 Persistent storage (saves to JSON file)
- 🤖 Ask questions about your tasks using AI (via Ollama)

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

### Installing Ollama (for AI features)
To use the AI functionality, you need to have Ollama running locally.

Install [Ollama](https://ollama.com/)

Then pull a model 
```bash
ollama pull llama3.2
```
Make sure Ollama is running on:
```bash
http://localhost:11434
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
doit add "Buy groceries"
doit add "Finish Rust tutorial"
# If you are running the app during development
cargo run -- add "Buy groceries"
cargo run -- add "Finish Rust tutorial"
```

**List all tasks:**
```bash
doit list
# If you are running the app during development
cargo run -- list
```

Output example:
```
🗒️  Todo List:
  ⬜ [1] Buy groceries
  ⬜ [2] Finish Rust tutorial
```

**Mark a task as done:**
```bash
doit done 1
# If you are running the app during development
cargo run -- done 1
```

Output:
```
✔️  Marked task #1 as done
```

**Remove a task:**
```bash
doit remove 2
# If you are running the app during development
cargo run -- remove 2
```

Output:
```
🗑️  Removed task #2
```

**AI Commands**

Ask a one-shot question (no conversation memory):
```bash
doit ask "What should I work on next?"
# If you are running the app during development
cargo run -- ask "What should I work on next?"
```

Start a conversational chat session:
```bash
doit ask "Help me prioritize my tasks" --chat
# or
doit ask "Help me prioritize my tasks" -c
# If you are running the app during development
cargo run -- ask "Help me prioritize my tasks" --chat
# or
cargo run -- ask "Help me prioritize my tasks" -c
```

In chat mode, you can keep asking questions and the AI will remember the previous messages until you exit the session.

Type **exit** or **quit** at any time to leave the chat session.

**Get help:**
```bash
doit --help
# If you are running the app during development
cargo run -- --help
```

**Check version:**
```bash
doit --version
# If you are running the app during development
cargo run -- --version
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
doit/
├── src/
│   ├── main.rs          # Entry point, CLI parsing, command routing
│   ├── models.rs        # All data structures (Task, API request/response types)
│   ├── task_manager.rs  # Task operations (load, save, add, remove, etc.)
│   └── ai.rs            # All AI-related functionality (ask_ai, chat, streaming)
├── Cargo.toml           # Project dependencies
├── tasks.json           # Task storage (created automatically)
├── README.md            # This file
├── LICENSE              # License information
└── CONTRIBUTING.md      # Contribution guidelines
```

## Dependencies

This project uses the following crates:
- **clap** - Command-line argument parsing
- **serde** - Serialization/deserialization framework
- **serde_json** - JSON support for serde
- **reqwest** – HTTP client
- **tokio** – Async runtime
- **futures-util** – Stream handling

## Learning Resources

This project demonstrates several Rust concepts:
- **Ownership and borrowing**
- **Traits**: `Serialize`, `Deserialize`, `Parser`, `Subcommand`
- **Iterators**: `iter()`, `iter_mut()`, `map()`, `max()`, `find()`, `retain()`
- **Error handling with** `Result`, `Option`, `unwrap()`, `unwrap_or()`, **and** `?`
- **Pattern matching with** `match`
- **Async / await with** `Tokio`
- **Streaming HTTP responses**
- **NDJSON parsing**

The code is heavily commented to help beginners understand each concept.

**📖 Detailed Tutorial**: For a comprehensive, step-by-step guide to building this project from scratch, check out my blog series:
- [Building Doit: A Command-Line Todo Application in Rust](https://opensourceodyssey.com/building-doit-a-simple-todo-app-in-rust/)
- [Building Doit Part 2: Adding an Intelligent Assistant (AI) to Your Rust CLI App](https://opensourceodyssey.com/building-doit-part-2-adding-an-intelligent-assistant-ai-to-your-rust-cli-app/)
- [Building Doit Part 3: Restructuring the Project with Rust Modules](https://opensourceodyssey.com/building-doit-part-3-restructuring-the-project-with-rust-modules/)
- [Building Doit Part 4: Cleaning Up the AI Code](https://opensourceodyssey.com/building-doit-part-4-cleaning-up-the-ai-code/)
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

---

**Happy coding! 🦀**
