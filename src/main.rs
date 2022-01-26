//! Default Compute@Edge template program.

use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};
use std::net::IpAddr;
use std::net::Ipv4Addr;

use svg::node::element::path::Data;
use svg::node::element::{Path, Text as TextElement};
use svg::node::Text;
use svg::Document;

/// The entry point for your application.
///
/// This function is triggered when your service receives a client request. It could be used to
/// route based on the request properties (such as method or path), send the request to a backend,
/// make completely new requests, and/or generate synthetic responses.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Filter request methods...
    match req.get_method() {
        // Allow GET and HEAD requests.
        &Method::GET | &Method::HEAD => (),

        // Deny anything else.
        _ => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD")
                .with_body_text_plain("This method is not allowed\n"))
        }
    };

    // Pattern match on the path...
    match req.get_path() {
        // If request is to the `/` path...
        "/" => {
            // Below are some common patterns for Compute@Edge services using Rust.
            // Head to https://developer.fastly.com/learning/compute/rust/ to discover more.

            // Create a new request.
            // let mut bereq = Request::get("http://httpbin.org/headers")
            //     .with_header("X-Custom-Header", "Welcome to Compute@Edge!")
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

            // Send a Hello World response.
            Ok(Response::from_status(StatusCode::OK).with_body_text_plain("Hello James"))
        }

        "/great.svg" => {
            let data = Data::new()
                .move_to((10, 10))
                .line_by((0, 50))
                .line_by((50, 0))
                .line_by((0, -50))
                .close();

            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 5)
                .set("d", data);

            let ip: String = req
                .get_client_ip_addr()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)))
                .to_string();

            let text = TextElement::new()
                .set("x", 15)
                .set("y", 25)
                .set("font-size", 9)
                .add(Text::new(ip));

            let document = Document::new()
                .set("viewBox", (0, 0, 70, 70))
                .add(path)
                .add(text);

            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::IMAGE_SVG)
                .with_body(document.to_string()))
        }

        "/bad.svg" => {
            let data = Data::new()
                .move_to((10, 10))
                .line_by((0, 50))
                .line_by((50, 0))
                .line_by((0, -50))
                .close();

            let path = Path::new()
                .set("fill", "red")
                .set("stroke", "none")
                .set("d", data);

            let document = Document::new().set("viewBox", (0, 0, 70, 70)).add(path);

            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::IMAGE_SVG)
                .with_body(document.to_string()))
        }

        "/chill" => Ok(Response::from_status(StatusCode::OK)
            .with_content_type(mime::TEXT_HTML_UTF_8)
            .with_body(include_str!("chill.html"))),

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}
