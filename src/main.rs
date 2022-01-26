//! Default Compute@Edge template program.

use chrono::DateTime;
use chrono::Timelike;
use chrono::Utc;

use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};

use std::net::IpAddr;
use std::net::Ipv4Addr;

use rand::{thread_rng, Rng};

use svg::node::element::path::Data;
use svg::node::element::Text as TextElement;
use svg::node::element::{Circle, Group, Path, Rectangle};
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

    let size = 120;
    let thickness = 2;
    let padding = thickness * 2;
    let inner_width = size - (padding * 2 + thickness);

    // Pattern match on the path...
    match req.get_path() {
        "/" => Ok(Response::from_status(StatusCode::OK)
            .with_content_type(mime::TEXT_HTML_UTF_8)
            .with_body(include_str!("index.html"))),

        "/great.svg" => {
            let fg = "olivedrab";
            let bg = "cornsilk";

            let dt: DateTime<Utc> = Utc::now();

            let data = Data::new()
                .move_to((thickness / 2, thickness / 2 + padding))
                .elliptical_arc_by((padding, padding, 0, 0, 1, padding, -padding))
                .horizontal_line_by(inner_width)
                .elliptical_arc_by((padding, padding, 0, 0, 1, padding, padding))
                .vertical_line_by(inner_width)
                .elliptical_arc_by((padding, padding, 0, 0, 1, -padding, padding))
                .horizontal_line_by(-inner_width)
                .elliptical_arc_by((padding, padding, 0, 0, 1, -padding, -padding))
                .close();

            let mut group = Group::new()
                .set("fill", fg)
                .set("stroke", bg)
                .set("stroke-width", 0.25);

            for n in 0..(dt.minute()) {
                let x = (n % 8) as i32 * 12;
                let y = (n / 8) as i32 * 12;
                let rect = Rectangle::new()
                    .set("x", x + padding)
                    .set("y", y + padding)
                    .set("rx", thickness)
                    .set("width", thickness * 3)
                    .set("height", thickness * 3);

                group = group.clone().add(rect);
            }

            for n in 0..(dt.second()) {
                let x = (n / 12) as i32 * 8;
                let y = (n % 12) as i32 * 8;
                let rect = Circle::new()
                    .set("cx", x + padding * 2)
                    .set("cy", y + padding * 2)
                    .set("r", thickness);

                group = group.clone().add(rect);
            }

            let path = Path::new()
                .set("fill", bg)
                .set("stroke", fg)
                .set("stroke-width", thickness)
                .set("d", data);

            let document = Document::new()
                .set("width", size * 3)
                .set("height", size * 3)
                .set("viewBox", (0, 0, size, size))
                .add(path)
                .add(group);

            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::IMAGE_SVG)
                .with_body(document.to_string()))
        }

        "/info.svg" => {
            let mut rng = thread_rng();

            let fg = ["olivedrab", "teal", "darkslategray", "maroon"][rng.gen_range(0..4)];
            let bg = ["cornsilk", "bisque", "papayawhip", "palegoldenrod"][rng.gen_range(0..4)];

            let rect = Rectangle::new()
                .set("fill", bg)
                .set("stroke", fg)
                .set("stroke-width", thickness)
                .set("x", thickness / 2)
                .set("y", thickness / 2)
                .set("rx", padding)
                .set("width", size - thickness)
                .set("height", size - thickness);

            let ip: String = req
                .get_client_ip_addr()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)))
                .to_string();

            let text = TextElement::new()
                .set("x", size / 2)
                .set("text-anchor", "middle")
                .set("y", 16 + padding)
                .set("font-family", "GuardianTextSansWeb, Helvetica, sans-serif")
                .set("font-size", 16)
                .set("fill", fg)
                .add(Text::new(["IP: ", &ip].concat()));

            let document = Document::new()
                .set("width", size * 3)
                .set("height", size * 3)
                .set("viewBox", (0, 0, size, size))
                .add(rect)
                .add(text);

            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::IMAGE_SVG)
                .with_body(document.to_string()))
        }

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}
