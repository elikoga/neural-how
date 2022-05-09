use clap::{crate_version, Arg, Command};
use neural_how::Question;

fn parse_args() -> Result<String, String> {
    let mut app = Command::new("how")
        .version(crate_version!())
        .trailing_var_arg(true)
        .arg(Arg::new("question").multiple_values(true));
    let matches = app.clone().get_matches();

    let question = matches.values_of("question").ok_or(app.render_usage())?;
    let question = question.collect::<Vec<&str>>().join(" ");
    Ok(question)
}

fn get_token_env() -> Result<String, String> {
    std::env::var("HOW_TOKEN").unwrap_or("custom-textsynth-token-maybe-forward-but-this-costs-money-for-me-so-try-it-sparingly-03b0100604e5bf9f5b2915175be65386".into())
}

fn get_server_env() -> String {
    std::env::var("HOW_SERVER").unwrap_or("https://how.6xr.de/how".into())
}

async fn main_result() -> Result<String, String> {
    let question = parse_args()?;
    let token = get_token_env()?;
    let completion = match Question::new(question, token).deconstruct() {
        Ok(completion) => {
            let client = reqwest::Client::new();
            completion.complete(client).await?
        }
        Err(question) => {
            // send to server
            let server = get_server_env();
            reqwest::Client::new()
                .post(&server)
                .header("Authorization", format!("Bearer {}", question.token))
                .query(&[("question", question.question)])
                .send()
                .await
                .map_err(|e| {
                    println!("This is erroring {}", e);
                    e.to_string()
                })?
                .error_for_status()
                .map_err(|e| {
                    println!("This b is erroring {}", e);
                    e.to_string()
                })?
                .text()
                .await
                .map_err(|e| e.to_string())?
        }
    };
    Ok(completion)
}

#[tokio::main]
async fn main() {
    // call main_result() and print the result or error
    match main_result().await {
        Ok(completion) => println!("{}", completion),
        Err(e) => eprintln!("{}", e),
    }
}
