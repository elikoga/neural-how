use actix_web::{
    error::{ErrorBadRequest, ErrorUnauthorized},
    middleware::Logger,
    post,
    web::{self, Data},
    App, HttpRequest, HttpServer, Responder,
};
use log::info;
use neural_how::Question;
use serde::Deserialize;
use serde_json::{Map, Value};
use simple_error::{SimpleError, SimpleResult};
use std::collections::HashMap;

type TokenMap = HashMap<String, String>;

struct AppData {
    token_map: TokenMap,
    reqwest_client: reqwest::Client,
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> SimpleResult<()> {
    // load token_mappings.json
    let token_mappings = std::fs::read_to_string("token_mappings.json")
        .map_err(|e| SimpleError::new(e.to_string()))?;
    let token_mappings = serde_json::from_str::<Map<String, Value>>(&token_mappings)
        .map_err(|e| SimpleError::new(e.to_string()))?;
    let mut token_mappings_hashmap: TokenMap = HashMap::new();
    for (token, value) in token_mappings.iter() {
        let value = value
            .as_str()
            .ok_or(SimpleError::new("somethings wrong with the token_mappings"))?;
        let value = value.to_string();
        token_mappings_hashmap.insert(token.to_string(), value);
    }
    let data = Data::new(AppData {
        token_map: token_mappings_hashmap,
        reqwest_client: reqwest::Client::new(),
    });

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::clone(&data))
            .service(how)
    })
    .bind(("127.0.0.1", 3030))
    .map_err(|e| SimpleError::new(e.to_string()))?
    .run()
    .await
    .map_err(|e| SimpleError::new(e.to_string()))
}

#[derive(Debug, Deserialize)]
struct HowQuery {
    question: String,
}

#[post("/how")]
async fn how(
    req: HttpRequest,
    data: Data<AppData>,
    query: web::Query<HowQuery>,
) -> actix_web::Result<impl Responder> {
    let auth = req
        .headers()
        .get("Authorization")
        .ok_or(ErrorUnauthorized("No Authorization header"))?;
    // check if token is valid
    let token = data
        .token_map
        .get(
            &auth
                .to_str()
                .map_err(|_| ErrorUnauthorized("Invalid Authorization header"))?["Bearer ".len()..],
        )
        .ok_or(ErrorUnauthorized("Invalid token"))?
        .clone();
    let question = Question::new(query.question.clone(), token.clone());

    info!("Using token: {}", token);

    question
        .deconstruct()
        .map_err(|_| ErrorBadRequest("Somehow, I can't understand the token I've found..."))?
        .complete(data.reqwest_client.clone())
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))
}
