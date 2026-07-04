use parser::parse;

fn main() {
    // コマンドライン引数で SQL が渡されたらそれを、なければデモ用クエリを使う
    // 例: cargo run -- "FROM users SELECT id"
    let args: Vec<String> = std::env::args().skip(1).collect();
    let queries: Vec<String> = if args.is_empty() {
        vec![
            "SELECT id, name FROM users WHERE age >= 20;".to_string(),
            "WITH adults AS (FROM users WHERE age >= 20 SELECT id, name) \
             FROM adults a JOIN orders o ON a.id = o.user_id \
             WHERE o.price > 100 GROUP BY a.name \
             SELECT a.name, count(o.id) AS order_count -- from-first"
                .to_string(),
        ]
    } else {
        args
    };

    for sql in queries {
        println!("SQL: {sql}");
        match parse(&sql) {
            Some(query) => println!("{query:#?}"),
            None => println!("  Parse failed."),
        }
        println!();
    }
}
