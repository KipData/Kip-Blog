# KipBlog

KipSQL sample application project

### How to run

Initialize table data
```
cargo run --bin init
```

When you need to insert a blog: markd "title name" ./your_post.md
```
cargo run --bin markd "Welcome to KipData" ./post.md
```

Server up!

```
cargo run --bin kip-blog
```

### Features
- ORM Mapping
```rust
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
```