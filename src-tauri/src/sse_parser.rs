// SSE stream parser — split event:/data: pairs, parse chunks, detect tool progress
// New module extracted from hermes.rs (originally src/main/sse-parser.ts, 129 lines)

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub cost: Option<f64>,
    pub rate_limit_remaining: Option<u64>,
    pub rate_limit_reset: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SseDataResult {
    pub content_delta: Option<String>,
    pub tool_progress: Option<String>,
    pub usage: Option<ParsedUsage>,
    pub error_message: Option<String>,
    pub is_done: bool,
    pub session_id: Option<String>,
}

/// Split a raw SSE buffer into blocks (separated by double-newline).
pub fn split_sse_blocks(buffer: &str) -> (Vec<String>, String) {
    let mut blocks = Vec::new();
    let mut remaining = String::new();
    if buffer.is_empty() { return (blocks, remaining); }
    let parts: Vec<&str> = buffer.split("\n\n").collect();
    let ends_with_delim = buffer.ends_with("\n\n");
    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        if is_last && !ends_with_delim {
            remaining = part.to_string();
        } else if !part.is_empty() {
            blocks.push(part.to_string());
        }
    }
    (blocks, remaining)
}

/// Parse a single SSE block into event type and data.
pub fn parse_sse_block(block: &str) -> (String, String) {
    let mut event_type = String::new();
    let mut data = String::new();
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("event: ") {
            event_type = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data: ") {
            data = rest.to_string();
        }
    }
    (event_type, data)
}

/// Process a single SSE data line (OpenAI-format streaming chunk).
pub fn process_sse_data(data: &str, tool_re: Option<&regex_lite::Regex>) -> SseDataResult {
    if data == "[DONE]" {
        return SseDataResult {
            content_delta: None, tool_progress: None, usage: None,
            error_message: None, is_done: true, session_id: None,
        };
    }

    let parsed = match serde_json::from_str::<serde_json::Value>(data) {
        Ok(v) => v,
        Err(_) => return SseDataResult {
            content_delta: None, tool_progress: None, usage: None,
            error_message: None, is_done: false, session_id: None,
        },
    };

    // Error
    if let Some(err) = parsed["error"].as_object() {
        let msg = err["message"].as_str().unwrap_or("Unknown error").to_string();
        return SseDataResult {
            content_delta: None, tool_progress: None, usage: None,
            error_message: Some(msg), is_done: false, session_id: None,
        };
    }

    let mut result = SseDataResult {
        content_delta: None, tool_progress: None, usage: None,
        error_message: None, is_done: false,
        session_id: parsed["session_id"].as_str().map(|s| s.to_string()),
    };

    // Usage
    if let Some(usage) = parsed.get("usage") {
        result.usage = Some(ParsedUsage {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0),
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0),
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0),
            cost: usage["cost"].as_f64(),
            rate_limit_remaining: usage["rate_limit_remaining"].as_u64(),
            rate_limit_reset: usage["rate_limit_reset"].as_u64(),
        });
    }

    // Delta content
    if let Some(choices) = parsed["choices"].as_array() {
        if let Some(choice) = choices.first() {
            if let Some(delta_content) = choice["delta"]["content"].as_str() {
                let content = delta_content.trim();
                // Tool progress detection in content: `emoji tool_name`
                if let Some(ref re) = tool_re {
                    if let Some(caps) = re.captures(content) {
                        result.tool_progress = Some(format!("{} {}", &caps[1], &caps[2]));
                    } else {
                        result.content_delta = Some(delta_content.to_string());
                    }
                } else {
                    result.content_delta = Some(delta_content.to_string());
                }
            }
        }
    }

    result
}

/// Process a custom SSE event (event: hermes.tool.progress).
pub fn process_custom_event(event_type: &str, data: &str) -> Option<String> {
    if event_type != "hermes.tool.progress" { return None; }
    let payload = serde_json::from_str::<serde_json::Value>(data).ok()?;
    let label = payload["label"].as_str().or(payload["tool"].as_str()).unwrap_or("");
    let emoji = payload["emoji"].as_str().unwrap_or("");
    if label.is_empty() { None } else { Some(if emoji.is_empty() { label.to_string() } else { format!("{} {}", emoji, label) }) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_blocks() {
        let (blocks, remaining) = split_sse_blocks("data: hello\n\n");
        assert_eq!(blocks.len(), 1);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_split_blocks_partial() {
        let (blocks, remaining) = split_sse_blocks("data: hello\n\ndata: wor");
        assert_eq!(blocks.len(), 1);
        assert_eq!(remaining, "data: wor");
    }

    #[test]
    fn test_parse_sse_block() {
        let (event, data) = parse_sse_block("event: hermes.tool.progress\ndata: {\"label\":\"search\"}");
        assert_eq!(event, "hermes.tool.progress");
        assert_eq!(data, "{\"label\":\"search\"}");
    }

    #[test]
    fn test_process_sse_data_done() {
        let result = process_sse_data("[DONE]", None);
        assert!(result.is_done);
    }

    #[test]
    fn test_process_sse_data_content() {
        let result = process_sse_data(r#"{"choices":[{"delta":{"content":"Hello"}}]}"#, None);
        assert_eq!(result.content_delta.unwrap(), "Hello");
    }

    #[test]
    fn test_process_sse_data_usage() {
        let result = process_sse_data(r#"{"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}}"#, None);
        assert_eq!(result.usage.unwrap().total_tokens, 15);
    }

    #[test]
    fn test_process_custom_event() {
        let result = process_custom_event("hermes.tool.progress", r#"{"label":"search_web","emoji":"🔍"}"#);
        assert_eq!(result.unwrap(), "🔍 search_web");
    }
}
