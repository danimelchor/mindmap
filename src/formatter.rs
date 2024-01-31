use crate::search::SearchResult;

pub fn list(results: &Vec<SearchResult>) -> String {
    let mut fmt = String::new();
    for r in results {
        let start_no = r.start_line_no;
        let end_no = r.end_line_no;

        let content = std::fs::read_to_string(&r.path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let start_no = if start_no > 0 { start_no - 1 } else { 0 };
        let end_no = if end_no < total_lines {
            end_no
        } else {
            total_lines
        };
        let text = lines[start_no..end_no].join("\n");

        fmt.push_str(&format!(
            "{}:{}:{}\n{}\n\n",
            r.path.display(),
            start_no,
            end_no,
            text
        ));
    }
    fmt
}

pub fn raw(results: &Vec<SearchResult>) -> String {
    let mut fmt = String::new();
    for r in results {
        fmt.push_str(&format!(
            "{}:{}:{}\n",
            r.path.display(),
            r.start_line_no,
            r.end_line_no
        ));
    }
    fmt
}
