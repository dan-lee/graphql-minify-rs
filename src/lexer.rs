use crate::block_string::{dedent_block_lines_mut, print_block_string, BlockStringToken};
use logos::{Lexer, Logos};

#[derive(Debug, PartialEq, Clone, Default)]
/// An enumeration of errors that can occur during the lexing process.
pub enum LexingError {
  #[default]
  UnknownToken,
  /// First value is the index of the first character of the unterminated string
  UnterminatedString(usize),
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"([\s,]+|#[^\r\n]*)+")]
#[logos(error = LexingError)]
pub(crate) enum Token<'a> {
  #[token("{")]
  BraceOpen,

  #[token("}")]
  BraceClose,

  #[token("(")]
  ParenOpen,

  #[token(")")]
  ParenClose,

  #[token("[")]
  BracketOpen,

  #[token("]")]
  BracketClose,

  #[token(":")]
  Colon,

  #[token("=")]
  Equals,

  #[token("!")]
  Exclamation,

  #[token("?")]
  Question,

  #[token("&")]
  Ampersand,

  #[token("|")]
  Pipe,

  #[token("...")]
  Ellipsis,

  #[regex(r#"""""#)]
  BlockStringDelimiter,

  #[regex(r#""([^"\\]*(\\.[^"\\]*)*)""#, |lexer| match lexer.slice() {
      s if s.contains(['\n', '\r']) => Err(LexingError::UnterminatedString(lexer.span().start)),
      s => Ok(s),
  })]
  String(&'a str),

  #[regex("-?[0-9]+")]
  Int(&'a str),

  #[regex("-?[0-9]+\\.[0-9]+(e-?[0-9]+)?")]
  Float(&'a str),

  #[regex("true|false")]
  Bool(&'a str),

  #[regex("@[a-zA-Z_][a-zA-Z0-9_]*")]
  Directive(&'a str),

  #[regex("\\$[a-zA-Z_][a-zA-Z0-9_]*")]
  Variable(&'a str),

  #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
  Identifier(&'a str),
}

impl<'a> Token<'a> {
  pub(crate) fn parse_block_string(&self, lexer: &mut Lexer<'a, Token<'a>>) -> String {
    let mut lines = vec![];
    let mut current_line = String::new();

    let remainder = lexer.remainder();
    let mut block_lexer = BlockStringToken::lexer(remainder);

    while let Some(Ok(token)) = block_lexer.next() {
      match token {
        BlockStringToken::NewLine => {
          lines.push(current_line);
          current_line = String::new();
        }
        BlockStringToken::Text | BlockStringToken::Quote | BlockStringToken::EscapeSeq => {
          current_line.push_str(block_lexer.slice())
        }
        BlockStringToken::EscapedTripleQuote => current_line.push_str(r#"""""#),
        BlockStringToken::TripleQuote => break,
      }
    }

    if !current_line.is_empty() {
      lines.push(current_line);
    }

    lexer.bump(remainder.len() - block_lexer.remainder().len());

    dedent_block_lines_mut(&mut lines);
    print_block_string(lines.join("\n"))
  }
}
