use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SYSTEM_PROMPT: &str = "You are helping a ministry worker prepare a humble, pastoral first response to a difficult question. Return JSON only with exactly two fields: draft (a short starting response) and verify (an array of 2 to 4 concrete claims, references, or sources to check). Make clear in the draft that it is a starting point for a human, not a final answer. Do not invent citations.";
const ANTHROPIC_MODEL: &str = "claude-sonnet-5";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Draft {
    pub draft: String,
    pub verify: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: String,
    pub asked_at: String,
    pub asker: String,
    pub context: String,
    pub question: String,
    pub tags: Vec<String>,
    pub draft: Option<Draft>,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not locate app data directory: {error}"))?;
    fs::create_dir_all(&data_dir)
        .map_err(|error| format!("Could not create app data directory: {error}"))?;
    Ok(data_dir.join("questions.json"))
}

fn read_questions(app: &AppHandle) -> Result<Vec<Question>, String> {
    let path = store_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents =
        fs::read_to_string(&path).map_err(|error| format!("Could not read questions: {error}"))?;
    let mut questions: Vec<Question> = serde_json::from_str(&contents)
        .map_err(|error| format!("Could not parse questions.json: {error}"))?;
    questions.sort_by(|left, right| right.asked_at.cmp(&left.asked_at));
    Ok(questions)
}

fn write_questions(app: &AppHandle, questions: &[Question]) -> Result<(), String> {
    let path = store_path(app)?;
    let contents = serde_json::to_string_pretty(questions)
        .map_err(|error| format!("Could not serialize questions: {error}"))?;
    fs::write(path, contents).map_err(|error| format!("Could not save questions: {error}"))
}

#[tauri::command]
fn save_question(
    app: AppHandle,
    asker: String,
    context: String,
    question: String,
    tags: Vec<String>,
) -> Result<Question, String> {
    let question_text = question.trim().to_string();
    if question_text.is_empty() {
        return Err("Question cannot be empty.".to_string());
    }

    let asked_at = Utc::now();
    let record = Question {
        id: format!("q_{}", asked_at.format("%Y%m%d_%H%M%S%3f")),
        asked_at: asked_at.to_rfc3339(),
        asker: asker.trim().to_string(),
        context: context.trim().to_string(),
        question: question_text,
        tags: tags
            .into_iter()
            .map(|tag| tag.trim().to_lowercase())
            .filter(|tag| !tag.is_empty())
            .collect(),
        draft: None,
    };

    let mut questions = read_questions(&app)?;
    questions.push(record.clone());
    write_questions(&app, &questions)?;
    Ok(record)
}

#[tauri::command]
fn list_questions(app: AppHandle) -> Result<Vec<Question>, String> {
    read_questions(&app)
}

#[tauri::command]
fn delete_question(app: AppHandle, id: String) -> Result<(), String> {
    let mut questions = read_questions(&app)?;
    let original_len = questions.len();
    questions.retain(|question| question.id != id);
    if questions.len() == original_len {
        return Err("Question was not found.".to_string());
    }
    write_questions(&app, &questions)
}

#[tauri::command]
async fn draft_answer(app: AppHandle, id: String) -> Result<Question, String> {
    let key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY is not set - see README.".to_string())?;
    let questions = read_questions(&app)?;
    let question = questions
        .iter()
        .find(|question| question.id == id)
        .cloned()
        .ok_or_else(|| "Question was not found.".to_string())?;

    let prompt = format!(
        "Question: {}\nAsked by: {}\nContext: {}\nTags: {}",
        question.question,
        question.asker,
        question.context,
        question.tags.join(", ")
    );
    let response = reqwest::Client::new()
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": ANTHROPIC_MODEL,
            "max_tokens": 700,
            "system": SYSTEM_PROMPT,
            "messages": [{ "role": "user", "content": prompt }]
        }))
        .send()
        .await
        .map_err(|error| format!("Could not reach Anthropic: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Could not read Anthropic response: {error}"))?;
    if !status.is_success() {
        return Err(format!("Anthropic returned {status}: {body}"));
    }
    let message: MessageResponse = serde_json::from_str(&body)
        .map_err(|error| format!("Could not parse Anthropic response: {error}"))?;
    let text = message
        .content
        .into_iter()
        .find(|block| block.block_type == "text")
        .and_then(|block| block.text)
        .ok_or_else(|| "Anthropic returned no text content.".to_string())?;
    let draft: Draft = serde_json::from_str(&text)
        .map_err(|error| format!("The AI response was not valid draft JSON: {error}"))?;

    let mut updated_questions = read_questions(&app)?;
    let updated = updated_questions
        .iter_mut()
        .find(|candidate| candidate.id == id)
        .ok_or_else(|| "Question was not found while saving draft.".to_string())?;
    updated.draft = Some(draft);
    let result = updated.clone();
    write_questions(&app, &updated_questions)?;
    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            save_question,
            list_questions,
            delete_question,
            draft_answer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
