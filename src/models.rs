use serde::{Serialize, Deserialize}; // Import two traits from the serde crate

// #[derive(Serialize, Deserialize)] Tell serde to automatically implement these traits for our custom type.
// This allows us to convert Task to JSON (serialize) and JSON to Task (deserialize).
#[derive(Serialize, Deserialize)] 
pub struct Task {
    pub id: u8, // The unique identifier for this task
    pub description: String, // What the task is about
    pub completed: bool, // Whether the task is done or not
}

// Request structure for Ollama's /api/generate endpoint
// Serialize trait allows converting this struct to JSON for the API request
#[derive(Serialize)]
pub struct GenerateRequest {
    pub model: String, // The Ollama model to use (e.g., "llama3.2")
    pub prompt: String, // The complete prompt including context and question
    pub stream: bool, // Whether to stream the response word-by-word (true) or wait for complete response (false)
}

// Response structure for /api/generate endpoint
// Deserialize trait allows converting JSON response back to this struct
#[derive(Deserialize)]
pub struct GenerateResponse {
    pub response: String, // The AI's generated text (either a chunk or complete response)
    pub done: bool, // Whether this is the final chunk (true = streaming complete)
}

// Request structure for Ollama's /api/chat endpoint (conversational mode with history)
// Serialize trait allows converting this struct to JSON for the API request
#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String, // The Ollama model to use (e.g., "llama3.2")
    pub messages: Vec<Message>, // Conversation history (system prompt, user messages, AI responses)
    pub stream: bool, // Whether to stream the response word-by-word
}

// Represents a single message in the conversation
// Clone trait allows us to duplicate messages when needed
// Serialize and Deserialize allow conversion to/from JSON
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String, // Message sender: "system" (instructions), "user" (human), or "assistant" (AI)
    pub content: String, // The actual message text
}

// Response structure for /api/chat
// Deserialize trait allows converting JSON response back to this struct
#[derive(Deserialize)]
pub struct ChatResponse {
    pub message: Message, // The AI's message containing role and content
    pub done: bool, // Whether this is the final chunk (true = streaming complete)
}