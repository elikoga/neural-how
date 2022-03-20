use serde::Serialize;
use serde_json::{json, Value};

#[derive(PartialEq)]
pub enum CompletionProvider {
    OpenAI { temperature: f32 },
    TextSynth { top_k: i32 },
}

pub struct Completion {
    provider: CompletionProvider,
    engine: String,
    max_tokens: u32,
    stop: String,
    prompt: String,
    token: String,
}

impl Completion {
    pub async fn complete(&self, client: reqwest::Client) -> Result<String, String> {
        let (endpoint, body): (String, Value) = match self.provider {
            CompletionProvider::OpenAI { temperature } => (
                format!(
                    "https://api.openai.com/v1/engines/{}/completion",
                    self.engine
                ),
                json!({
                    "max_tokens": self.max_tokens,
                    "temperature": temperature,
                    "stop": self.stop,
                    "prompt": self.prompt,
                }),
            ),
            CompletionProvider::TextSynth { top_k } => (
                format!(
                    "https://api.textsynth.com/v1/engines/{}/completions",
                    self.engine
                ),
                json!({
                    "max_tokens": self.max_tokens,
                    "top_k": top_k,
                    "stop": self.stop,
                    "prompt": self.prompt,
                }),
            ),
        };
        println!("{:?}", body);
        let response = client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Value>()
            .await
            .map_err(|e| e.to_string())?;
        (|| {
            Some(match self.provider {
                CompletionProvider::OpenAI { .. } => response
                    .get("choices")?
                    .get(0)?
                    .get("text")?
                    .as_str()?
                    .to_string(),
                CompletionProvider::TextSynth { .. } => response.get("text")?.as_str()?.to_string(),
            })
        })()
        .ok_or(response.to_string())
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct Question {
    pub question: String,
    pub token: String,
}

impl Question {
    pub fn new(question: String, token: String) -> Self {
        Question { question, token }
    }
    pub fn deconstruct(&self) -> Result<Completion, Question> {
        let mut parts = self.token.split('-');
        // first part is always the provider
        let provider = match parts.next() {
            Some(provider) => match provider {
                "openai" => CompletionProvider::OpenAI { temperature: 0.0 },
                "textsynth" => CompletionProvider::TextSynth { top_k: 1 },
                _ => return Err(self.clone()),
            },
            None => return Err(self.clone()),
        };
        // second part is always the engine
        let engine = match parts.next() {
            Some(engine) => {
                // if OpenAI, replace _ with -
                if let CompletionProvider::OpenAI { .. } = provider {
                    engine.replace("_", "-")
                } else {
                    engine.into()
                }
            }
            None => return Err(self.clone()),
        };
        let prompt = format!("how {}\nA:\n```bash\n", self.question);
        // collect token from remaining parts
        let token = parts.collect::<Vec<&str>>().join("-");
        Ok(Completion {
            provider,
            engine,
            prompt,
            max_tokens: 256,
            stop: "\n```".into(),
            token,
        })
    }
}

pub enum Error {
    JsonHandleError(serde_json::Error),
}
