# KipBlog

KipSQL sample application project

### How to run

Initialize table data
```
cargo run --bin init
```

When you need to insert a blog: markd "title name" ./your_post.md
```
cargo run --bin markd "kip-data" ./post.md
```

Server up!

```
cargo run --bin kip-blog
```