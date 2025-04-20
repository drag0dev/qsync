pub fn unroll_anyhow_result(e: anyhow::Error) -> String {
    let mut res = String::new();
    for (i, small_e) in e.chain().enumerate() {
        res.push_str(&format!("{}{}\n", "\t".repeat(i), small_e));
    }
    res
}

