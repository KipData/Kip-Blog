use kip_sql::db::{Database, DatabaseError};

#[tokio::main]
async fn main() -> Result<(), DatabaseError>{
    let kip_sql = Database::with_kipdb("./data").await?;
    let _ = kip_sql.run("CREATE TABLE myposts(\
                                post_id varchar PRIMARY KEY, \
                                post_date DATETIME NOT NULL, \
                                post_title VARCHAR, \
                                post_body VARCHAR\
                             )").await?;

    println!("Initialization successful!");
    Ok(())
}