use tokenizer::tokenize::tokenize;

fn main() {
    // 標準的な書き方と FROM-first の書き方の両方をトークナイズしてみる
    let queries = [
        "SELECT id, name FROM users WHERE age >= 20;",
        "FROM users u JOIN orders o ON u.id = o.user_id WHERE o.price > 1.5 SELECT u.name -- from-first",
    ];

    for sql in queries {
        println!("SQL: {sql}");
        match tokenize(sql) {
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
