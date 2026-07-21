#[macro_use]
extern crate log;

use reqwest::{Client, Url};
use teloxide::{
    Bot,
    dispatching::{Dispatcher, UpdateFilterExt as _},
    requests::{Request as _, Requester as _},
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText, Update,
    },
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

async fn inline(bot: Bot, q: InlineQuery, client: Client) -> BotResult {
    let translation = if q.query.chars().count() > 2 {
        let target_lang = "en"; // TODO: guess and/or add an option to override

        let api_url = std::env::var(PROMT_API_URL_VAR).unwrap();
        let api_url = Url::parse_with_params(&api_url, &[("to", target_lang)])?;

        let translation = client.post(api_url).body(q.query).send().await?;
        let translation = translation.text().await?;
        trace!("trans: {translation}");
        translation
    } else {
        String::new()
    };

    let trimmed: String = translation.chars().take(PREVIEW_LENGTH).collect();

    let result = InlineQueryResult::Article(
        InlineQueryResultArticle::new(
            "0",
            "ProMT",
            InputMessageContent::Text(InputMessageContentText::new(translation.to_string())),
        )
        .description(trimmed),
    );

    let response = bot.answer_inline_query(q.id.clone(), vec![result]).send().await;

    if let Err(err) = response {
        error!("Error in inline handler: {err:?}");
    }

    Ok(())
}
