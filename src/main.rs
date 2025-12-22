use clap::{Parser, Subcommand}; // Import two traits from the clap crate
use serde::{Serialize, Deserialize}; // Import two traits from the serde crate
use std::fs; // Import the fs module from the standard library for file operations
use std::path::Path; // Import the Path type for working with file paths
use reqwest; // Import reqwest for making HTTP requests to the Ollama API
use futures_util::StreamExt; // Import StreamExt trait to work with async streams (enables .next() method on streams) 
use std::io::{self, Write}; // Import io utilities for reading user input and flushing output to display text immediately

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

// #[derive(Serialize, Deserialize)] Tell serde to automatically implement these traits for our custom type.
// This allows us to convert Task to JSON (serialize) and JSON to Task (deserialize).
#[derive(Serialize, Deserialize)] 
struct Task {
    id: u8, // The unique identifier for this task
    description: String, // What the task is about
    completed: bool, // Whether the task is done or not
}
 
// Request structure for Ollama's /api/generate endpoint
// Serialize trait allows converting this struct to JSON for the API request
#[derive(Serialize)]
struct GenerateRequest {
    model: String, // The Ollama model to use (e.g., "llama3.2")
    prompt: String, // The complete prompt including context and question
    stream: bool, // Whether to stream the response word-by-word (true) or wait for complete response (false)
}

// Response structure for /api/generate endpoint
// Deserialize trait allows converting JSON response back to this struct
#[derive(Deserialize)]
struct GenerateResponse {
    response: String, // The AI's generated text (either a chunk or complete response)
    done: bool, // Whether this is the final chunk (true = streaming complete)
}

// Request structure for Ollama's /api/chat endpoint (conversational mode with history)
// Serialize trait allows converting this struct to JSON for the API request
#[derive(Serialize)]
struct ChatRequest {
    model: String, // The Ollama model to use (e.g., "llama3.2")
    messages: Vec<Message>, // Conversation history (system prompt, user messages, AI responses)
    stream: bool, // Whether to stream the response word-by-word
}

// Represents a single message in the conversation
// Clone trait allows us to duplicate messages when needed
// Serialize and Deserialize allow conversion to/from JSON
#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String, // Message sender: "system" (instructions), "user" (human), or "assistant" (AI)
    content: String, // The actual message text
}

