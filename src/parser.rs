use std::{
    collections::{HashMap, VecDeque},
    fs::{self},
    path::PathBuf,
};

use logos::{Lexer, Logos};

use crate::{
    ast,
    errors::{Error, Result},
    lexer::Token,
};

/// Turns a stream of Tokens into a Parser object.
/// ```
/// # use nous::parser::Parser;
/// # use logos::Logos;
/// # use nous::lexer::Token;
/// # use nous::ast;
/// # let file = String::from("int main(void) { return 2; }");
/// let mut lexer = Token::lexer(&file);
/// let mut parser : Parser = Parser::from_lexer(&mut lexer);
/// // Creating an ast object
/// let ast_program : ast::Program = parser.to_ast_program();
/// ```
pub struct Parser {
    /// Queue of tokens
    tokens: VecDeque<Token>,
    /// Current token in token stream
    current_token: Token,
    /// Next token in token stream
    peek_token: Token,

    /// Map of operator precedences
    precedences: HashMap<Token, usize>,
}

impl From<String> for Parser {
    fn from(value: String) -> Self {
        let mut tokens: VecDeque<Token> = VecDeque::from_iter(
            Token::lexer(&value).map(|token| token.expect("Should return token")),
        );

        let current_token = tokens.pop_front().unwrap();
        let peek_token = tokens.pop_front().unwrap();

        Self {
            tokens,
            current_token,
            peek_token,
            precedences: Parser::get_precedence_map(),
        }
    }
}

impl From<PathBuf> for Parser {
    fn from(value: PathBuf) -> Self {
        let file = fs::read_to_string(value).expect("Should read file");

        Parser::from(file)
    }
}

impl Parser {
    //
    // TODO: Document which functions consume the current
    // token in the token stream.
    //

    /// Returns a Parser given a lexer.
    pub fn from_lexer(lexer: &mut Lexer<Token>) -> Self {
        let mut tokens: VecDeque<Token> =
            VecDeque::from_iter(lexer.into_iter().map(|x| x.expect("Building token queue")));

        let current_token = tokens.pop_front().unwrap();
        let peek_token = tokens.pop_front().unwrap();

        Self {
            tokens,
            current_token,
            peek_token,
            precedences: Parser::get_precedence_map(),
        }
    }

    pub fn get_precedence_map() -> HashMap<Token, usize> {
        let mut precedences = HashMap::new();

        // Defining the precedence values.
        precedences.insert(Token::Mul, 50);
        precedences.insert(Token::Div, 50);
        precedences.insert(Token::Remainder, 50);
        precedences.insert(Token::Add, 45);
        precedences.insert(Token::Negation, 45);
        precedences.insert(Token::LessThan, 35);
        precedences.insert(Token::LessThanOrEq, 35);
        precedences.insert(Token::GreaterThan, 35);
        precedences.insert(Token::GreaterThanOrEq, 35);
        precedences.insert(Token::EqualTo, 30);
        precedences.insert(Token::NotEqualTo, 30);
        precedences.insert(Token::And, 10);
        precedences.insert(Token::Or, 5);

        precedences
    }

    /// Generates and AST from the constructed parser.
    pub fn to_ast_program(&mut self) -> Result<ast::Program> {
        self.parse_program()
    }

