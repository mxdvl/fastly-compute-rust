//! Default Compute@Edge template program.

use chrono::DateTime;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::Utc;
use fastly::Dictionary;
use serde_json::Value;

use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};

use rand::{thread_rng, Rng};

use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Path, Rectangle};
use svg::{Document, Node};

fn pick_randomly<'a>(items: &'a [&str]) -> &'a str {
    let mut rng = thread_rng();

    items[rng.gen_range(0..items.len())]
}

const UPSTASH_API: &str = "upstash";

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

    let mut rng = thread_rng();

    let size = 120;
    let thickness = 2;
    let padding = thickness * 2;
    let inner_width = size - (padding * 2 + thickness);

    // Pattern match on the path...
    match req.get_path() {
        "/" => Ok(Response::from_status(StatusCode::OK)
            .with_content_type(mime::TEXT_HTML_UTF_8)
            .with_body(include_str!("index.html"))),

        "/clock.svg" => {
            let dt: DateTime<Utc> = match req.get_query_parameter("rand") {
                Some(_) => Utc.ymd(2022, 1, 26).and_hms(
                    rng.gen_range(0..24),
                    rng.gen_range(0..60),
                    rng.gen_range(0..60),
                ),

                None => Utc::now(),
            };

            let am = dt.hour12().0;

            let dark = pick_randomly(&["olivedrab", "teal", "darkslategray", "maroon"]);
            let light = pick_randomly(&["cornsilk", "bisque", "papayawhip", "palegoldenrod"]);

            let fg = if am { dark } else { light };
            let bg = if am { light } else { dark };

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

            let grid = 9;

            let mut hours = Group::new().set("fill", fg).set("stroke", "none").set(
                "transform",
                format!(
                    "translate({},{})",
                    padding * 2 + grid * 6,
                    padding * 2 + grid * 10
                ),
            );

            for n in 0..(dt.hour() % 12) {
                let x = (n % 6) as i32 * grid;
                let y = (n / 6) as i32 * grid;
                let rect = Circle::new()
                    .set("cx", x)
                    .set("cy", y)
                    .set("r", thickness * 2);

                hours.append(rect);
            }

            let mut minutes = Group::new()
                .set("fill", "none")
                .set("stroke", fg)
                .set("stroke-width", thickness)
                .set(
                    "transform",
                    format!("translate({},{})", padding * 2, padding * 2),
                );

            for n in 0..(dt.minute()) {
                let x = (n % 12) as i32 * grid;
                let y = (n / 12) as i32 * grid * 2 + (n % 2) as i32 * grid;
                let rect = Circle::new()
                    .set("cx", x)
                    .set("cy", y)
                    .set("r", thickness * 2);

                minutes.append(rect);
            }

            let mut seconds = Group::new().set("fill", fg).set("stroke", "none").set(
                "transform",
                format!("translate({},{})", padding * 2, padding * 2),
            );

            for n in 0..(dt.second()) {
                let x = (n / 12) as i32 * grid;
                let y = (n % 12) as i32 * grid;
                let rect = Circle::new().set("cx", x).set("cy", y).set("r", thickness);

                seconds.append(rect);
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
                .add(hours)
                .add(minutes)
                .add(seconds);

            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::IMAGE_SVG)
                .with_body(document.to_string()))
        }

        "/presence.svg" => {
            let edge = 9;
            let max = edge * edge;

            let id = rng.gen_range(0..max);

            let dict = Dictionary::open("tokens");
            let token = dict
                .get("UPSTASH_TOKEN")
                .unwrap_or("--missing--".to_string());

            let auth = format!("Bearer {}", token);

            let api_req = Request::post("https://us1-worthy-duckling-35789.upstash.io")
                .with_header("Authorization", &auth)
                .with_body(format!("[\"SET\", \"{key}\", 1, \"EX\", 12]", key = id));
            // let mut beresp =
            api_req.send(UPSTASH_API)?;

            let ids = (0..=81)
                .collect::<Vec<u32>>()
                .into_iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(",");

            let api_req = Request::post("https://us1-worthy-duckling-35789.upstash.io")
                .with_header("Authorization", &auth)
                .with_body(format!("[\"MGET\", {keys}]", keys = ids));
            let beresp = api_req.send(UPSTASH_API)?;

            let data: Value = serde_json::from_str(&beresp.into_body_str())?;

            let dark = pick_randomly(&["olivedrab", "teal", "darkslategray", "maroon"]);
            let light = pick_randomly(&["cornsilk", "bisque", "papayawhip", "palegoldenrod"]);

            let grid = 12;

            let rect = Rectangle::new()
                .set("fill", light)
                .set("stroke", dark)
                .set("stroke-width", thickness)
                .set("x", thickness / 2 - grid)
                .set("y", thickness / 2 - grid)
                .set("rx", grid)
                .set("width", size - thickness)
                .set("height", size - thickness);

            let mut dots = Group::new().set("fill", dark);

            for n in 0..max {
                let present: bool = match data.pointer(&format!("/result/{}", n)) {
                    Some(val) => !val.is_null(),
                    None => false,
                };
                println!("Position {}: {}", n, present);

                if present || n == id {
                    let x = (n / edge) * grid;
                    let y = (n % edge) * grid;
                    let dot = Circle::new().set("cx", x).set("cy", y).set("r", thickness);
                    dots.append(dot)
                }
            }

            let dot = Circle::new()
                .set("fill", "none")
                .set("stroke", dark)
                .set("stroke-width", thickness)
                .set("cx", (id / edge) * grid)
                .set("cy", (id % edge) * grid)
                .set("r", thickness * 3);

            let margin = 20;

            let document = Document::new()
                .set("width", (size + margin) * 3)
                .set("height", (size + margin) * 3)
                .set(
                    "viewBox",
                    (
                        -grid - margin / 2,
                        -grid - margin / 2,
                        size + margin,
                        size + margin,
                    ),
                )
                .add(rect)
                .add(dots)
                .add(dot);

            Ok(Response::from_status(StatusCode::OK)
                .with_content_type(mime::IMAGE_SVG)
                .with_body(document.to_string()))
        }

        // Catch all other requests and return a 404.
        _ => Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n")),
    }
}
