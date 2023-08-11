use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum BlockStringToken {
  #[token("\"")]
  Quote,

  #[regex(r#"\n|\r\n|\r"#)]
  NewLine,

  #[regex(r#"\\""""#)]
  EscapedTripleQuote,

  #[regex(r#"\\"#)]
  EscapeSeq,

  #[regex(r#"[^"\r\n\\]+"#)]
  Text,

  #[token(r#"""""#)]
  TripleQuote,
}

pub(crate) fn print_block_string<T: AsRef<str>>(input: T) -> String {
  let str = input.as_ref();
  let str = str.replace(r#"""""#, r#"\""""#);
  let lines = str.lines().collect::<Vec<_>>();

  let force_leading_new_line = lines.len() > 1
    && lines[1..].iter().all(|line| match line.chars().next() {
      Some(ch) => line.is_empty() || is_gql_whitespace(ch),
      None => true,
    });

  let has_trailing_triple_quotes = str.ends_with(r#"\""""#);
  let has_trailing_quote = str.ends_with('"') && !has_trailing_triple_quotes;
  let has_trailing_slash = str.ends_with('\\');
  let force_trailing_newline = has_trailing_quote || has_trailing_slash;

  let mut result = String::with_capacity(str.len() + 7);

  result.push_str(r#"""""#);

  if force_leading_new_line {
    result.push('\n');
  }

  for line in lines {
    result.push_str(line);
    result.push('\n');
  }

  if !force_trailing_newline && result.ends_with('\n') {
    result.pop();
  }

  result.push_str(r#"""""#);
  result
}

pub(crate) fn dedent_block_lines_mut(lines: &mut Vec<String>) {
  let mut common_indent = usize::MAX;
  let mut first_non_empty_line = None;
  let mut last_non_empty_line = None;

  for (i, line) in lines.iter().enumerate() {
    let indent = leading_whitespace(line);

    if indent < line.len() {
      first_non_empty_line.get_or_insert(i);
      last_non_empty_line = Some(i);

      if i != 0 && indent < common_indent {
        common_indent = indent;
      }
    }
  }

  match (first_non_empty_line, last_non_empty_line) {
    (Some(start), Some(end)) => {
      for line in lines.iter_mut().skip(1) {
        if line.len() > common_indent {
          *line = line.split_off(common_indent);
        } else {
          line.clear();
        }
      }

      lines.drain(..start);
      lines.drain((end + 1 - start)..);
    }
    _ => lines.clear(),
  }
}

fn is_gql_whitespace(ch: char) -> bool {
  ch == ' ' || ch == '\t'
}

fn leading_whitespace(s: &str) -> usize {
  s.chars().take_while(|&ch| is_gql_whitespace(ch)).count()
}

#[cfg(test)]
mod test_dedent {
  use super::dedent_block_lines_mut;

  fn get_dedented_vec(lines: &[&str]) -> Vec<String> {
    let mut lines = lines.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    dedent_block_lines_mut(&mut lines);
    lines.iter().map(|s| s.to_string()).collect()
  }

  #[test]
  fn does_not_dedent_first_line() {
    assert_eq!(get_dedented_vec(&["  a"]), &["  a"]);
    assert_eq!(get_dedented_vec(&[" a", "  b"]), &[" a", "b"]);
  }

  #[test]
  fn removes_minimal_indentation_length() {
    assert_eq!(get_dedented_vec(&["", " a", "  b"]), &["a", " b"]);
    assert_eq!(get_dedented_vec(&["", "  a", " b"]), &[" a", "b"]);
    assert_eq!(
      get_dedented_vec(&["", "  a", " b", "c"]),
      &["  a", " b", "c"]
    );
  }

  #[test]
  fn dedent_both_tab_and_space_as_single_character() {
    assert_eq!(
      get_dedented_vec(&["", "\ta", "          b"]),
      &["a", "         b"]
    );
    assert_eq!(
      get_dedented_vec(&["", "\t a", "          b"]),
      &["a", "        b"]
    );
    assert_eq!(
      get_dedented_vec(&["", " \t a", "          b"]),
      &["a", "       b"]
    );
  }

  #[test]
  fn dedent_do_not_take_empty_lines_into_account() {
    assert_eq!(get_dedented_vec(&["a", "", " b"]), &["a", "", "b"]);
    assert_eq!(get_dedented_vec(&["a", " ", "  b"]), &["a", "", "b"]);
  }

  #[test]
  fn removes_uniform_indentation_from_a_string() {
    let lines = vec![
      "",
      "    Hello,",
      "      World!",
      "",
      "    Yours,",
      "      GraphQL.",
    ];
    assert_eq!(
      get_dedented_vec(&lines),
      &["Hello,", "  World!", "", "Yours,", "  GraphQL.",]
    );
  }

  #[test]
  fn removes_empty_leading_and_trailing_lines() {
    let lines = vec![
      "",
      "",
      "    Hello,",
      "      World!",
      "",
      "    Yours,",
      "      GraphQL.",
      "",
      "",
    ];
    assert_eq!(
      get_dedented_vec(&lines),
      &["Hello,", "  World!", "", "Yours,", "  GraphQL.",]
    );
  }

  #[test]
  fn removes_blank_leading_and_trailing_lines() {
    let lines = vec![
      "  ",
      "        ",
      "    Hello,",
      "      World!",
      "",
      "    Yours,",
      "      GraphQL.",
      "        ",
      "  ",
    ];
    assert_eq!(
      get_dedented_vec(&lines),
      &["Hello,", "  World!", "", "Yours,", "  GraphQL.",]
    );
  }

  #[test]
  fn retains_indentation_from_first_line() {
    let lines = vec![
      "    Hello,",
      "      World!",
      "",
      "    Yours,",
      "      GraphQL.",
    ];
    assert_eq!(
      get_dedented_vec(&lines),
      &["    Hello,", "  World!", "", "Yours,", "  GraphQL.",]
    );
  }

  #[test]
  fn does_not_alter_trailing_spaces() {
    let lines = vec![
      "               ",
      "    Hello,     ",
      "      World!   ",
      "               ",
      "    Yours,     ",
      "      GraphQL. ",
      "               ",
    ];
    assert_eq!(
      get_dedented_vec(&lines),
      &[
        "Hello,     ",
        "  World!   ",
        "           ",
        "Yours,     ",
        "  GraphQL. ",
      ]
    );
  }
}

#[cfg(test)]
mod test_print {
  use super::print_block_string;

  #[test]
  fn does_not_escape_characters() {
    let str = r#" \ / \b \f \n \r \t"#;
    assert_eq!(print_block_string(str), r#"""" \ / \b \f \n \r \t""""#);
  }

  #[test]
  fn by_default_print_block_strings_as_single_line() {
    let str = r#"one liner"#;
    assert_eq!(print_block_string(str), r#""""one liner""""#);
  }

  #[test]
  fn by_default_print_block_strings_ending_with_triple_quotation_as_multi_line() {
    let str = r#"triple quotation """"#;
    assert_eq!(print_block_string(str), r#""""triple quotation \"""""""#);
  }

  #[test]
  fn correctly_prints_single_line_with_leading_space() {
    let str = "    space-led string";
    assert_eq!(print_block_string(str), r#""""    space-led string""""#);
  }

  #[test]
  fn correctly_prints_single_line_with_leading_space_and_trailing_quotation() {
    let str = "    space-led value \"quoted string\"";
    assert_eq!(
      print_block_string(str),
      r#""""    space-led value "quoted string"
""""#
    );
  }

  #[test]
  fn correctly_prints_single_line_with_trailing_backslash() {
    let str = "backslash \\";
    assert_eq!(
      print_block_string(str),
      r#""""backslash \
""""#
    );
  }

  #[test]
  fn correctly_prints_multi_line_with_internal_indent() {
    let str = "no indent\n with indent";
    assert_eq!(
      print_block_string(str),
      r#""""
no indent
 with indent""""#
    );
  }

  #[test]
  fn correctly_prints_string_with_a_first_line_indentation() {
    let str = ["    first  ", "  line     ", "indentation", "     string"].join("\n");

    assert_eq!(
      print_block_string(&str),
      [
        r#""""    first  "#,
        "  line     ",
        "indentation",
        r#"     string""""#
      ]
      .join("\n")
    );
  }
}
