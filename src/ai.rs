use reqwest; // Import reqwest for making HTTP requests to the Ollama API
use futures_util::StreamExt; // Import StreamExt trait to work with async streams (enables .next() method on streams) 
use std::io::{self, Write}; // Import io utilities for reading user input and flushing output to display text immediately
// Import our custom data structures from the models module (models.rs file) for API communication and conversation management
use crate::models::{OllamaRequest, OllamaResponse, Message};

// Main function to handle AI requests
// Routes to either one-shot (one question) mode or chat mode based on the chat flag
// Takes references (&str) because we only need to read the data, not own it
// Returns Result to handle potential errors (network issues, API errors, etc.)
pub async fn ask_ai(prompt: &str, tasks_json: &str, chat: bool) -> Result<(), Box<dyn std::error::Error>> {
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
    // Initialize conversation with system prompt and first user question
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
    // Build the request body for /api/chat endpoint using our helper constructor
    // OllamaRequest::chat() ensures prompt is None and messages is Some(messages)
    let request = OllamaRequest::chat(
        "llama3.2".to_string(),
        messages.to_vec(), // Copy messages into a Vec for JSON serialization
        true, // Enable token-by-token streaming
    );

    // Send request to the chat endpoint and get the full response text
    let full_response = send_ollama_request("http://localhost:11434/api/chat", request).await?;

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
        "You are a helpful assistant. Here are the user's tasks:\n{}\n\nQuestion: {}",
        tasks_json, prompt
    );

    // Create the request body for /api/generate endpoint using our helper constructor
    // OllamaRequest::generate() ensures messages is None and prompt is Some(full_prompt)
    let request = OllamaRequest::generate(
        "llama3.2".to_string(),
        full_prompt,
        true, // Enable token-by-token streaming
    );

    // Send request to the generate endpoint (response is printed but not returned)
    send_ollama_request("http://localhost:11434/api/generate", request).await?;

    Ok(()) // Everything went fine - no value to return
}

// Send requests to any Ollama endpoint
// Takes the endpoint URL and the request body
// Handles the HTTP POST, streaming, and response processing for both /api/chat and /api/generate
// Returns the accumulated full response text
// Returns Result to handle network/API errors
async fn send_ollama_request(
    endpoint: &str,           // The Ollama API endpoint URL (e.g., "http://localhost:11434/api/chat")
    request: OllamaRequest,   // The unified request body (works for both chat and generate)
) -> Result<String, Box<dyn std::error::Error>> {
    // Create HTTP client for making requests
    let client = reqwest::Client::new();
    
    // Build and send POST request to the specified Ollama endpoint
    // .json() serializes request body to JSON and sets Content-Type header
    // .send() actually sends the HTTP request over the network
    // .await waits for the request to complete and response to arrive
    // ? propagates any network errors (connection failed, timeout, etc.)
    let response = client.post(endpoint)
        .json(&request)
        .send()
        .await?;

    // Process the response stream and return the accumulated text
    process_stream(response).await
}

// Stream processor for both /api/generate and /api/chat endpoints
// Takes a reqwest Response object (which comes from the Ollama API)
// Extracts the byte stream from it and processes the NDJSON data
// Returns the accumulated full response text
// Returns Result to handle network/parsing errors
async fn process_stream(
    response: reqwest::Response  // The HTTP response from Ollama containing the streaming data
) -> Result<String, Box<dyn std::error::Error>> {
    // Extract the byte stream from the response
    // bytes_stream() converts the HTTP response body into a stream of byte chunks
    let mut stream = response.bytes_stream();
    
    // Buffer for incomplete JSON lines (chunks may arrive mid-line)
    let mut buffer = String::new();
    // Accumulate complete response text for returning to caller
    let mut full_response = String::new();

    // Ollama streams data in NDJSON ("newline-delimited JSON").
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
            // We create a new String here because we need to use it after modifying buffer
            let json_str = buffer[..newline_pos].to_string();
            // Remove the processed line from the buffer using drain() for efficiency
            // drain() modifies the string in-place without allocating a new String
            // Any remaining text (after the newline) stays in the buffer.
            // This remaining text may be:
            // - an empty string
            // - partial JSON waiting for the next chunk
            // - multiple future records still waiting for more data
            buffer.drain(..newline_pos + 1);

            // Skip empty lines
            if json_str.trim().is_empty() {
                continue;
            }

            // Attempt to deserialize the JSON object into our unified OllamaResponse struct.
            // OllamaResponse works for both /api/generate and /api/chat responses
            // because it has #[serde(default)] on both response and message fields
            if let Ok(response) = serde_json::from_str::<OllamaResponse>(&json_str) {
                // Extract content using our helper method that works for both endpoint types
                // For /api/generate: returns response.response
                // For /api/chat: returns response.message.content
                let content = response.content();
                
                // Print the content immediately to the terminal
                // The streaming API sends incremental content tokens, so each
                // NDJSON object usually contains a small piece of text.
                print!("{}", content);
                
                // Force immediate display instead of waiting for buffer to fill
                io::stdout().flush()?;
                
                // Save the content to build complete response for returning
                full_response.push_str(&content);

                // Check if this is the last chunk (streaming is complete)
                if response.done {
                    println!(); // Add final newline after complete response
                    return Ok(full_response); // Return the accumulated response text
                }
            }
        }
    }

    // If we exit the loop without hitting done=true, return what we have
    Ok(full_response)
}