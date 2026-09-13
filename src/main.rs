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

fn parse_link(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(String, usize)> {
  let remaining: String = chars.clone().collect();

  if let Some(close_brac) = remaining.find("](") {
    if let Some(close_paren) = remaining[close_brac + 2..].find(')') {
      let text = &remaining[..close_brac];
      let url_st = close_brac + 2;
      let url_end = url_st + close_paren;

      let url = &remaining[url_st..url_end];
      let count = url_end + 1;

      let html = format!("<a href=\"{}\">{}</a>", url, text);

      // for _ in 0..url_end + 1 {
      //   chars.next();
      // }

      return Some((html, count));
    }
  }

  None
}

fn parse_bold(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(String, usize)> {

  // if chars.peek() != Some(&'*') {
  //   return None;
  // }

  // chars.next();

  let mut lookahead = chars.clone();
  if lookahead.next() != Some('*') {
    return None;
  }

  
  let mut text = String::new();
  let mut count = 1;
  
  while let Some(txt) = lookahead.next() {
    count += 1;
    if txt == '*' && lookahead.peek() == Some(&'*') {
      lookahead.next(); 
      count += 1;
      
      return Some((format!("<strong>{}</strong>", text), count,));
    }

    text.push(txt);
  }
  None
}

fn parse_code(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(String, usize)> {

  let mut lookahead = chars.clone();
  let mut text: String = String::new();
  let mut count = 0;
  
  while let Some(ct) = lookahead.next() {
    count += 1;
    if ct == '`' {
      return Some((format!("<code>{}</code>", text), count));
    }

    text.push(ct);
  }

  None
}

fn parse_italic(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(String, usize)> {

  // chars.next();

  let mut lookahead = chars.clone();
  // if lookahead.next() != Some('*') {
  //   return None;
  // }
  
  let mut text: String = String::new();
  let mut count: usize = 0;
  
  while let Some(c) = lookahead.next() {
    count+=1;
    if c == '*' {
      return Some((format!("<em>{}</em>", text), count));
    }

    text.push(c);
  }
  None
  
}

fn parse_list_item(line: &str) -> Option<String> {
  if line.starts_with("- ") {
    let text = &line[2..];

    let html = parse_inline(text);

    return Some(format!("<li>{}</li>", html));
  }

  None
}

fn parse_list(lines: &[&str]) -> String {
  let mut html = String::from("<ul>\n");

  for line in lines {
    if let Some(item) = parse_list_item(line) {
      html.push_str(&item);
      html.push('\n');
    }
  }

  html.push_str("</ul>"); 
  html
}

fn parse_ordered_list_item(line: &str) -> Option<String> {
  let mut parts = line.splitn(2, ". ");
  let number = parts.next()?;
  let text = parts.next()?;

  if number.parse::<usize>().is_err() {
    return None;
  }

  let html = parse_inline(text);
  Some(format!("<li>{}</li>", html))
}

fn parse_ordered_list(lines: &[&str]) -> String {
  let mut html = String::from("<ol>\n");

  for line in lines {
    if let Some(text) = parse_ordered_list_item(line) {
      html.push_str(&text);
      html.push('\n');
    }
  }
  html.push_str("</ol>");
  html
}

fn push_text(result: &mut String, text: &str) {
  result.push_str(&escape_html(text));
}

fn flush_paragraph(html: &mut String, paragraph_lines: &mut Vec<&str>) {
  if paragraph_lines.is_empty() {
    return;
  }

  html.push_str(&parse_paragraph(paragraph_lines));
  html.push('\n');

  paragraph_lines.clear();
}

fn flush_unordered_list(html: &mut String, list_lines: &mut Vec<&str>) {
  if list_lines.is_empty() {
    return;
  }

  html.push_str(&parse_list(list_lines));
  html.push('\n');

  list_lines.clear();
}

fn flush_ordered_list(html: &mut String, ordered_list_lines: &mut Vec<&str>) {
  if ordered_list_lines.is_empty() {
    return;
  }

  html.push_str(&parse_ordered_list(ordered_list_lines));
  html.push('\n');

  ordered_list_lines.clear();
}

fn flush_lists(html: &mut String, list_lines: &mut Vec<&str>, ordered_list_lines: &mut Vec<&str>) {
  
  flush_unordered_list(html, list_lines);
  flush_ordered_list(html, ordered_list_lines);
}

fn flush_code_block(html: &mut String, code_lines: &mut Vec<&str>, language: &str) {
  if code_lines.is_empty() {
    return;
  }

  if language.is_empty() {
    html.push_str("<pre><code>");
  } else {
    html.push_str(&format!("<pre><code class=\"language-{}\">", escape_html(language)));
  }
  
  for line in code_lines.iter() {
    html.push_str(&escape_html(line));
    html.push('\n');
  }

  html.push_str("</code></pre>\n");
  code_lines.clear();
}

fn flush_blockquote(result: &mut String, quote_lines: &mut Vec<&str>) {
  if quote_lines.is_empty() {
    return;
  }

  let content: Vec<&str> = quote_lines.iter().copied().filter(|line| !line.is_empty()).collect();

  result.push_str("<blockquote>\n");
  result.push_str("<p>");
  result.push_str(&parse_inline(&content.join("\n")));
  result.push_str("</p>\n");
  result.push_str("</blockquote>\n");
  quote_lines.clear();


  
}

fn escape_html(text: &str) -> String {
  text.replace('&', "&amp;")
  .replace('<', "&lt;")
  .replace('>', "&gt;")
  .replace('"', "&quot;")
  .replace('\'', "&#39;")
}

fn parse_inline(text: &str) -> String {
  let mut result = String::new();
  let mut chars = text.chars().peekable();

  // let mut bold: bool = false;
  // let mut italic: bool = false;
  

  while let Some(c) = chars.next() {
    if c == '!' && chars.peek() == Some(&'['){
      // chars.next();

      if let Some((text, count)) = parse_image(&mut chars) {
        result.push_str(&text);

        for _ in 0..count {
          chars.next();
        }

        continue;
      }

      push_text(&mut result, "!");
    }

    else if c == '[' {
      if let Some((html, consumed)) = parse_link(&mut chars) {
        result.push_str(&html);

        for _ in 0..consumed {
          chars.next();
        }

        continue;
      }
      push_text(&mut result, &c.to_string());
    }
    
    else if c == '*' && chars.peek() == Some(&'*') {
      // chars.next();
      if let Some((txt, count)) = parse_bold(&mut chars) {
        result.push_str(&txt);
        for _ in 0..count {
          chars.next();
        }
        continue;
      }
      push_text(&mut result, &c.to_string());
    }

    else if c == '*' {
      if let Some((ct, count)) = parse_italic(&mut chars) {
        result.push_str(&ct);

        for _ in 0..count {
          chars.next();
        }

        continue;
      } push_text(&mut result, &c.to_string());     }
    else if c == '`' {
      if let Some((ct, count)) = parse_code(&mut chars) {
        result.push_str(&ct);

        for _ in 0..count {
          chars.next();
        }

        continue;
      } push_text(&mut result, &c.to_string());  }
    
    else {
      push_text(&mut result, &c.to_string());
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

fn parse_image(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(String, usize)> {
  if chars.peek() != Some(&'[') {
    return None;
  }

  let remaining: String = chars.clone().collect();

  if let Some(close_bracket) = remaining.find("](") {
    if let Some(close_paren) = remaining[close_bracket + 2..].find(')') {
      let alt = &remaining[1..close_bracket];

      let url_start = close_bracket + 2;
      let url_end = url_start + close_paren;

      let src = &remaining[url_start..url_end];

      let html = format!("<img src=\"{}\" alt=\"{}\">", src, alt);
      let count = url_end + 1;

      // for _ in 0..url_end + 1 {
      //   chars.next();
      // }

      return Some((html, count));
    }
  }

  None
}

fn parse_document(markdown: &str) -> String {
  let lines: Vec<&str> = markdown.lines().collect();
  let mut list_lines: Vec<&str> = Vec::new();
  let mut html = String::new();
  let mut paragraph_lines: Vec<&str> = Vec::new();
  let mut ordered_list_lines: Vec<&str> = Vec::new();
  let mut code_lines: Vec<&str> = Vec::new();
  let mut quote_lines: Vec<&str> = Vec::new();
  let mut in_code_block = false;
  let mut code_language: String = String::new();

  for line in &lines {
    if in_code_block {
      if line.starts_with("```") {
        flush_code_block(&mut html, &mut code_lines, &code_language);

        code_language.clear();
        in_code_block = false;
        continue;
      }

      code_lines.push(line);
      continue;
    }

    if line.starts_with("```") {
      code_language = line[3..].trim().to_string();
      in_code_block = true;
      continue;
    }
    
    if line.trim().is_empty() {
       flush_blockquote(&mut html, &mut quote_lines);
      
      flush_paragraph(&mut html, &mut paragraph_lines);

      flush_lists(
          &mut html,
          &mut list_lines,
          &mut ordered_list_lines,
      );

      continue;
    }

    if line.starts_with('>') {
      let content = line[1..].trim_start();
      quote_lines.push(content);
      continue;
    }

    if let Some(heading) = parse_heading(line) {
      flush_lists(
          &mut html,
          &mut list_lines,
          &mut ordered_list_lines,
      );

      flush_blockquote(&mut html, &mut quote_lines);

      flush_paragraph(&mut html, &mut paragraph_lines);

      html.push_str(&heading);
      html.push('\n');

      continue;
    }

    flush_blockquote(&mut html, &mut quote_lines);
    

    if parse_list_item(line).is_some() {
      list_lines.push(line);
      continue;
    }

    if parse_ordered_list_item(line).is_some() {
      ordered_list_lines.push(line);
      continue;
    }

    flush_blockquote(&mut html, &mut quote_lines);

    flush_unordered_list(&mut html, &mut list_lines);
    
    paragraph_lines.push(line);
  }

  flush_blockquote(&mut html, &mut quote_lines);

  flush_lists(
      &mut html,
      &mut list_lines,
      &mut ordered_list_lines,
  );
  
  flush_paragraph(&mut html, &mut paragraph_lines);

  if in_code_block {
    flush_code_block(&mut html, &mut code_lines, &code_language);
  }

  html
}

fn main() {
  let markdown = fs::read_to_string("content/index.md").expect("Failed to read the md");

  // let text = r#"<script>alert("hello")</script>"#;
  // for line in markdown.lines() {
  //   match parse_heading(line) {
  //     Some(html) => println!("{}", html),
  //     None => println!("Not a heading: {}", line)
  //   }
  // }

  // let lines: Vec<&str> = markdown.lines().collect();
  // let para = parse_paragraph(&lines);

  // println!("{}", escape_html(text));

  
  let html = parse_document(&markdown);
  println!("{}", html)
}
