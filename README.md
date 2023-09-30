# Kip-Blog

KipSQL sample application project

### How to run

Initialize table data
```
cargo run --bin init
```

When you need to insert a blog
```
cargo run --bin markd "kip-data" ./post.md
```

Server up!

```
cargo run --bin blog-rs
```