    /// Consumes the next token in token stream
    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self
            .tokens
            .pop_front()
            .unwrap_or(self.current_token.clone());
    }

    /// Compares current token with a given token
    fn current_token_is(&self, token: &Token) -> bool {
        self.current_token == *token
    }

    #[allow(dead_code)]
    fn next_token_is(&self, token: &Token) -> bool {
        self.peek_token == *token
    }

    /// Returns an AST Program or an Error string.
    ///
    /// <program> ::== <function>
    fn parse_program(&mut self) -> Result<ast::Program> {
        let function = self.parse_function()?;
        Ok(ast::Program(function))
    }

    /// Returns an ast::Function or an Error String.
    ///
    /// <function> ::== "int" <identifier> "(" "void" ")" "{" <statement> "}"
    fn parse_function(&mut self) -> Result<ast::Function> {
        if self.current_token_is(&Token::Int) {
            self.next_token();

            let identifier = self.parse_identifier()?;
            let expected_structure = vec![Token::LParen, Token::Void, Token::RParen, Token::LBrace];

            // Check if incoming token stream matches the expected_structure
            for token in expected_structure {
                if !self.current_token_is(&token) {
                    return Err(Error::UnexpectedToken {
                        expected: token.clone(),
                        found: self.current_token.clone(),
                        message: Some("within `parse_function`".into()),
                    });
                } else {
                    self.next_token();
                }
            }

            let statement = self.parse_statement()?;

            if self.current_token_is(&Token::RBrace) {
                self.next_token();
                Ok(ast::Function {
                    name: identifier,
                    body: statement,
                })
            } else {
                Err(Error::UnexpectedToken {
                    expected: Token::RBrace,
                    found: self.current_token.clone(),
                    message: None,
                })
            }
        } else {
            Err(Error::UnexpectedToken {
                expected: Token::Int,
                found: self.current_token.clone(),
                message: None,
            })
        }
    }

    /// Matches on the current token, if it is
    /// a unary operator then *it advances the token stream*
    /// and returns the unary operator wrapped in a Result
    /// variant. Otherwise it returns an error message.
    fn parse_unaryop(&mut self) -> Result<ast::UnaryOperator> {
        match self.current_token {
            Token::Negation => {
                self.next_token();

                Ok(ast::UnaryOperator::Negate)
            }
            Token::BitComp => {
                self.next_token();

                Ok(ast::UnaryOperator::Complement)
            }
            Token::Not => {
                self.next_token();
                Ok(ast::UnaryOperator::Not)
            }
            _ => Err(Error::NotUnop {
                found: self.current_token.clone(),
            }),
        }
    }

    /// Obtains the variant of the current
    /// binary operation
    fn parse_binaryop(&mut self) -> Result<ast::BinaryOperator> {
        match self.current_token {
            Token::Add => Ok(ast::BinaryOperator::Add),
            Token::Negation => Ok(ast::BinaryOperator::Subtract),
            Token::Mul => Ok(ast::BinaryOperator::Multiply),
            Token::Div => Ok(ast::BinaryOperator::Divide),
            Token::Remainder => Ok(ast::BinaryOperator::Remainder),
            Token::LessThan => Ok(ast::BinaryOperator::LessThan),
            Token::LessThanOrEq => Ok(ast::BinaryOperator::LessOrEqual),
            Token::GreaterThan => Ok(ast::BinaryOperator::GreaterThan),
            Token::GreaterThanOrEq => Ok(ast::BinaryOperator::GreaterOrEqual),
            Token::EqualTo => Ok(ast::BinaryOperator::Equal),
            Token::NotEqualTo => Ok(ast::BinaryOperator::NotEqual),
            Token::And => Ok(ast::BinaryOperator::And),
            Token::Or => Ok(ast::BinaryOperator::Or),
            _ => Err(Error::NotBinop {
                found: self.current_token.clone(),
            }),
        }
    }

    /// Returns an ast::Identifier or an Error String.
    ///
    /// <identifier> ::== An identifier token
    fn parse_identifier(&mut self) -> Result<ast::Identifier> {
        if let Token::Identifier(s) = self.current_token.clone() {
            self.next_token();
            Ok(ast::Identifier(s.to_string()))
        } else {
            Err(Error::UnexpectedToken {
                expected: Token::Identifier("identifier name".into()),
                found: self.current_token.clone(),
                message: None,
            })
        }
    }

    /// Parses the grammar:
    ///
    /// <exp> ::== <factor> | <exp> <binop> <exp>
    fn parse_expression(&mut self, min_precedence: usize) -> Result<ast::Expression> {
        let mut left = self.parse_factor()?;

        let mut next_token = self.peek_token.clone();

        while self.is_binary_operator(&next_token)
            && self.get_precedence(&next_token)? >= min_precedence
        {
            self.next_token();
            let operator = self.parse_binaryop()?;
            self.next_token();
            let right = self.parse_expression(self.get_precedence(&next_token)? + 1)?;
            left = ast::Expression::Binary(operator, Box::new(left), Box::new(right));

            next_token = self.peek_token.clone()
        }

        Ok(left)
    }

    /// <factor> ::== <int> | <unop> <factor> | "(" <exp> ")"
    fn parse_factor(&mut self) -> Result<ast::Expression> {
        let current = self.current_token.clone();
        match current {
            Token::Constant(i) => Ok(ast::Expression::Constant(i)),
            // If token is "~" or "-"
            Token::Negation | Token::BitComp | Token::Not => {
                let operator = self.parse_unaryop()?;
                let inner_expression = self.parse_factor()?;

                Ok(ast::Expression::Unary(operator, Box::new(inner_expression)))
            }
            Token::LParen => {
                self.next_token();
                let inner_expression = self.parse_expression(0);
                self.next_token();
                if self.current_token_is(&Token::RParen) {
                    inner_expression
                } else {
                    Err(Error::MalformedFactor {
                        missing: Some(Token::RParen),
                        found: self.current_token.clone(),
                    })
                }
            }
            _ => Err(Error::MalformedFactor {
                missing: None,
                found: self.current_token.clone(),
            }),
        }
    }

    /// Parses the following grammar:
    ///
    /// <statement> ::== "return" <exp> ";"
    fn parse_statement(&mut self) -> Result<ast::Statement> {
        if self.current_token_is(&Token::Return) {
            self.next_token();

            let expression = self.parse_expression(0)?;
            self.next_token();

            if self.current_token_is(&Token::Semicolon) {
                self.next_token();
                Ok(ast::Statement::Return(expression))
            } else {
                Err(Error::UnexpectedToken {
                    expected: Token::Semicolon,
                    found: self.current_token.clone(),
                    message: None,
                })
            }
        } else {
            Err(Error::UnexpectedToken {
                expected: Token::Return,
                found: self.current_token.clone(),
                message: None,
            })
        }
    }

    /// Returns the precedence of a given operator.
    fn get_precedence(&self, binary_operator: &Token) -> Result<usize> {
        if let Some(i) = self.precedences.get(binary_operator) {
            Ok(*i)
        } else {
            Err(Error::Precedence {
                found: binary_operator.clone(),
            })
        }
    }

    /// Returns true if the current token is a
    /// binary operator
    fn is_binary_operator(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Add
                | Token::Mul
                | Token::Div
                | Token::Negation
                | Token::Remainder
                | Token::And
                | Token::Or
                | Token::EqualTo
                | Token::NotEqualTo
                | Token::LessThan
                | Token::LessThanOrEq
                | Token::GreaterThan
                | Token::GreaterThanOrEq
        )
    }
}
