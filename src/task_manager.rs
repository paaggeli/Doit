use std::fs; // Import the fs module from the standard library for file operations
use std::path::Path; // Import the Path type for working with file paths
use crate::models::Task; // Import the Task struct from our models module (models.rs file) to work with todo items

// Name of the file where tasks are stored. 
// This is known at compile time, stored in the binary, and lives for the entire program duration.
const TASKS_FILE: &str = "tasks.json";

// Load tasks from the JSON file
pub fn load_tasks() -> Vec<Task> { // Returns a vector containing Task objects
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
pub fn save_tasks(tasks: &Vec<Task>) {
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
pub fn get_next_id(tasks: &Vec<Task>) -> u8 {
    tasks.iter() // Iterate over tasks
        .map(|t| t.id) // Extract just the IDs (example [1, 3, 5])
        .max() // Find the highest ID. Returns Option<u8>: Some(max_id) or None if empty
        .unwrap_or(0) + 1 // Extract the value from Some, or use 0 if None (no tasks exist) and Add 1 to get the next available ID
}

// Load tasks as JSON string for AI context
// This function reads the tasks file and returns its content as a JSON string.
// Unlike load_tasks() which deserializes into Vec<Task>, this keeps the data as a string
// because we need to pass it directly to the AI in the prompt.
// Returns: Valid JSON string (either task list or empty array "[]")
pub fn load_tasks_as_json() -> String {
    // Try to read the tasks file
    match fs::read_to_string(TASKS_FILE) {
        // File was read successfully
        Ok(content) => {
            // Validate that the content is valid JSON before using it
            // We use serde_json::Value as a generic JSON type - it can represent any valid JSON
            // The turbofish syntax ::<Type> tells from_str what type to deserialize into
            // from_str returns Result<Value, Error>, and .is_ok() checks if deserialization succeeded
            // If is_ok() is true, the JSON is valid; if false, it's malformed
            if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                // JSON is valid, return the original content
                content
            } else {
                // JSON is malformed - warn the user and return empty array
                eprintln!("Warning: Invalid JSON in tasks file, using empty task list");
                "[]".to_string()
            }
        },
        // File doesn't exist or couldn't be read - return empty array
        // This happens on first run before any tasks are created
        Err(_) => "[]".to_string(),
    }
}