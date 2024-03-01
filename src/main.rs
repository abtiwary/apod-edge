//! Default Compute template program.

use std::str;

use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};

use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Serialize, Deserialize};

const APOD_HOST_URL: &'static str = "https://api.nasa.gov/";
const APOD_PATH: &'static str = "planetary/apod";
const BACKEND_NAME: &'static str = "apod_host";

/// The entry point for your application.
///
/// This function is triggered when your service receives a client request. It could be used to
/// route based on the request properties (such as method or path), send the request to a backend,
/// make completely new requests, and/or generate synthetic responses.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.

#[derive(Serialize, Deserialize, Debug)]
struct ApodItem {
    date: String,
    explanation: String,
    hdurl: Option<String>,
    media_type: String,
    title: String,
    url: Option<String>,
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Log service version
    println!(
        "FASTLY_SERVICE_VERSION: {}",
        std::env::var("FASTLY_SERVICE_VERSION").unwrap_or_else(|_| String::new())
    );

    // Filter request methods...
    match req.get_method() {
        // Block requests with unexpected methods
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD, PURGE")
                .with_body_text_plain("This method is not allowed\n"))
        }

        // Let any other requests through
        _ => (),
    };

    // Pattern match on the path...
    match req.get_path() {
        // If request is to the `/` path...
        "/" => {
            // Below are some common patterns for Compute services using Rust.
            // Head to https://developer.fastly.com/learning/compute/rust/ to discover more.

            // Create a new request.
            // let mut bereq = Request::get("http://httpbin.org/headers")
            //     .with_header("X-Custom-Header", "Welcome to Compute!")
            //     .with_ttl(60);

            // Add request headers.
            // bereq.set_header(
            //     "X-Another-Custom-Header",
            //     "Recommended reading: https://developer.fastly.com/learning/compute",
            // );

            // Forward the request to a backend.
            // let mut beresp = bereq.send("backend_name")?;

            // Remove response headers.
            // beresp.remove_header("X-Another-Custom-Header");

            // Log to a Fastly endpoint.
            // use std::io::Write;
            // let mut endpoint = fastly::log::Endpoint::from_name("my_endpoint");
            // writeln!(endpoint, "Hello from the edge!").unwrap();
            
            let apod_api_key: &str = req.get_header("x-custom-apod-api-key").unwrap().to_str()?;
            println!("received an APOD API Key = {}", &apod_api_key);

            let fastly_key: &str = req.get_header("Fastly-Key").unwrap().to_str()?;
            println!("received a Fastly AKey = {}", &fastly_key);

            let today = Utc::now();
            let five_days_ago = today - Duration::days(5);
            let formatted_today = format!("{}-{:0>2}-{:0>2}", &today.year(), &today.month(), &today.day());
            let formatted_five_days_ago = format!("{}-{:0>2}-{:0>2}", &five_days_ago.year(), &five_days_ago.month(), &five_days_ago.day());
            println!("{}", formatted_today);
            println!("{}", formatted_five_days_ago);
            
            //let apod_api_request_path = format!("{}&api_key={}&start_date={}&end_date={}", &APOD_PATH, &apod_api_key, &formatted_five_days_ago, &formatted_today);
            //println!("{}", &apod_api_request_path);

            let apod_request_start = Utc::now();
            let mut req = Request::get(APOD_HOST_URL)
                .with_path(&APOD_PATH)
                //.with_path(&apod_api_request_path)
                .with_query_str(format!("api_key={}&start_date={}&end_date={}", &apod_api_key, &formatted_five_days_ago, &formatted_today));
            req.set_header("Fastly-Key", fastly_key);
            req.set_pass(true);

            println!("{:#?}", &req);

            let mut resp = req.send(BACKEND_NAME)?;
            let resp_body = resp.take_body_str();
            let apod_request_end = Utc::now();
            
            println!("APOD responded with: {:?}", &resp_body);
            
            //let response_text = str::from_utf8(&resp_body).unwrap();
            //println!("APOD responded with: {:?}", &response_text);
            //let mut items: Vec<ApodItem> = serde_json::from_str(&response_text)?;
            
            let mut items: Vec<ApodItem> = serde_json::from_str(resp_body.as_str())?;
            items.reverse();

            println!("{:?}", &resp_body);
            println!("{:#?}", &items.get(0));
            
            let mut item: Option<&ApodItem> = None;
            for i in items.iter() {
                if i.media_type != "image" {
                    continue;
                }
                item = Some(i);
                break;
            }
            
            let item = item.unwrap();
            println!("{:?}", &item);

            let final_response = Response::from_status(StatusCode::OK)
                .with_content_type(mime::TEXT_HTML_UTF_8)
                .with_header("apod-date", format!("{:?}", &item.date))
                .with_header("apod-title", format!("{:?}", &item.title))
                .with_header("apod-hdurl", if item.hdurl.is_none() { "N/A".to_owned() } else { format!("{:?}", &item.hdurl.clone().unwrap().to_string()) })
                .with_header("apod-url", if item.url.is_none() { "N/A".to_owned() } else { format!("{:?}", &item.url.clone().unwrap().to_string()) })
                .with_header("total-apod-request-time", format!("{} ns", (apod_request_end - apod_request_start).num_nanoseconds().unwrap()))
                //.with_body(include_str!("welcome-to-compute.html")))
                .with_body(format!("{:?}", &item.explanation));

            Ok(final_response)
        }

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}

