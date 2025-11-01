use clap::{Parser, Subcommand}; // Import two traits from the clap crate
use serde::{Serialize, Deserialize}; // Import two traits from the serde crate
use std::fs; // Import the fs module from the standard library for file operations
use std::path::Path; // Import the Path type for working with file paths

// Name of the file where tasks are stored. 
// This is known at compile time, stored in the binary, and lives for the entire program duration.
const TASKS_FILE: &str = "tasks.json";

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
}

// #[derive(Serialize, Deserialize)] Tell serde to automatically implement these traits for our custom type.
// This allows us to convert Task to JSON (serialize) and JSON to Task (deserialize).
#[derive(Serialize, Deserialize)] 
struct Task {
    id: u8, // The unique identifier for this task
    description: String, // What the task is about
    completed: bool, // Whether the task is done or not
}

// Load tasks from the JSON file
fn load_tasks() -> Vec<Task> { // Returns a vector containing Task objects
    if Path::new(TASKS_FILE).exists() { // Path::new(TASKS_FILE) creates a Path object. exists() checks if the file actually exists. 
        let data = fs::read_to_string(TASKS_FILE) // Read file contents into a String. Returns Result<String, Error>
            .unwrap(); // Extract the String from Result (panics if error)
        serde_json::from_str(&data) // Use serde_json's from_str() to deserialize the JSON string into Vec<Task>. Returns Result<Vec<Task>, Error>
            .unwrap_or(Vec::new()) // Extract the vector, or return empty vector if deserialization fails
    } else {
        Vec::new() // Return an empty vector if file doesn't exist
    }
}

// Save tasks to the JSON file
// Takes a reference (&Vec<Task>) instead of taking ownership (Vec<Task>).
// Why use a reference?
// 1. Efficiency - we don't need to move or copy the entire vector into this function
// 2. We only need to READ the tasks to convert them to JSON, not modify or take ownership
// 3. After calling save_tasks(&tasks), the caller can still use 'tasks' because we just borrowed it
fn save_tasks(tasks: &Vec<Task>) {
    let json = serde_json::to_string_pretty(tasks) // Serialize the vector to pretty-formatted JSON string. Returns Result<String, Error>
        .unwrap(); // Extract the String from Result (panics if error)
    // Write the JSON string to file. Returns Result<(), Error>
    // The () or Ok(()) means "unit type" - means the function succeeded but has nothing to return (like void in other languages)
    fs::write(TASKS_FILE, json)
        .unwrap(); // If Ok(()), do nothing and continue. If Err(error), panic
}
// Calculate the next available ID for a new task
// Takes a reference (&Vec<Task>) instead of taking ownership (Vec<Task>).
// Why use a reference?
// 1. Efficiency - we don't need to move or copy the entire vector into this function
// 2. We only need to READ the tasks to find the highest ID, not modify them
// 3. After calling get_next_id(&tasks), the caller can still use 'tasks' because we just borrowed it
// Returns u8 - the next available ID number
fn get_next_id(tasks: &Vec<Task>) -> u8 {
    tasks.iter() // Iterate over tasks
        .map(|t| t.id) // Extract just the IDs (example [1, 3, 5])
        .max() // Find the highest ID. Returns Option<u8>: Some(max_id) or None if empty
        .unwrap_or(0) + 1 // Extract the value from Some, or use 0 if None (no tasks exist) and Add 1 to get the next available ID
}

fn main() {
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
                save_tasks(&tasks); // Save changes to file
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
    }
}