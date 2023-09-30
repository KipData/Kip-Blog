use std::env;
use std::fs::File;
use std::io::Read;
use chrono::Local;
use kip_sql::db::{Database, DatabaseError};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), DatabaseError>{
    let args: Vec<String> = env::args().collect();
    let inserter;

    match File::open(&args[2]) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            inserter = content;
        },
        Err(err) => panic!("file: {} error: {}", args[2], err),
    }

    let kip_sql = Database::with_kipdb("./data").await?;
    let insert_sql = format!(
        "insert into myposts (post_id, post_date, post_title, post_body) values ('{}', '{}', '{}', '{}')",
        Uuid::new_v4(),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        &args[1],
        inserter
    );
    let _ = kip_sql.run(insert_sql.as_str()).await?;

    println!("Inserted successfully!");
    Ok(())
}
