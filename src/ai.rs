use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::config::{gemini_api_key, model};
use crate::errors::MinionError;

const INTERACTIONS_URL: &str = "https://generativelanguage.googleapis.com/v1/interactions";

#[derive(Debug, Serialize)]
struct InteractionRequest<'a> {
    model: &'a str,
    input: &'a str,
    stream: bool,
    store: bool,
}

#[derive(Debug, Deserialize)]
struct InteractionResponse {
    #[serde(default)]
    steps: Vec<InteractionStep>,
}

#[derive(Debug, Deserialize)]
struct InteractionStep {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    content: Vec<InteractionContent>,
}

#[derive(Debug, Deserialize)]
struct InteractionContent {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorEnvelope {
    error: ApiErrorBody,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    message: String,
}

pub fn ask(prompt: &str) -> Result<String, MinionError> {
    if prompt.trim().is_empty() {
        return Err(MinionError::message("Prompt must not be empty."));
    }

    let api_key = gemini_api_key()?;
    let model = model();
    let client = Client::new();

    let response = client
        .post(INTERACTIONS_URL)
        .header("x-goog-api-key", api_key)
        .json(&InteractionRequest {
            model: &model,
            input: prompt,
            stream: false,
            store: false,
        })
        .send()
        .map_err(|err| MinionError::message(format!("Could not reach the Gemini API: {err}")))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        let message = serde_json::from_str::<ApiErrorEnvelope>(&body)
            .map(|error| error.error.message)
            .unwrap_or_else(|_| {
                let trimmed = body.trim();
                if trimmed.is_empty() {
                    status
                        .canonical_reason()
                        .unwrap_or("Unknown Gemini API error")
                        .to_owned()
                } else {
                    trimmed.to_owned()
                }
            });

        return Err(MinionError::message(format!(
            "Gemini API error ({}): {message}",
            status.as_u16()
        )));
    }

    let interaction = response.json::<InteractionResponse>().map_err(|err| {
        MinionError::message(format!("Gemini returned an invalid response: {err}"))
    })?;

    let result: String = interaction
        .steps
        .into_iter()
        .filter(|step| step.kind == "model_output")
        .flat_map(|step| step.content)
        .filter(|content| content.kind == "text")
        .filter_map(|content| content.text)
        .collect();

    if result.trim().is_empty() {
        return Err(MinionError::message("Gemini returned no text."));
    }

    Ok(result)
}
