use tokenizer::tokenize::tokenize;

fn main() {
    // コマンドライン引数で SQL が渡されたらそれを、なければデモ用クエリを使う
    // 例: cargo run -- "SELECT id FROM users"
    let args: Vec<String> = std::env::args().skip(1).collect();
    let queries: Vec<String> = if args.is_empty() {
        vec![
            "SELECT id, name FROM users WHERE age >= 20;".to_string(),
            "FROM users u JOIN orders o ON u.id = o.user_id WHERE o.price > 1.5 SELECT u.name -- from-first"
                .to_string(),
        ]
    } else {
        args
    };

    for sql in queries {
        println!("SQL: {sql}");
        match tokenize(&sql) {
            Some(tokens) => {
                for token in tokens {
                    println!("  {token:?}");
                }
            }
            None => println!("  Tokenize failed."),
        }
        println!();
    }
}
