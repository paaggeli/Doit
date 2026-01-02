use serde::{Serialize, Deserialize}; // Import two traits from the serde crate

// #[derive(Serialize, Deserialize)] Tell serde to automatically implement these traits for our custom type.
// This allows us to convert Task to JSON (serialize) and JSON to Task (deserialize).
#[derive(Serialize, Deserialize)] 
pub struct Task {
    pub id: u8, // The unique identifier for this task
    pub description: String, // What the task is about
    pub completed: bool, // Whether the task is done or not
}

// Unified request structure for both Ollama's /api/generate and /api/chat endpoints
// Using Option<T> allows us to have fields that may or may not be present depending on which endpoint we're calling
// Serialize trait allows converting this struct to JSON for the API request
#[derive(Serialize)]
pub struct OllamaRequest {
    pub model: String, // The Ollama model to use (e.g., "llama3.2")
    // #[serde(skip_serializing_if = "Option::is_none")] tells serde to completely omit this field from the JSON
    // if it's None. Without this, serde would include "prompt": null in the JSON, which could confuse the API.
    // This field is used ONLY by /api/generate endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>, // The complete prompt including context and question - Used by /api/generate
    // Same skip_serializing_if logic as above - omit from JSON when None
    // This field is used ONLY by /api/chat endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>, // Conversation history (system prompt, user messages, AI responses) - Used by /api/chat
    pub stream: bool, // Whether to stream the response word-by-word (true) or wait for complete response (false)
}

// Implementation block - adds methods (functions) to OllamaRequest struct
// These are "constructor helper methods" that make it easier and safer to create OllamaRequest objects
// They ensure you can't accidentally set both prompt and messages, or forget to set one to None
impl OllamaRequest {
    // Constructor helper for /api/generate endpoint
    // Takes only the fields needed for generate, automatically sets messages to None
    // Returns Self (which means OllamaRequest) - "Self" is shorthand for the struct type
    pub fn generate(model: String, prompt: String, stream: bool) -> Self {
        Self {
            model,
            prompt: Some(prompt), // Wrap prompt in Some() because it's Option<String>
            messages: None, // Generate endpoint doesn't use messages
            stream,
        }
    }

    // Constructor helper for /api/chat endpoint
    // Takes only the fields needed for chat, automatically sets prompt to None
    // Returns Self (which means OllamaRequest)
    pub fn chat(model: String, messages: Vec<Message>, stream: bool) -> Self {
        Self {
            model,
            prompt: None, // Chat endpoint doesn't use prompt
            messages: Some(messages), // Wrap messages in Some() because it's Option<Vec<Message>>
            stream,
        }
    }
}

// Unified response structure for both /api/generate and /api/chat endpoints
// Both endpoints return different JSON structures, but we use one struct to handle both
// Deserialize trait allows converting JSON response back to this struct
#[derive(Deserialize)]
pub struct OllamaResponse {
    // #[serde(default)] tells serde: "if this field is missing in the JSON, use the default value instead of erroring"
    // For String, the default is "" (empty string)
    // This is needed because /api/chat responses DON'T include a "response" field
    // Without #[serde(default)], deserializing chat responses would fail with "missing field 'response'"
    #[serde(default)]
    pub response: String, // The AI's generated text (either a chunk or complete response) - Used by /api/generate
    // #[serde(default)] here means if "message" field is missing, use Message::default()
    // (which creates a Message with empty strings for role and content)
    // This is needed because /api/generate responses DON'T include a "message" field
    // Without #[serde(default)], deserializing generate responses would fail with "missing field 'message'"
    #[serde(default)]
    pub message: Message, // The AI's message containing role and content - Used by /api/chat
    pub done: bool, // Whether this is the final chunk (true = streaming complete)
}

// Implementation block - adds methods to OllamaResponse struct
// This particular method helps us extract content regardless of which endpoint was used
impl OllamaResponse {
    // Helper method that abstracts away which field contains the actual AI-generated text
    // Returns the content as a String
    // &self means this method borrows the struct (doesn't take ownership)
    pub fn content(&self) -> String {
        // Check if the "response" field has content (used by /api/generate)
        if !self.response.is_empty() {
            self.response.clone() // Clone creates a copy of the String
        } else {
            // Otherwise, get content from "message.content" field (used by /api/chat)
            self.message.content.clone()
        }
    }
}

// Represents a single message in the conversation (used in chat mode)
// Clone trait allows us to duplicate messages when needed
// Serialize allows converting Message to JSON (for sending to API)
// Deserialize allows converting JSON to Message (for receiving from API)
// Default trait allows creating an "empty" Message with Message::default()
//   - Default is needed because OllamaResponse has #[serde(default)] on the message field
//   - When deserializing /api/generate responses (which don't have a message field), 
//     serde needs to create a default Message, so Message must implement Default
//   - Message::default() creates: Message { role: "", content: "" }
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Message {
    // #[serde(default)] means if "role" is missing in JSON, use "" (empty string)
    // This makes deserialization more forgiving - won't fail if role field is missing
    #[serde(default)]
    pub role: String, // Message sender: "system" (instructions), "user" (human), or "assistant" (AI)
    // #[serde(default)] means if "content" is missing in JSON, use "" (empty string)
    // This makes deserialization more forgiving - won't fail if content field is missing
    #[serde(default)]
    pub content: String, // The actual message text
}