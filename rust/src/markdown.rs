use owo_colors::OwoColorize;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

pub fn render(markdown: &str) -> String {
    let mut out = String::new();
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut code_block_buf = String::new();
    let mut pending_link: Option<String> = None;

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);

    for ev in Parser::new_ext(markdown, opts) {
        match ev {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let prefix = match level as u8 {
                        1 => "# ",
                        2 => "## ",
                        3 => "### ",
                        4 => "#### ",
                        _ => "",
                    };
                    out.push_str(&prefix.bold().to_string());
                }
                Tag::List(_) => list_depth += 1,
                Tag::Item => {
                    out.push_str(&"- ".bold().to_string());
                }
                Tag::CodeBlock(_kind) => {
                    in_code_block = true;
                    code_block_buf.clear();
                }
                Tag::Link { dest_url, .. } => {
                    pending_link = Some(dest_url.to_string());
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    out.push('\n');
                }
                TagEnd::List(_) => {
                    if list_depth > 0 { list_depth -= 1; }
                    out.push('\n');
                }
                TagEnd::Item => {
                    out.push('\n');
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    // render block in a dimmed style with indentation
                    for line in code_block_buf.lines() {
                        out.push_str(&format!("    {}\n", line.blue()));
                    }
                }
                TagEnd::Link => {
                    if let Some(url) = pending_link.take() {
                        out.push(' ');
                        out.push_str(&format!("({})", url.cyan()));
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                if in_code_block {
                    code_block_buf.push_str(&t);
                } else {
                    out.push_str(&t.to_string());
                }
            }
            Event::Code(code) => {
                out.push_str(&format!("{}", code.yellow()))
            }
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push('\n'),
            Event::Rule => {
                out.push_str(&"—".repeat(20));
                out.push('\n');
            }
            _ => {}
        }
    }
    out.push('\n');
    out
}
