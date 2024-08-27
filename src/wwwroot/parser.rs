use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
enum Token {
    TagOpen(String),
    TagClose(String),
    Text(String),
    Attribute(String, String),
}

#[derive(Debug)]
enum ParseError {
    UnexpectedEndOfInput,
    UnexpectedToken(Token),
}

struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
        }
    }

    fn next_token(&mut self) -> Option<Result<Token, ParseError>> {
        self.consume_whitespace();
        match self.chars.peek() {
            Some('<') => {
                self.chars.next(); // Consume '<'
                match self.chars.peek() {
                    Some('/') => {
                        self.chars.next(); // Consume '/'
                        let tag_name = self.consume_while(|c| c.is_alphanumeric());
                        self.consume_until('>');
                        Some(Ok(Token::TagClose(tag_name)))
                    }
                    Some(_) => {
                        let tag_name = self.consume_while(|c| c.is_alphanumeric());
                        let mut attributes = vec![];
                        loop {
                            self.consume_whitespace();
                            match self.chars.peek() {
                                Some('>') => {
                                    self.chars.next(); // Consume '>'
                                    break;
                                }
                                Some(_) => {
                                    let attr_name = self.consume_while(|c| c.is_alphanumeric());
                                    self.consume_until('=');
                                    self.chars.next(); // Consume '='
                                    self.consume_until('"');
                                    self.chars.next(); // Consume '"'
                                    let attr_value = self.consume_while(|c| c != '"');
                                    self.chars.next(); // Consume closing '"'
                                    attributes.push((attr_name, attr_value));
                                }
                                None => return Some(Err(ParseError::UnexpectedEndOfInput)),
                            }
                        }
                        Some(Ok(Token::TagOpen(tag_name)))
                    }
                    None => Some(Err(ParseError::UnexpectedEndOfInput)),
                }
            }
            Some(_) => Some(Ok(Token::Text(self.consume_while(|c| c != '<')))),
            None => None,
        }
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(&c) = self.chars.peek() {
            if test(c) {
                result.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        result
    }

    fn consume_until(&mut self, stop: char) {
        while let Some(&c) = self.chars.peek() {
            if c == stop {
                break;
            }
            self.chars.next();
        }
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }
}

#[derive(Debug)]
struct Node {
    tag: String,
    children: Vec<Node>,
    text: Option<String>,
}

impl Node {
    fn new(tag: String) -> Self {
        Node {
            tag,
            children: vec![],
            text: None,
        }
    }

    fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }
}

struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Option<Result<Token, ParseError>>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(input),
            current_token: None,
        }
    }

    fn parse(&mut self) -> Result<Node, ParseError> {
        self.current_token = self.tokenizer.next_token();
        self.parse_node()
    }

    fn parse_node(&mut self) -> Result<Node, ParseError> {
        match self.current_token.take() {
            Some(Ok(Token::TagOpen(tag_name))) => {
                let mut node = Node::new(tag_name);
                self.current_token = self.tokenizer.next_token();
                while let Some(Ok(token)) = &self.current_token {
                    match token {
                        Token::TagClose(_) => {
                            self.current_token = self.tokenizer.next_token();
                            break;
                        }
                        Token::TagOpen(_) => {
                            let child = self.parse_node()?;
                            node.add_child(child);
                        }
                        Token::Text(text) => {
                            node.set_text(text.clone());
                            self.current_token = self.tokenizer.next_token();
                        }
                        _ => return Err(ParseError::UnexpectedToken(token.clone())),
                    }
                }
                Ok(node)
            }
            Some(Ok(Token::Text(text))) => {
                let mut node = Node::new(String::new());
                node.set_text(text);
                Ok(node)
            }
            Some(Err(e)) => Err(e),
            _ => Err(ParseError::UnexpectedEndOfInput),
        }
    }
}

fn main() {
    let html = "<html><body><h1>Hello, World!</h1><p>This is a paragraph.</p></body></html>";
    let mut parser = Parser::new(html);
    match parser.parse() {
        Ok(document) => println!("{:?}", document),
        Err(e) => println!("Error: {:?}", e),
    }
}