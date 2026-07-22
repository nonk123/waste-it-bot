#[macro_use]
extern crate log;

use reqwest::{Client, Url};
use teloxide::{
    Bot,
    dispatching::{Dispatcher, UpdateFilterExt as _},
    requests::{Request as _, Requester as _},
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
        ParseMode, Update,
    },
    utils::markdown,
};

const PROMT_API_URL_VAR: &str = "PROMT_API_URL";
const PREVIEW_LENGTH: usize = 80;

type ErrorValue = Box<dyn std::error::Error + Send + Sync + 'static>;
type BotResult<T = ()> = Result<T, ErrorValue>;

#[tokio::main]
async fn main() -> BotResult {
    let _ = dotenvy::dotenv();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    std::env::var(PROMT_API_URL_VAR).expect("PROMT_API_URL to be set");

    let bot = Bot::from_env();

    let handler = Update::filter_inline_query().branch(dptree::endpoint(inline));
    let reqwest = Client::builder().build()?;

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![reqwest])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn translate(client: Client, text: String, target_lang: String) -> BotResult<String> {
    if text.chars().count() < 3 {
        return Ok(String::from("..."));
    }

    let api_url = std::env::var(PROMT_API_URL_VAR).unwrap();
    let api_url = Url::parse_with_params(&api_url, &[("to", target_lang)])?;

    let translation = client.post(api_url).body(text).send().await?;
    let translation = translation.text().await?;

    trace!("trans: {translation}");

    Ok(translation)
}

async fn inline(bot: Bot, q: InlineQuery, client: Client) -> BotResult {
    let target_langs = ["en", "ru"];
    let mut results = Vec::with_capacity(target_langs.len());

    for target_lang in target_langs {
        let translation = translate(client.clone(), q.query.to_string(), target_lang.to_string()).await?;

        let response = format!(
            "{}\n{}",
            markdown::blockquote(&markdown::escape(&q.query)),
            markdown::escape(&translation)
        );

        let trimmed: String = translation.chars().take(PREVIEW_LENGTH).collect();

        results.push(InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                target_lang,
                format!("ProMT → {}", target_lang),
                InputMessageContent::Text(InputMessageContentText::new(response).parse_mode(ParseMode::MarkdownV2)),
            )
            .description(trimmed),
        ));
    }

    let response = bot.answer_inline_query(q.id.clone(), results).send().await;

    if let Err(err) = response {
        error!("Error in inline handler: {err:?}");
    }

    Ok(())
}