// Response structure for /api/chat
// Deserialize trait allows converting JSON response back to this struct
#[derive(Deserialize)]
struct ChatResponse {
    message: Message, // The AI's message containing role and content
    done: bool, // Whether this is the final chunk (true = streaming complete)
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

// Load tasks as JSON string for AI context
// This function reads the tasks file and returns its content as a JSON string.
// Unlike load_tasks() which deserializes into Vec<Task>, this keeps the data as a string
// because we need to pass it directly to the AI in the prompt.
// Returns: Valid JSON string (either task list or empty array "[]")
fn load_tasks_as_json() -> String {
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

// Main function to handle AI requests
// Routes to either one-shot(one question) mode or chat mode based on the chat flag
// Takes references (&str) because we only need to read the data, not own it
// Returns Result to handle potential errors (network issues, API errors, etc.)
async fn ask_ai(prompt: &str, tasks_json: &str, chat: bool) -> Result<(), Box<dyn std::error::Error>> {
    if chat {
        // User wants conversational mode - use chat endpoint with history
        ask_chat(prompt, tasks_json).await
    } else {
        // User wants one-shot question - use generate endpoint without history
        ask_once(prompt, tasks_json).await
    }
}

// Handle interactive chat mode with conversation history
// Takes the initial question and current tasks as JSON
// Maintains conversation context so AI remembers previous exchanges
// Returns Result to handle errors during the conversation
async fn ask_chat(initial_prompt: &str, tasks_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    // initialize conversation with system prompt and first user question
    // Vec stores the conversation history - each message is kept for context
    let mut messages = vec![
        Message {
            role: "system".to_string(),
            // System message sets the AI's behavior and provides task context
            content: format!("You are a helpful assistant. Here are the user's tasks:\n{}", tasks_json),
        },
        Message {
            role: "user".to_string(),
            // First user question that initiated chat mode
            content: initial_prompt.to_string(),
        },
    ];

    // Send the first message and get AI response
    // The .await waits for the async operation to complete
    // The ? operator propagates errors up if the request fails
    let ai_response = send_chat_message(&messages).await?;

    // Add AI's response to conversation history 
    // This allows AI to reference its previous answers in follow-up questions
    messages.push(ai_response);
    
    // Inform user how to continue or exit the conversation
    println!("\nType your follow-up questions, or 'exit' to quit.\n");

    // Enter interactive loop - continues until user types 'exit' or 'quit'
    loop {
        // Print prompt symbol to indicate we're waiting for input
        print!("> ");
        // Flush so the prompt appears immediately
        io::stdout().flush()?; // Without flush() you can't see what you type (something like typing a password)

        // Create an empty String to store whatever the user types
        let mut user_input = String::new();
        // Put the user's typed input into `user_input`
        io::stdin().read_line(&mut user_input)?;

        // Remove leading/trailing whitespace (including the newline from Enter)
        let user_input = user_input.trim();

        // Check if user wants to exit
        // eq_ignore_ascii_case compares strings case-insensitively ('EXIT' == 'exit')
        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break; // Exit the loop and end chat mode
        }

        // Skip empty input (user just pressed Enter without typing)
        if user_input.is_empty() {
            continue; // Go back to start of loop, show prompt again
        }

        // Add user's message to conversation history
        messages.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        // Send entire conversation history to AI and get response
        // AI sees all previous messages for context
        let ai_response = send_chat_message(&messages).await?;

        // Add AI's response to history so it can reference it later
        messages.push(ai_response);
    }

    Ok(())
}

// Send conversation to AI and stream the response
// Takes a slice of messages (the entire conversation history)
// Returns the complete AI response as a Message for adding to history
// Returns Result to handle network/API errors
async fn send_chat_message(messages: &[Message]) -> Result<Message, Box<dyn std::error::Error>> {
    // Build the request body /api/chat endpoint
    let request_body = ChatRequest {
        model: "llama3.2".to_string(),
        messages: messages.to_vec(), // Copy messages into a Vec for JSON serialization
        stream: true, // Enable token-by-token streaming
    };

    // Create HTTP client for making requests
    let client = reqwest::Client::new();
    // Build and send POST request to Ollama's chat endpoint
    // .json() serializes request_body to JSON and sets Content-Type header
    // .send() actually sends the HTTP request over the network
    // .await waits for the request to complete and response to arrive
    // ? propagates any network errors (connection failed, timeout, etc.)
    let res = client.post("http://localhost:11434/api/chat")
        .json(&request_body)
        .send()
        .await?;

    // Get response as a stream of bytes (chunks arrive as AI generates text)
    let mut stream = res.bytes_stream();
    // Buffer for incomplete JSON lines (chunks may arrive mid-line)
    let mut buffer = String::new();
    // Accumulate complete response text for returning to caller
    let mut full_response = String::new();

    // Ollama streams data in NDJSON (“newline-delimited JSON”).
    // Each complete JSON object is sent as a single line, ending with \n.
    // A chunk from the network may contain:
    // - half a JSON object
    // - 3 JSON objects
    // - 1.5 NDJSON lines
    // - or a newline in the middle of a UTF-8 character
    // Example chunks you might get:
    // CHUNK 1: 
    //       {"message":{"content":"Hel"},"do               <- there is NO new line here 
    // CHUNK 2: 
    //       ne":false}\n                                   <- here we have a new line 
    //       {"message":{"content":"lo wo"},"done":false}\n <- here we have a new line 
    // CHUNK 3: 
    //       {"message":{"content":"r      <-- there is NO new line here 
    // CHUNK 4: 
    //       ld"},"done":false}\n          <-- here we have a new line 
    //       {"done":true}\n               <-- here we have a new line
    // Read the incoming response chunk by chunk as the server sends bytes.
    while let Some(chunk_result) = stream.next().await {
        // Extract the chunk bytes, or return the error if chunk failed to download
        let chunk = chunk_result?;
        // Convert raw bytes to UTF-8 text. `from_utf8_lossy` ensures that even if
        // the stream splits a multibyte character between chunks, invalid byte
        // sequences are replaced safely with �.
        //
        // Important: this does **not** guarantee we now have a whole JSON object.
        let text = String::from_utf8_lossy(&chunk);
        // Add the incoming text fragment to our running buffer.
        // The buffer now may contain:
        // - incomplete data from previous chunks
        // - the new text
        buffer.push_str(&text);

        // Process all complete NDJSON lines in the buffer.
        // NDJSON format guarantees that each JSON object ends with a newline '\n'.
        // So as long as we find a newline in the buffer, we know we have one
        // complete JSON object ready to parse.
        while let Some(newline_pos) = buffer.find('\n') {
            // Extract the substring that represents exactly one NDJSON record.
            // Everything before the newline is one JSON object.
            let json_str = buffer[..newline_pos].to_string();
            // Remove the processed line from the buffer.
            // Any remaining text (after the newline) stays in the buffer.
            // This remaining text may be:
            // - an empty string
            // - partial JSON waiting for the next chunk
            // - multiple future records still waiting for more data
            buffer = buffer[newline_pos + 1..].to_string();

            // Skip empty lines
            if json_str.trim().is_empty() {
                continue;
            }

            // Attempt to deserialize the JSON object into our struct ChatResponse.
            if let Ok(response) = serde_json::from_str::<ChatResponse>(&json_str) {
                // If the field `message.content` contains text, output it.
                // The streaming API sends incremental content tokens, so each
                // NDJSON object usually contains a small piece of text.
                print!("{}", response.message.content);
                
                // Force immediate display instead of waiting for buffer to fill
                io::stdout().flush()?;
                
                // Save the content to build complete response
                full_response.push_str(&response.message.content);

                // Check if this is the last chunk
                if response.done {
                    println!("\n"); // Add final newline after complete response
                    break; // Exit the inner loop
                }
            }
        }
    }

    // Return the complete message for adding to conversation history
    Ok(Message {
        role: "assistant".to_string(),
        content: full_response,
    })
}

// Handle one-shot AI question (no conversation history)
// Takes user's question and current tasks as JSON
// Streams the response word-by-word and exits
// Returns Result to handle network/API errors
async fn ask_once(prompt: &str, tasks_json: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Build the full prompt with task context and user's question
    // The AI sees both the tasks and the question in one prompt 
    let full_prompt = format!(
        "You are a helpful assistant. Here are the user's tasks:\n{}\n\nQuestion: {}", tasks_json, prompt
    );

    // Create the request body for /api/generate endpoint
    let request_body = GenerateRequest {
        model: "llama3.2".to_string(),
        prompt: full_prompt,
        stream: true,
    };

    // Create HTTP client for making requests
    let client = reqwest::Client::new();
    // Build and send POST request to Ollama's generate endpoint
    // .json() serializes request_body to JSON and sets Content-Type header
    // .send() actually sends the HTTP request over the network
    // .await waits for the request to complete and response to arrive
    // ? propagates any network errors (connection failed, timeout, etc.)
    let res = client.post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;

    // Get response as a stream of bytes (chunks arrive as AI generates text)
    let mut stream = res.bytes_stream();
    // Buffer for incomplete JSON lines (chunks may arrive mid-line)
    let mut buffer = String::new();

    // Read the incoming response chunk by chunk as the server sends bytes.
    while let Some(chunk_result) = stream.next().await {
        // Extract the chunk bytes, or return the error if chunk failed to download
        let chunk = chunk_result?;
        // Convert raw bytes to UTF-8 text. `from_utf8_lossy` ensures that even if
        // the stream splits a multibyte character between chunks, invalid byte
        // sequences are replaced safely with �.
        //
        // Important: this does **not** guarantee we now have a whole JSON object.
        let text = String::from_utf8_lossy(&chunk);
        // Add the incoming text fragment to our running buffer.
        // The buffer now may contain:
        // - incomplete data from previous chunks
        // - the new text
        buffer.push_str(&text);
        
        // Process all complete NDJSON lines in the buffer.
        while let Some(newline_pos) = buffer.find('\n') {
            // Extract the substring that represents exactly one NDJSON record.
            // Everything before the newline is one JSON object.
            let json_str = buffer[..newline_pos].to_string();
            // Remove the processed line from the buffer.
            // Any remaining text (after the newline) stays in the buffer.
            buffer = buffer[newline_pos + 1..].to_string();
            
            // Skip empty lines
            if json_str.trim().is_empty() {
                continue;
            }

            // Attempt to deserialize the JSON object into our struct GenerateResponse.
            if let Ok(response) = serde_json::from_str::<GenerateResponse>(&json_str) {
                // If the field `message.content` contains text, output it.
                // The streaming API sends incremental content tokens, so each
                // NDJSON object usually contains a small piece of text.
                print!("{}", response.response);

                // Force immediate display instead of waiting for buffer to fill
                io::stdout().flush()?;
                
                // Check if this is the last chunk (streaming is complete)
                if response.done {
                    println!(); // Final newline
                    break;
                }
            }
        }
    }

    Ok(()) //Everything went fine - no value to return
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
