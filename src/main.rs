#![feature(async_closure)]
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use chashmap::CHashMap;
use futures::StreamExt;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::convert::AsRef;
use std::time::Instant;
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

async fn crawl(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let start = Instant::now();
    let target_domain = req
        .match_info()
        .get("domain")
        .expect("The route should only match with a domain name.");

    let mut crawled_urls = HashSet::<String>::default();
    let mut urls_to_visit = HashSet::<String>::default();
    urls_to_visit.insert(format!("http://{}", target_domain));

    while !urls_to_visit.is_empty() {
        crawled_urls.extend(urls_to_visit.iter().cloned());
        let get_child_urls: Vec<_> = urls_to_visit.into_iter().map(get_urls_on_page).collect();
        let child_urls = execute_throttled(get_child_urls).await;

        urls_to_visit = HashSet::default();
        for url in child_urls {
            if url.domain() == Some(target_domain) && !crawled_urls.contains(url.as_str()) {
                urls_to_visit.insert(url.into());
            } else {
                crawled_urls.insert(url.into());
            }
        }
    }
    data.crawled_pages
        .insert(target_domain.into(), crawled_urls.into_iter().collect());
    format!(
        "crawled {} in {}ms!",
        &target_domain,
        start.elapsed().as_millis()
    )
}

async fn list_urls(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let domain = req
        .match_info()
        .get("domain")
        .expect("The route should only match with a domain name.")
        .to_string();
    let urls = match data.crawled_pages.get(&domain) {
        Some(read_guard) => read_guard.clone(),
        None => Vec::default(),
    };
    web::Json(urls)
}

async fn count_urls(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let domain = req
        .match_info()
        .get("domain")
        .expect("The route should only match with a domain name.")
        .to_string();
    let count = match data.crawled_pages.get(&domain) {
        Some(read_guard) => read_guard.len(),
        None => 0,
    };
    web::Json(count)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState::default());
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/v1/crawl/{domain}", web::get().to(crawl))
            //I would usually consider /v1/crawl to be a POST route
            //but because I don't know how this code will be tested I also exposed
            //it as a GET route
            .route("/v1/crawl/{domain}", web::post().to(crawl))
            .route("/v1/urls/{domain}", web::get().to(list_urls))
            .route("/v1/url_count/{domain}", web::get().to(count_urls))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
