use axum::{
    http::StatusCode, routing::{get, Router},
    response::{Html, IntoResponse},
    extract::{State, Path},
};

use tower_http::services::ServeDir;

use std::sync::Arc;

use askama::Template;
use itertools::Itertools;
use kip_sql::db::{Database, DatabaseError};
use kip_sql::storage::kip::KipStorage;

pub(crate) const BANNER: &str = "
 █████   ████  ███               ███████████  ████
░░███   ███░  ░░░               ░░███░░░░░███░░███
 ░███  ███    ████  ████████     ░███    ░███ ░███   ██████   ███████
 ░███████    ░░███ ░░███░░███    ░██████████  ░███  ███░░███ ███░░███
 ░███░░███    ░███  ░███ ░███    ░███░░░░░███ ░███ ░███ ░███░███ ░███
 ░███ ░░███   ░███  ░███ ░███    ░███    ░███ ░███ ░███ ░███░███ ░███
 █████ ░░████ █████ ░███████     ███████████  █████░░██████ ░░███████
░░░░░   ░░░░ ░░░░░  ░███░░░     ░░░░░░░░░░░  ░░░░░  ░░░░░░   ░░░░░███
                    ░███                                     ███ ░███
                    █████                                   ░░██████
                   ░░░░░                                     ░░░░░░  ";

// post template 
// localhost:4000/post/:query_title
#[derive(Template)]
#[template(path = "posts.html")]
struct PostTemplate<'a> {
    post_title: &'a str,
    post_date: &'a str,
    post_body: &'a str,
}

// homepage template 
// localhost:4000/ 
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub index_title: String,
    pub index_links: &'a Vec<String>,
}

// SQL query will return all posts  
// into a Vec<Post>
#[derive(Debug, Clone)]
pub struct Post {
    pub post_title: String,
    pub post_date: String,
    pub post_body: String,
}

// Our custom Askama filter to replace spaces with dashes in the title
mod filters {

    // now in our templates with can add tis filter e.g. {{ post_title|rmdash }}
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
     }
}

// post router uses two extractors 
// Path to extract the query: localhost:4000/post/thispart
// State that holds a Vec<Post> used to render the post that the query matches 
async fn post(Path(query_title): Path<String>, State(state): State<Arc<Database<KipStorage>>>) -> impl IntoResponse {
    let mut template = PostTemplate{post_title: "none", post_date: "none", post_body: "none"};
    let posts = get_posts(&state).await.unwrap();
    // if the user's query matches a post title then render a template
    for post in &posts {
        if query_title == post.post_title {
            template = PostTemplate{
                post_title: &post.post_title,
                post_date: &post.post_date,
                post_body: &post.post_body
            };
            break;
        } else {
            continue
        }
    }

    // 404 if no title found matching the user's query 
    if &template.post_title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response()
    }
}

// index router (homepage) will return all blog titles in anchor links 
async fn index(State(state): State<Arc<Database<KipStorage>>>) -> impl IntoResponse {
    let mut plinks: Vec<String> = Vec::new();

    for post in get_posts(&state).await.unwrap() {
        plinks.push(post.post_title);
    }

    let template = IndexTemplate{index_title: String::from("Kip-Blog"), index_links: &plinks};

    match template.render() {
         Ok(html) => Html(html).into_response(),
         Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error {}", err),
            ).into_response(),
    }
}

#[tokio::main]
async fn main() {
    let kip_sql = Database::with_kipdb("./data").await.unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route("/post/:query_title", get(post))
        .with_state(Arc::new(kip_sql))
        .nest_service("/assets", ServeDir::new("assets"));

    println!("{} \nVersion: {}\n", BANNER, env!("CARGO_PKG_VERSION"));
    println!("Listening on port 4000");

    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_posts(kip_sql: &Database<KipStorage>) -> Result<Vec<Post>, DatabaseError> {
    Ok(kip_sql.run("select post_title, post_date, post_body from myposts")
        .await?
        .into_iter()
        .map(|tuple| {
            Post {
                post_title: tuple.values[0].to_string(),
                post_date: tuple.values[1].to_string(),
                post_body: tuple.values[2].to_string(),
            }
        })
        .collect_vec())
}
