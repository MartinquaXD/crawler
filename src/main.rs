#![feature(async_closure)]
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use chashmap::CHashMap;
use futures::StreamExt;
use reqwest::Client;
use scraper::{Html, Selector};
use std::convert::AsRef;
use url::Url;

type Error = Box<dyn std::error::Error>;
type MyResult<T> = Result<T, Error>;

#[derive(Debug, Clone, Default)]
struct AppState {
    pub crawled_pages: CHashMap<String, Vec<String>>,
}

async fn get_page(link: &impl AsRef<str>) -> MyResult<Html> {
    let response = Client::builder()
        .build()?
        .get(link.as_ref())
        .send()
        .await?
        .text()
        .await?;

    Ok(Html::parse_document(&response))
}

async fn get_urls_on_page(url: String) -> Vec<Url> {
    match get_page(&url).await {
        Err(err) => {
            println!("{}", err);
            vec![]
        },
        Ok(page) => {
            let selector = Selector::parse("a").expect("Can't parse selector");
            page.select(&selector)
                .filter_map(|element| {
                    element
                        .value()
                        .attr("href")
                        .map(Url::parse)
                        .filter(|parse_result| parse_result.is_ok())
                        .map(|parse_result| parse_result.unwrap())
                })
                .collect()
        }
    }
}

async fn execute_throttled<T>(futures: Vec<impl futures::Future<Output = Vec<T>>>) -> Vec<T> {
    futures::stream::iter(futures)
        //only request at most 1 page per CPU to avoid timeout errors
        .buffer_unordered(num_cpus::get())
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState::default());
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
