pub const BEGIN_MARKER: &str = "# BEGIN Break — 이 블록은 Break가 관리합니다. 직접 고치지 마세요.";
pub const END_MARKER: &str = "# END Break";

pub fn wrap(body: &str) -> String {
    if body.is_empty() {
        return String::new();
    }

    let mut out = String::from(BEGIN_MARKER);
    out.push('\n');
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(END_MARKER);
    out.push('\n');
    out
}

pub fn strip(content: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == BEGIN_MARKER {
            inside = true;
            continue;
        }
        if inside {
            if trimmed == END_MARKER {
                inside = false;
            }
            continue;
        }
        kept.push(line);
    }

    while kept.last().is_some_and(|line| line.trim().is_empty()) {
        kept.pop();
    }

    if kept.is_empty() {
        return String::new();
    }

    let mut out = kept.join("\n");
    out.push('\n');
    out
}

pub fn apply(content: &str, block: &str) -> String {
    let base = strip(content);
    if block.is_empty() {
        return base;
    }
    if base.is_empty() {
        return block.to_string();
    }
    format!("{base}\n{block}")
}

pub fn contains_block(content: &str) -> bool {
    content.lines().any(|line| line.trim() == BEGIN_MARKER)
}
