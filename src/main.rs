use clap::{Parser, Subcommand}; // Import two traits from the clap crate

mod models; // Declare the models module (tells Rust to look for models.rs file)
mod task_manager; // Declare the task_manager module (tells Rust to look for task_manager.rs file)
mod ai; // Declare the ai module (tells Rust to look for ai.rs file)

use models::Task; // Import the Task struct for creating and managing todo items
// Import task management functions: 
// - load_tasks: reads from file, 
// - save_tasks: writes to file, 
// - get_next_id: generates IDs, 
// - load_tasks_as_json: provides task data for AI context
use task_manager::{load_tasks, save_tasks, get_next_id, load_tasks_as_json};
use ai::ask_ai; // Import the main AI function that routes to either one-shot or chat mode

/// Simple TODO list 
#[derive(Parser, Debug)] // Ask clap to automatically implement the Parser trait for this struct
#[command(version, about = "A tiny CLI todo app")] // The #[command()] attribute is provided by clap to configure the CLI: adds --version flag and sets the description
struct CLI {
    #[command(subcommand)] // Tell clap this field will hold which subcommand the user chose
    command: Commands, // This field stores the subcommand the user chose. Type is 'Commands' (an enum defined below)
}

#[derive(Subcommand, Debug)] // Ask clap to automatically implement the Subcommand trait for this enum
enum Commands { // Each variant represents a different subcommand the user can run
    /// Show the whole todo list
    List,

    /// Add a new task
    Add {
        /// Text of the new task
        #[arg(value_name = "TASK")] // Customize how this argument appears in --help text
        task: String,
    },

    /// Mark a task as completed
    Done {
        /// ID of the task to mark done
        #[arg(value_name = "ID")] // Customize how this argument appears in --help text
        id: u8,
    },

    /// Delete a task
    Remove {
        /// ID of the task to delete
        #[arg(value_name = "ID")] // Customize how this argument appears in --help text
        id: u8,
    },

    /// Ask AI
    Ask {
        /// AI prompt - one question (no conversation history)
        #[arg(value_name = "PROMPT")] // Customize how this argument appears in --help text
        prompt: String,

        /// Start a chat session with AI 
        #[arg(short, long)]  // Allows this flag to be used as either -c (short) or --chat (long)
        chat: bool,
    },
}

#[tokio::main] // Needed so we can use async/await inside main()
async fn main() -> Result<(), Box<dyn std::error::Error>> { // Return any error or () on success
    // Parse the command-line arguments provided by the user and create a CLI instance.
    // Example: User types in terminal: `doit add "Buy milk"`
    //   - `doit` is the program name
    //   - `add` is the subcommand
    //   - `"Buy milk"` is the parameter
    // This line (CLI::parse()) reads all of that, validates it, and stores it in the `cli` variable.
    // This is where clap does all the work automatically for us.
    let cli = CLI::parse();
    // Match on which subcommand the user chose and execute the corresponding action
    match cli.command {
        // User use the 'list' command. Display all tasks
        Commands::List => { 
            // Get the tasks from the file and save them into a vector
            // The 'tasks' variable now holds all our tasks as Vec<Task>
            let tasks =  load_tasks();
            if tasks.is_empty() {
                println!("📝 No tasks yet!"); // Show message if there is no tasks
            } else { // if there are tasks
                println!("🗒️  Todo List:"); 
                for task in tasks { // Loop through each task in the vector
                    // Check if task is completed and set the an emoji
                    // If completed is true, use ✅, otherwise use ⬜
                    let status = if task.completed { "✅" } else { "⬜" };
                    println!("  {} [{}] {}", status, task.id, task.description); // Display: emoji [id] description
                }
            }
        },
        // User use the 'add' command with a parameter. Create a new task
        Commands::Add { task } => {
            // Get the tasks from the file and save them to a mutable vector
            // We use 'mut' (mutable) because we will modify this vector later (by adding a new task)
            let mut tasks = load_tasks(); // Load tasks (mutable because we'll add to it)
            let new_task = Task { // Creata a new Task according to user's parameter
                id: get_next_id(&tasks), // Assign next available ID
                // .clone() creates a copy of 'task' string because we use it again in println! below
                // Without .clone(), 'task' would be moved here and we couldn't use it later
                // .clone() lets us use the same string in two places
                description: task.clone(),
                completed: false, // New tasks start as incomplete
            };
            tasks.push(new_task); // Add the new task to the vector
            save_tasks(&tasks); // Save the updated list to file
            println!("✅  Adding task: {}", task); // Show a successful message
        },
        // User use the 'done' command with a parameter. Mark a task as completed
        Commands::Done { id } => {
            // Get the tasks from the file and save them to a mutable vector
            // We use 'mut' (mutable) because we will modify this vector later (by changing the status)
            let mut tasks = load_tasks(); // Load tasks (mutable because we'll modify one)
            // Search for a task with the matching ID
            // iter_mut() gives mutable references so we can modify the task
            // find() returns Option: Some(task) if found, None if not found
            if let Some(task) = tasks.iter_mut().find(|t| t.id == id) { // With Some(task) we extract the Some value to a task variable to use it in the if block.
                task.completed = true; // Mark as completed
                save_tasks(&tasks);// Save changes to file
                println!("✔️  Marked task #{} as done", id); // Display successful message
            } else {
                println!("❌ Task #{} not found", id); // If no task found show no found message
            }
        },
        // User use the 'remove' command with a parameter. Delete a task
        Commands::Remove { id } => {
            // Get the tasks from the file and save them to a mutable vector
            // We use 'mut' (mutable) because we will modify this vector later (by removing a task)
            let mut tasks = load_tasks(); // Load tasks (mutable because we'll remove one)
            let original_len = tasks.len(); // Remember how many tasks we had
            tasks.retain(|t| t.id != id); // retain() keeps only tasks where the condition is true (id != the one we want to remove)
            // Check if a task was actually removed by comparing lengths. No need to save again the same vector if nothing removed
            if tasks.len() < original_len {
                save_tasks(&tasks); // Save the updated list to file
                println!("🗑️  Removed task #{}", id); // Display successful message
            } else {
                println!("❌ Task #{} not found", id); // If no task found show no found message
            }
        },
        // User use the 'ask' command with a parameter. Ask AI 
        Commands::Ask { prompt, chat } => {
            // Validate that user provided a non-empty question
            if prompt.trim().is_empty() {
                println!("Error: Please provide a question");
                return Ok(()); // Return early without calling AI
            }
            
            // Load current tasks from file as JSON string for AI context
            let tasks_json = load_tasks_as_json();

            // Route to appropriate AI function based on chat flag
            // If chat=true: enters conversational mode with history
            // If chat=false: asks one question and exits
            ask_ai(&prompt, &tasks_json, chat).await?;
        }
    }
    Ok(()) // Program ended successfully
}
