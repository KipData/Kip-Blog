use axum::{
    http::StatusCode, routing::{get, Router},
    response::{Html, IntoResponse},
    extract::State,
};

use tower_http::services::ServeDir;

use std::sync::Arc;

use askama::Template;
use chrono::NaiveDateTime;
use itertools::Itertools;
use kip_sql::db::{Database, DatabaseError};
use kip_sql::implement_from_tuple;
use kip_sql::storage::kip::KipStorage;
use kip_sql::types::value::DataValue;
use kip_sql::types::tuple::Tuple;
use kip_sql::types::LogicalType;

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

struct PostTemplate<'a> {
    post_title: &'a str,
    post_date: String,
    post_body: &'a str,
}

// homepage template 
// localhost:4000/ 
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    posts: Vec<PostTemplate<'a>>,
}

// SQL query will return all posts  
// into a Vec<Post>
#[derive(Debug, Clone, Default)]
pub struct Post {
    pub post_title: String,
    pub post_date: NaiveDateTime,
    pub post_body: String,
}

implement_from_tuple!(Post, (
    post_title: String => |post: &mut Post, value: DataValue| {
        if let Some(title) = value.utf8() {
            post.post_title = title;
        }
    },
    post_date: NaiveDateTime => |post: &mut Post, value: DataValue| {
        if let Some(date_time) = value.datetime() {
            post.post_date = date_time;
        }
    },
    post_body: String => |post: &mut Post, value: DataValue| {
        if let Some(body) = value.utf8() {
            post.post_body = body;
        }
    }
));

// Our custom Askama filter to replace spaces with dashes in the title
mod filters {

    // now in our templates with can add tis filter e.g. {{ post_title|rmdash }}
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
     }
}

// index router (homepage) will return all blog titles in anchor links 
async fn index(State(state): State<Arc<Database<KipStorage>>>) -> impl IntoResponse {
    let posts = get_posts(&state).await.unwrap();

    let template = IndexTemplate{
        posts: posts
            .iter()
            .map(|post| {
                let post_date = post
                    .post_date
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                PostTemplate {
                    post_title: &post.post_title,
                    post_date,
                    post_body: &post.post_body
                }
            })
            .collect_vec()
    };

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
        .map(|tuple| Post::from(tuple))
        .collect_vec())
}
