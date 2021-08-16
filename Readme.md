# Simple Web Crawler

## Behavior

When given a domain the crawler will request that page. On the received HTML it will look through the href attribute of all anchor tags. Every href which is parsable as an URL is considered visable.
Every new URL which belongs to the requested domain will be crawled afterwards. Every URL which doesn't belong to the same domain will simply be indexed but not followed up.

## REST Endpoints

**POST /v1/crawl/{domain}**
Indexes links for the given domain.

**GET /v1/crawl/{domain}**
This endpoint is only also registered to the GET method because it is easier to test with the browser and I don't know how third parties would like to use the API.
Associating this route to the POST method would otherwise be more idiomatic.

**GET /v1/url_count/{domain}**
After a domain has been successfully crawled, this route will output the total number of unique URLs.
If a domain has not been crawled yet, this route will simply return 0. An error response would make more sense but since the exact behavior in such a case was not defined I decided to make the implementation simpler.

**GET /v1/urls/{domain}**
After a domain has been successfully crawled, this route will output a list of all unique indexed URLs.
If a domain has not been crawled yet, this route will simply return an empty list. An error response would make more sense but since the exact behavior in such a case was not defined I decided to make the implementation simpler.

### Examples

I tested with the official actix.rs web page. Which would lead to following REST calls:

**GET /v1/crawl/actix.rs**
**GET /v1/url_count/actix.rs**
**GET /v1/urls/actix.rs**

## Docker

Since Rusts statically linked binaries make it very easy to use with Docker, I decided to offer a Dockerfile for it.
The Dockerfile performs a couple of simple steps:
1. Prepare docker container as a build environment
2. Build all dependencies of the binary (this was done to keep the build times to a minimum)
3. Build the binary itself
4. Strip unnecessary symbols from the executable to reduce its size
5. Copy binary and SSL certificates into final Alpine docker image

## Technical Details

To not reinvent the wheel I decided to use the actix framework as the base for the web server.
Because the REST endpoints influence each others behavior, they share the same server state. This state is just a mapping from a crawled domain to all indexed URLs.
Actix builds on top of tokio and therefore utilizes event driven async IO. For improved performance I decided to support fetching multiple URLs in parallel.
Because I didn't want to run into deadlock issues with actual Mutexes and because I didn't want to use the async await syntax of tokio's Mutexes, I decided to use a concurrent HashMap from the chashmap crate.
To actually search through the fetched web pages I used the scraper crate which uses the servo engine under the hood. This was quite nice because I already had some experience with it from a hobby [project](https://github.com/MartinquaXD/letgo) I did.

