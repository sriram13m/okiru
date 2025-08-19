use crate::monitor::AppInfo;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AlignmentEvaluation {
    pub alignment_score: f32,
    pub reasoning: String,
    pub suggestion: Option<String>,
    pub confidence: f32,
}

#[derive(Debug)]
pub enum OllamaError {
    NetworkError(reqwest::Error),
    JsonParseError(serde_json::Error),
    ModelError(String),
    InvalidResponse(String),
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaError::NetworkError(e) => write!(f, "Network error: {}", e),
            OllamaError::JsonParseError(e) => write!(f, "Json Parse error: {}", e),
            OllamaError::ModelError(e) => write!(f, "Model error: {}", e),
            OllamaError::InvalidResponse(e) => write!(f, "Invalid Response: {}", e),
        }
    }
}

impl std::error::Error for OllamaError {}

impl From<reqwest::Error> for OllamaError {
    fn from(err: reqwest::Error) -> Self {
        OllamaError::NetworkError(err)
    }
}

impl From<serde_json::Error> for OllamaError {
    fn from(err: serde_json::Error) -> Self {
        OllamaError::JsonParseError(err)
    }
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url,
            model: model,
        }
    }

    pub async fn evaluate_alignment(
        &self,
        intention: &str,
        current_application: &AppInfo,
        _context: &str,
    ) -> Result<AlignmentEvaluation, OllamaError> {
        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt(intention, current_application);

        let response = self.generate(&user_prompt, Some(&system_prompt)).await?;
        self.parse_alignment_response(&response)
    }

    async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String, OllamaError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: system.map(|s| s.to_string()),
        };

        let url = format!("{}/api/generate", self.base_url);

        let response = self.client.post(&url).json(&request).send().await?;

        let ollama_response: OllamaResponse = response.json().await?;

        if !ollama_response.done {
            return Err(OllamaError::InvalidResponse(
                "Response not complete".to_string(),
            ));
        }

        Ok(ollama_response.response)
    }

    fn build_system_prompt(&self) -> String {
        r#"You are an AI assistant that evaluates how well a user's current activity aligns with their stated intention.

Your job is to analyze the user's intention and their current application activity, then provide:
1. An alignment score from 0.0 to 1.0 (where 1.0 = perfect alignment)
2. Brief reasoning for your score
3. A helpful suggestion (optional)
4. Your confidence in the assessment (0.0 to 1.0)

Respond in this exact JSON format:
{
  "alignment_score": 0.8,
  "reasoning": "User intended to learn Rust and is actively coding in VSCode with Rust files",
  "suggestion": "Great focus! Consider taking a short break in 30 minutes",
  "confidence": 0.9
}

Be encouraging when alignment is good, and gently redirect when it's poor. Consider context like:
- Educational content related to the intention should score high
- Brief breaks or related research are okay
- Entertainment during work intentions should score low
- Tool-switching for the same goal is fine"#.to_string()
    }

    fn build_user_prompt(&self, intention: &str, app: &AppInfo) -> String {
        format!(
            r#"User's Intention: "{}"

Current Activity:
- Application: {}
- Window Title: {}
- Bundle ID: {}

Please evaluate how well this current activity aligns with the user's intention."#,
            intention, app.app_name, app.window_title, app.bundle_id
        )
    }

    fn parse_alignment_response(&self, response: &str) -> Result<AlignmentEvaluation, OllamaError> {
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| OllamaError::InvalidResponse(format!("Invalid JSON: {}", e)))?;

        let alignment_score = parsed["alignment_score"].as_f64().unwrap_or(0.0) as f32;

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let suggestion = parsed["suggestion"].as_str().map(|s| s.to_string());

        let confidence = parsed["confidence"].as_f64().unwrap_or(0.5) as f32;

        Ok(AlignmentEvaluation {
            alignment_score: alignment_score.clamp(0.0, 1.0),
            reasoning,
            suggestion,
            confidence: confidence.clamp(0.0, 1.0),
        })
    }
}
