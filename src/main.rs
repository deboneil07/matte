use std::{ fs};

fn parse_heading(line: &str) -> Option<String> {
  if line.starts_with("# ") {
    let text = &line[2..];
    return Some(format!("<h1>{}</h1>", text));
  }

  if line.starts_with("## ") {
    let text = &line[3..];
    return Some(format!("<h2>{}</h2>", text));
  }

  if line.starts_with("### ") {
    let text = &line[4..];
    return Some(format!("<h3>{}</h3>", text));
  }

  None
}

fn parse_link(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
  let remaining: String = chars.clone().collect();

  if let Some(close_brac) = remaining.find("](") {
    if let Some(close_paren) = remaining[close_brac + 2..].find(')') {
      let text = &remaining[..close_brac];
      let url_st = close_brac + 2;
      let url_end = url_st + close_paren;

      let url = &remaining[url_st..url_end];

      let html = format!("<a href=\"{}\">{}</a>", url, text);

      for _ in 0..url_end + 1 {
        chars.next();
      }

      return Some(html);
    }
  }

  None
}

fn parse_bold(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {

  if chars.peek() != Some(&'*') {
    return None;
  }

  chars.next();

  let mut text = String::new();
  while let Some(txt) = chars.next() {
    if txt == '*' && chars.peek() == Some(&'*') {
      chars.next(); 
      return Some(format!("<strong>{}</strong>", text));
    }

    text.push(txt);
  }
  None
}

fn parse_inline(text: &str) -> String {
  let mut result = String::new();
  let mut chars = text.chars().peekable();

  // let mut bold: bool = false;
  let mut italic: bool = false;
  

  while let Some(c) = chars.next() {
    if c == '[' {
      if let Some(html) = parse_link(&mut chars) {
        result.push_str(&html);
        continue;
      }
      result.push(c);
    }
    
    else if c == '*' && chars.peek() == Some(&'*') {
      if let Some(txt) = parse_bold(&mut chars) {
        result.push_str(&txt);
        continue;
      }
      result.push(c);
    }

    else if c == '*' {
      if italic {
        result.push_str("</em>");
      } else {
        result.push_str("<em>");
      }
      italic = !italic
    }
    else {
      result.push(c);
    }
  }

  result
  // result = result.replace("**", "<strong>");
  // result
}

fn parse_paragraph(lines: &[&str]) -> String {
  let text = lines.join("\n");
  let html = parse_inline(&text);

  format!("<p>{}</p>", html)
}

fn parse_document(markdown: &str) -> String {
  let lines: Vec<&str> = markdown.lines().collect();
  let mut html = String::new();
  let mut paragraph_lines: Vec<&str> = Vec::new();

  for line in lines {
    if line.trim().is_empty() {
      if !paragraph_lines.is_empty() {
        html.push_str(&parse_paragraph(&paragraph_lines));
        html.push('\n');

        paragraph_lines.clear();
      }

      continue;
    }

    if let Some(heading) = parse_heading(line) {
      if !paragraph_lines.is_empty() {
        html.push_str(&parse_paragraph(&paragraph_lines));
        html.push('\n');

        paragraph_lines.clear();
      }

      html.push_str(&heading);
      html.push('\n');

      continue;
    }

    paragraph_lines.push(line);
  }

  if !paragraph_lines.is_empty() {
    html.push_str(&parse_paragraph(&paragraph_lines));
    html.push('\n');
  }

  html
}

fn main() {
  let markdown = fs::read_to_string("content/index.md").expect("Failed to read the md");

  // let text = "Visit [GitHub](https://github.com) for my projects.";
  // for line in markdown.lines() {
  //   match parse_heading(line) {
  //     Some(html) => println!("{}", html),
  //     None => println!("Not a heading: {}", line)
  //   }
  // }

  // let lines: Vec<&str> = markdown.lines().collect();
  // let para = parse_paragraph(&lines);

  
  let html = parse_document(&markdown);
  println!("{}", html)
}
