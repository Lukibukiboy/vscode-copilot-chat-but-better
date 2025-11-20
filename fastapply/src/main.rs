use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tree_sitter::Parser;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/find-component", post(find_component));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 FastApply Engine listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct FindRequest {
    source_code: String,
    target_name: String,
}

#[derive(Serialize)]
struct FindResponse {
    found: bool,
    start_line: usize,
    end_line: usize,
    message: String,
}

async fn find_component(Json(payload): Json<FindRequest>) -> Json<FindResponse> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::language_tsx();
    parser.set_language(language).unwrap();

    let tree = parser.parse(&payload.source_code, None).unwrap();
    let root_node = tree.root_node();

    // Query to find function/variable definitions by name
    let query_str = format!(r#"
        (function_declaration name: (identifier) @name (#eq? @name "{}")) @def
        (variable_declarator name: (identifier) @name (#eq? @name "{}")) @def
    "#, payload.target_name, payload.target_name);

    let query = tree_sitter::Query::new(language, &query_str);

    if let Ok(q) = query {
        let mut cursor = tree_sitter::QueryCursor::new();
        // FIXED: Added 'mut' here because .next() modifies the iterator
        let mut matches = cursor.matches(&q, root_node, payload.source_code.as_bytes());

        // FIXED: Changed .first() to .next()
        if let Some(m) = matches.next() {
            // We look for the capture that is NOT the name (we want the definition)
            // @def is usually the container, so it's larger.
            // We pick the capture with the largest byte range to ensure we get the full function.
            let node = m.captures.iter()
                .max_by_key(|c| c.node.end_byte() - c.node.start_byte())
                .unwrap()
                .node;

            let range = node.start_position();
            let end_range = node.end_position();

            println!("Found {} at lines {}-{}", payload.target_name, range.row, end_range.row);

            return Json(FindResponse {
                found: true,
                start_line: range.row,
                end_line: end_range.row,
                message: "Found".to_string(),
            });
        }
    }

    Json(FindResponse {
        found: false,
        start_line: 0,
        end_line: 0,
        message: "Not Found".to_string(),
    })
}