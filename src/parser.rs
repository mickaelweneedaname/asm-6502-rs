use super::ast::Ast;
use super::lexer::{get_token_type, Lexer, Symbol, Token, TokenType};

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Parser {
    pub fn new(text: String) -> Self {
        let mut parser = Self {
            lexer: Lexer::new(text),
            current_token: None,
        };
        parser.current_token = Some(parser.lexer.get_next_token().unwrap());
        parser
    }

    //program = statements EOF
    pub fn parse(&mut self) -> Ast {
        self.statements()
    }

    //statements = (statement NEW_LINE)+
    #[deny(clippy::while_immutable_condition)]
    fn statements(&mut self) -> Ast {
        let mut statements = Vec::new();
        let token_type = self.get_current_token_type();
        while [TokenType::Text, TokenType::Symbol].contains(&token_type) {
            let statement = self.statement();
            if let Some(statement) = statement {
                statements.push(statement)
            }
            if self.current_token_is(&Token::Symbol(Symbol::NewLine)) {
                self.eat(Token::Symbol(Symbol::NewLine));
            } else if self.current_token_is(&Token::Eof) {
                self.eat(Token::Eof);
                break;
            }
        }
        Ast::Statements(statements)
    }

    //statement = comment | (LABEL COLON | INSTRUCTION (args)?) (comment)?
    fn statement(&mut self) -> Option<Ast> {
        if self.current_token_is(&Token::Symbol(Symbol::SemiColon)) {
            self.comment();
            return None;
        }
        let token = self.eat_type(TokenType::Text);
        let (is_label, arg) = match self.get_current_token() {
            Token::Symbol(Symbol::Colon) => {
                self.eat(Token::Symbol(Symbol::Colon));
                (true, None)
            }
            Token::Binary(_)
            | Token::Decimal(_)
            | Token::Hexa16(_)
            | Token::Hexa8(_)
            | Token::Symbol(Symbol::LPar)
            | Token::Symbol(Symbol::HashTag)
            | Token::Text(_) => (false, Some(Box::new(self.arg()))),
            _ => (false, None),
        };
        if let Token::Text(t) = token {
            let ast = if is_label {
                Ast::Label(t)
            } else {
                Ast::Instruction {
                    instruction: t,
                    args: arg,
                }
            };
            if self.current_token_is(&Token::Symbol(Symbol::SemiColon)) {
                self.comment();
            }
            Some(ast)
        } else {
            panic!("Unexpected token : {:?}", token)
        }
    }

    //comment = SEMICOLON
    fn comment(&mut self) {
        self.eat(Token::Symbol(Symbol::SemiColon));
    }

    //args = label| immediate | absolute | zero_page | indirect
    fn arg(&mut self) -> Ast {
        match self.get_current_token_type() {
            TokenType::Hexa16 => self.absolute(),
            TokenType::Hexa8 => self.zero_page(),
            TokenType::Symbol => match self.get_current_token() {
                Token::Symbol(Symbol::LPar) => self.indirect(),
                Token::Symbol(Symbol::HashTag) => self.immediate(),
                _ => panic!(
                    "Parser : unexpected token, {:?}",
                    self.current_token.as_ref().unwrap()
                ),
            },
            TokenType::Text => self.label(),
            _ => panic!("Parser : unexpected token"),
        }
    }

    //immediate = HASHTAG number
    fn immediate(&mut self) -> Ast {
        self.eat(Token::Symbol(Symbol::HashTag));
        self.number()
    }

    //number = binary | hexa8 | hexa16 | decimal
    fn number(&mut self) -> Ast {
        let (token_type, ast) = match self.get_current_token() {
            Token::Decimal(d) => (TokenType::Decimal, Ast::Number8(*d)),
            Token::Binary(b) => (TokenType::Binary, Ast::Number8(*b)),
            Token::Hexa8(h) => (TokenType::Hexa8, Ast::Number8(*h)),
            _ => panic!("Parser : number: unexpected token"),
        };
        self.eat_type(token_type);
        ast
    }

    //label = LABEL
    fn label(&mut self) -> Ast {
        let ast = if let Token::Text(t) = self.eat_type(TokenType::Text) {
            if ["A", "a"].contains(&t.as_ref()){
                Ast::Accumulator
            } else {
                Ast::Label(t)
            }
        } else {
            panic!("Unexpected token {:?}", self.current_token);
        };
        ast
    }

    // absolute = hexa16 (COMA [X,Y])?
    fn absolute(&mut self) -> Ast {
        let token = self.eat_type(TokenType::Hexa16);
        if let Token::Hexa16(h) = token {
            if self.current_token_is(&Token::Symbol(Symbol::Coma)) {
                self.eat(Token::Symbol(Symbol::Coma));
                let token = self.eat_type(TokenType::Text);
                if let Token::Text(t) = token {
                    match t.as_ref() {
                        "x" | "X" | "y" | "Y" => Ast::AsoluteIndexed(h, t.chars().next().unwrap()),
                        _ => panic!("Unexpected token:  {:?}", t),
                    }
                } else {
                    panic!("Unexpected Token : {:?}", token)
                }
            } else {
                Ast::Absolute(h)
            }
        } else {
            panic!("Unexpected Token : {:?}", token);
        }
    }

    // zero_page = hexa8 (COMA [X,Y])?
    fn zero_page(&mut self) -> Ast {
        let token = self.eat_type(TokenType::Hexa8);
        if let Token::Hexa8(h) = token {
            if self.current_token_is(&Token::Symbol(Symbol::Coma)) {
                self.eat(Token::Symbol(Symbol::Coma));
                let token = self.eat_type(TokenType::Text);
                if let Token::Text(t) = token {
                    match t.as_ref() {
                        "x" | "X" | "y" | "Y" => Ast::ZeroPageIndexed(h, t.chars().next().unwrap()),
                        _ => panic!("Unexpected token:  {:?}", t),
                    }
                } else {
                    panic!("Unexpected Token : {:?}", token)
                }
            } else {
                Ast::ZeroPage(h)
            }
        } else {
            panic!("Unexpected Token : {:?}", token);
        }
    }

    // indirect = LPAR (absolute_indirect | zero_page_indirect)
    fn indirect(&mut self) -> Ast {
        self.eat(Token::Symbol(Symbol::LPar));
        match self.get_current_token() {
            Token::Hexa16(_) => self.absolute_indirect(),
            Token::Hexa8(_) => self.zero_page_indirect(),
            _ => panic!("Unexpected token : {:?}", self.current_token.as_ref()),
        }
    }

    // absolute_indirect = hexa16 RPAR
    fn absolute_indirect(&mut self) -> Ast {
        let token = self.eat_type(TokenType::Hexa16);
        if let Token::Hexa16(h) = token {
            self.eat(Token::Symbol(Symbol::RPar));
            Ast::AbsoluteIndirect(h)
        } else {
            panic!("Unexpected token : {:?}", token)
        }
    }

    // zero_page_indirect = hexa8 COMA X RPAR | hexa8 RPAR COMA Y
    fn zero_page_indirect(&mut self) -> Ast {
        let token = self.eat_type(TokenType::Hexa8);
        if let Token::Hexa8(h) = token {
            let token = self.eat_type(TokenType::Symbol);
            match token {
                Token::Symbol(Symbol::RPar) => {
                    self.eat(Token::Symbol(Symbol::Coma));
                    let token = self.eat_type(TokenType::Text);
                    if let Token::Text(t) = token {
                        if t.to_uppercase() != "Y" {
                            panic!("Unexpected token : {:?}", t);
                        }
                    }
                    Ast::IndirectY(h)
                }
                Token::Symbol(Symbol::Coma) => {
                    let token = self.eat_type(TokenType::Text);
                    if let Token::Text(t) = token {
                        if t.to_uppercase() != "X" {
                            panic!("Unexpected token : {:?}", t);
                        }
                    }
                    self.eat(Token::Symbol(Symbol::RPar));
                    Ast::IndirectX(h)
                }
                _ => panic!("Unexpected token : {:?}", token),
            }
        } else {
            panic!("Unexpected token : {:?}", token)
        }
    }

    fn eat(&mut self, token: Token) -> Token {
        let current = self.current_token.take();
        if *current.as_ref().unwrap() == token {
            self.current_token = Some(self.lexer.get_next_token().unwrap());
            current.unwrap()
        } else {
            panic!(
                "Unexpected token : {:?}, should be {:?}",
                self.current_token, token
            )
        }
    }

    fn eat_type(&mut self, token_type: TokenType) -> Token {
        let current = self.current_token.take();
        if get_token_type(current.as_ref().unwrap()) == token_type {
            self.current_token = Some(self.lexer.get_next_token().unwrap());
            current.unwrap()
        } else {
            panic!(
                "Unexpected token : {:?}, should be {:?}",
                self.current_token, token_type
            )
        }
    }

    fn current_token_is(&self, token: &Token) -> bool {
        *self.current_token.as_ref().unwrap() == *token
    }

    fn get_current_token_type(&self) -> TokenType {
        get_token_type(self.current_token.as_ref().unwrap())
    }

    fn get_current_token(&self) -> &Token {
        self.current_token.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_instructions() {
        let txt = r#"       STA ($AA,X) ; this is a comment
                    label:  ADC ($BBAA)
                            STC ($AB),Y ; other comment
                            BNE label   ; still comment
                    other:  INX
                            ORA $F4F5,X ; commenting again
                            BEQ other"#;
        let mut parser = Parser::new(String::from(txt));
        parser.parse();
    }
}
