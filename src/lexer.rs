#[derive(Debug, PartialEq, Clone)]
pub enum Symbol {
    LPar,
    RPar,
    NewLine,
    SemiColon,
    Coma,
    Colon,
    HashTag,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Symbol(Symbol),
    Text(String),
    Decimal(u8),
    Hexa8(u8),
    Hexa16(u16),
    Binary(u8),
    Eof,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Symbol,
    Text,
    Decimal,
    Hexa8,
    Hexa16,
    Binary,
    Eof,
}

pub fn get_token_type(token: &Token) -> TokenType {
    match token {
        Token::Symbol(_) => TokenType::Symbol,
        Token::Text(_) => TokenType::Text,
        Token::Decimal(_) => TokenType::Decimal,
        Token::Hexa8(_) => TokenType::Hexa8,
        Token::Hexa16(_) => TokenType::Hexa16,
        Token::Binary(_) => TokenType::Binary,
        Token::Eof => TokenType::Eof,
    }
}

#[derive(Debug)]
pub struct Lexer {
    text: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(t: String) -> Self {
        Self {
            text: t.chars().collect::<Vec<char>>(),
            position: 0,
        }
    }

    pub fn get_next_token(&mut self) -> Result<Token, String> {
        if self.position >= self.text.len() {
            return Ok(Token::Eof);
        }
        while (self.text[self.position] == ' ') | (self.text[self.position] == '\t') {
            self.position += 1;
        }
        let current_position = self.position;
        let current_char = self.text[current_position];
        let token = match current_char {
            '(' => Ok(Token::Symbol(Symbol::LPar)),
            ')' => Ok(Token::Symbol(Symbol::RPar)),
            '\n' => Ok(Token::Symbol(Symbol::NewLine)),
            ';' => {
                self.skip_comment();
                Ok(Token::Symbol(Symbol::SemiColon))
            }
            ':' => Ok(Token::Symbol(Symbol::Colon)),
            ',' => Ok(Token::Symbol(Symbol::Coma)),
            '#' => Ok(Token::Symbol(Symbol::HashTag)),
            'a'..='z' | 'A'..='Z' | '.' | '=' => self.parse_text(),
            '0'..='9' => self.parse_decimal(),
            '$' => self.parse_hexa(),
            '%' => self.parse_binary(),
            unknown => Err(format!("Unknown char : {unknown}")),
        };
        self.position += 1;
        token
    }

    fn skip_comment(&mut self) {
        self.position += 1;
        self.parse_if(|c: char| c != '\n');
    }

    fn parse_text(&mut self) -> Result<Token, String> {
        let text = self.parse_if(|c: char| c.is_alphabetic());
        Ok(Token::Text(text))
    }

    fn parse_decimal(&mut self) -> Result<Token, String> {
        let decimal_str = self.parse_if(|c: char| c.is_numeric());
        let decimal = decimal_str.parse::<u8>();
        match decimal {
            Ok(decimal) => Ok(Token::Decimal(decimal)),
            Err(_) => Err(format!("decimal value overflow {decimal_str}")),
        }
    }

    fn parse_hexa(&mut self) -> Result<Token, String> {
        self.position += 1;
        let hexa = self.parse_if(|c: char| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F'));
        match hexa.len() {
            1..=2 => Ok(Token::Hexa8(u8::from_str_radix(&hexa, 16).unwrap())),
            3..=4 => Ok(Token::Hexa16(u16::from_str_radix(&hexa, 16).unwrap())),
            _ => Err(format!("Hexadecimal litteral has unexpected size : {hexa}")),
        }
    }

    fn parse_binary(&mut self) -> Result<Token, String> {
        self.position += 1;
        let bin = self.parse_if(|c: char| (c == '0') | (c == '1'));
        match bin.len() {
            1..=8 => Ok(Token::Binary(u8::from_str_radix(&bin, 2).unwrap())),
            _ => Err(format!("Binary litteral has unexpected size : {bin}")),
        }
    }

    fn parse_if<T>(&mut self, predicate: T) -> String
    where
        T: Fn(char) -> bool,
    {
        let mut current_char = self.text[self.position];
        let mut res = String::new();
        while predicate(current_char) {
            res.push(current_char);
            if self.position == self.text.len() - 1 {
                break;
            }
            self.position += 1;
            current_char = self.text[self.position];
        }
        if self.position < self.text.len() - 1 {
            self.position -= 1;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_text() {
        let str = String::from("myLabel");
        let mut lexer = Lexer::new(str);
        assert_eq!(
            lexer.get_next_token(),
            Ok(Token::Text(String::from("myLabel")))
        )
    }

    #[test]
    fn read_decimal() {
        let str = String::from("123");
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.get_next_token(), Ok(Token::Decimal(123)))
    }

    #[test]
    fn read_hexa8() {
        let str = String::from("$A0");
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.get_next_token(), Ok(Token::Hexa8(0xA0)))
    }

    #[test]
    fn read_hexa16() {
        let str = String::from("$0BF1");
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.get_next_token(), Ok(Token::Hexa16(0x0BF1)))
    }

    #[test]
    fn read_hexa_err() {
        let str = String::from("$0B5F1");
        let mut lexer = Lexer::new(str);
        assert!(lexer.get_next_token().is_err())
    }

    #[test]
    fn read_bin() {
        let str = String::from("%101101");
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.get_next_token(), Ok(Token::Binary(0b101101)))
    }

    #[test]
    fn skip_comment_single() {
        let str = String::from(";a comment");
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::SemiColon)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Eof));
    }

    #[test]
    fn skip_comment_multiple() {
        let str = String::from(";a comment\n; other comment");
        let mut lexer = Lexer::new(str);
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::SemiColon)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::NewLine)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::SemiColon)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Eof));
    }

    #[test]
    fn read_mutliple_lines() {
        let str = String::from(
            r#"myLabel
            otherLine"#,
        );
        let mut lexer = Lexer::new(str);
        assert_eq!(
            lexer.get_next_token(),
            Ok(Token::Text(String::from("myLabel")))
        );
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::NewLine)));
        assert_eq!(
            lexer.get_next_token(),
            Ok(Token::Text(String::from("otherLine")))
        );
        assert_eq!(lexer.get_next_token(), Ok(Token::Eof));
    }

    #[test]
    fn read_complete_instruction() {
        let str = String::from("JMP ($AABB) ; this is a comment");
        let mut lexer = Lexer::new(str);
        println!("0");
        assert_eq!(lexer.get_next_token(), Ok(Token::Text(String::from("JMP"))));
        println!("1");
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::LPar)));
        println!("2");
        assert_eq!(lexer.get_next_token(), Ok(Token::Hexa16(0xAABB)));
        println!("3");
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::RPar)));
        println!("4");
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::SemiColon)));
        println!("5");
        assert_eq!(lexer.get_next_token(), Ok(Token::Eof));
    }

    #[test]
    fn read_mutiple_complete_instruction() {
        let str = String::from(
            r###"myLabel: JMP ($AABB) ; this is a comment
        ADC ($FF, X) ; other comment"###,
        );
        let mut lexer = Lexer::new(str);
        assert_eq!(
            lexer.get_next_token(),
            Ok(Token::Text(String::from("myLabel")))
        );
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::Colon)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Text(String::from("JMP"))));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::LPar)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Hexa16(0xAABB)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::RPar)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::SemiColon)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::NewLine)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Text(String::from("ADC"))));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::LPar)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Hexa8(0xFF)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::Coma)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Text(String::from("X"))));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::RPar)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Symbol(Symbol::SemiColon)));
        assert_eq!(lexer.get_next_token(), Ok(Token::Eof));
    }
}
