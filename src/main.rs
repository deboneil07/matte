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

fn parse_inline(text: &str) -> String {
  let mut result = String::new();
  let mut chars = text.chars().peekable();

  // let mut bold: bool = false;
  // let mut italic: bool = false;
  

  while let Some(c) = chars.next() {
    if c == '!' && chars.peek() == Some(&'['){
      // chars.next();

      if let Some(text) = parse_image(&mut chars) {
        result.push_str(&text);
        continue;
      }

      result.push('!');
    }

    else if c == '[' {
      if let Some(html) = parse_link(&mut chars) {
        result.push_str(&html);
        continue;
      }
      result.push(c);
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
      result.push(c);
    }

    else if c == '*' {
      if let Some((ct, count)) = parse_italic(&mut chars) {
        result.push_str(&ct);

        for _ in 0..count {
          chars.next();
        }

        continue;
      } result.push(c)    }
    else if c == '`' {
      if let Some((ct, count)) = parse_code(&mut chars) {
        result.push_str(&ct);

        for _ in 0..count {
          chars.next();
        }

        continue;
      } result.push(c);  }
    
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

fn parse_image(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
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

      for _ in 0..url_end + 1 {
        chars.next();
      }

      return Some(html);
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

  for line in &lines {
    if line.trim().is_empty() {
      if !paragraph_lines.is_empty() {
        html.push_str(&parse_paragraph(&paragraph_lines));
        html.push('\n');

        paragraph_lines.clear();
      }

      if !list_lines.is_empty() {
        html.push_str(&parse_list(&list_lines));
        html.push('\n');

        list_lines.clear();
      }

      if !ordered_list_lines.is_empty() {
        html.push_str(&parse_ordered_list(&ordered_list_lines));
        html.push('\n');

        ordered_list_lines.clear();
      }

      continue;
    }

    if let Some(heading) = parse_heading(line) {
      if !list_lines.is_empty() {
        html.push_str(&parse_list(&lines));
        html.push('\n');

        list_lines.clear();
      }

      if !ordered_list_lines.is_empty() {
        html.push_str(&parse_ordered_list(&ordered_list_lines));
        html.push('\n');

        ordered_list_lines.clear();
      }

      if !paragraph_lines.is_empty() {
        html.push_str(&parse_paragraph(&paragraph_lines));
        html.push('\n');

        paragraph_lines.clear();
      }

      html.push_str(&heading);
      html.push('\n');

      continue;
    }

    if parse_list_item(line).is_some() {
      list_lines.push(line);
      continue;
    }

    if parse_ordered_list_item(line).is_some() {
      ordered_list_lines.push(line);
      continue;
    }

    if !list_lines.is_empty() {
        html.push_str(&parse_list(&list_lines));
        html.push('\n');
    
        list_lines.clear();
    }
    
    paragraph_lines.push(line);
  }

  if !ordered_list_lines.is_empty() {
    html.push_str(&parse_ordered_list(&ordered_list_lines));
    html.push('\n');

    ordered_list_lines.clear();
  }
  if !list_lines.is_empty() {
      html.push_str(&parse_list(&list_lines));
      html.push('\n');
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
