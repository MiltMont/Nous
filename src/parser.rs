use std::collections::{HashMap, VecDeque};

use logos::Lexer;

use crate::{ast, lexer::Token};

/// Turns a stream of Tokens into a Parser object.
/// ```
/// let mut lexer = Token::Lexer(&file);
/// let mut parser : Parser = Parser::build(&mut lexer);
/// // Creating an ast object
/// let ast_program : ast::Program = parser.to_ast_program();
/// ```
pub struct Parser<'a> {
    /// Queue of tokens
    tokens: VecDeque<Token>,
    /// Current token in token stream
    current_token: Token,
    /// Next token in token stream
    peek_token: Token,

    /// Map of operator precedences
    precedences: HashMap<&'a Token, i64>,
}

impl<'a> Parser<'a> {
    /// Returns a Parser given a Lexer.
    pub fn build(lexer: &mut Lexer<Token>) -> Self {
        let mut tokens: VecDeque<Token> =
            VecDeque::from_iter(lexer.into_iter().map(|x| x.expect("Building token queue")));

        let current_token = tokens.pop_front().unwrap();
        let peek_token = tokens.pop_front().unwrap();

        let mut precedences = HashMap::new();

        // Defining the precedence values.
        precedences.insert(&Token::Mul, 50);
        precedences.insert(&Token::Div, 50);
        precedences.insert(&Token::Remainder, 50);
        precedences.insert(&Token::Add, 45);
        precedences.insert(&Token::Negation, 45);

        Self {
            tokens,
            current_token,
            peek_token,
            precedences,
        }
    }

    /// Generates and AST from the constructed parser.
    pub fn to_ast_program(&mut self) -> ast::Program {
        self.parse_program().expect("Parser: Parsing program.")
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

    fn next_token_is(&self, token: &Token) -> bool {
        self.peek_token == *token
    }

    /// Returns an AST Program or an Error string.
    ///
    /// <program> ::== <function>
    fn parse_program(&mut self) -> Result<ast::Program, String> {
        let function = self.parse_function()?;
        Ok(ast::Program(function))
    }

    /// Returns an ast::Function or an Error String.
    ///
    /// <function> ::== "int" <identifier> "(" "void" ")" "{" <statement> "}"
    fn parse_function(&mut self) -> Result<ast::Function, String> {
        if self.current_token_is(&Token::Int) {
            self.next_token();

            let identifier = self.parse_identifier()?;
            let expected_structure = vec![Token::LParen, Token::Void, Token::RParen, Token::LBrace];

            // Check if incoming token stream matches the expected_structure
            for token in expected_structure {
                if !self.current_token_is(&token) {
                    return Err(format!(
                        "Expected {:?}, got {:?}",
                        token, self.current_token
                    ));
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
                Err(format!("Expected }} but found {:?}", self.current_token))
            }
        } else {
            Err(format!("Expected int but found {:?}", self.current_token))
        }
    }

    /// Obtains the type of the current
    /// unary operation
    fn parse_unaryop(&mut self) -> Result<ast::UnaryOperator, String> {
        match self.current_token {
            Token::Negation => {
                // HACK: Should this be done?
                self.next_token();

                Ok(ast::UnaryOperator::Negate)
            }
            Token::BitComp => {
                self.next_token();

                Ok(ast::UnaryOperator::Complement)
            }
            _ => Err(String::from("Not a unary operator")),
        }
    }

    /// Obtains the variant of the current
    /// binary operation
    fn parse_binaryop(&mut self) -> Result<ast::BinaryOperator, String> {
        match self.current_token {
            Token::Add => Ok(ast::BinaryOperator::Add),
            Token::Negation => Ok(ast::BinaryOperator::Subtract),
            Token::Mul => Ok(ast::BinaryOperator::Multiply),
            Token::Div => Ok(ast::BinaryOperator::Divide),
            Token::Remainder => Ok(ast::BinaryOperator::Remainder),
            _ => Err(format!(
                "Not a binary operator, found {:?}",
                self.current_token
            )),
        }
    }

    /// Returns an ast::Identifier or an Error String.
    ///
    /// <identifier> ::== An identifier token
    fn parse_identifier(&mut self) -> Result<ast::Identifier, String> {
        if let Token::Identifier(s) = self.current_token.clone() {
            self.next_token();
            Ok(ast::Identifier(s.to_string()))
        } else {
            Err(format!(
                "Error parsing identifier, got {:?}",
                self.current_token
            ))
        }
    }

    /// Parses the grammar:
    ///
    /// <exp> ::== <factor> | <exp> <binop> <exp>
    fn parse_expression(&mut self) -> Result<ast::Expression, String> {
        let mut left = self.parse_factor().expect("Parsing left factor");

        while self.next_token_is(&Token::Add) || self.next_token_is(&Token::Negation) {
            self.next_token();
            let operator = self.parse_binaryop().expect("Parsing binary operator");
            self.next_token();
            let right = self.parse_factor().expect("Parsing right factor");
            left = ast::Expression::Binary(operator, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// <factor> ::== <int> | <unop> <factor> | "(" <exp> ")"
    fn parse_factor(&mut self) -> Result<ast::Expression, String> {
        match self.current_token {
            Token::Constant(i) => Ok(ast::Expression::Constant(i)),
            // If token is "~" or "-"
            Token::Negation | Token::BitComp => {
                let operator = self.parse_unaryop().expect("Parsing unary operator");
                let inner_expression = self.parse_factor().expect("Parsing inner expression");

                Ok(ast::Expression::Unary(operator, Box::new(inner_expression)))
            }
            Token::LParen => {
                self.next_token();
                let inner_expression = self.parse_expression();
                self.next_token();
                if self.current_token_is(&Token::RParen) {
                    inner_expression
                } else {
                    Err(String::from(
                        "Malformed factor, missing closing parenthesis",
                    ))
                }
            }
            _ => Err(format!("Malformed factor, found {:?}", self.current_token)),
        }
    }

    /// Parses the following grammar:
    ///
    /// <statement> ::== "return" <exp> ";"
    fn parse_statement(&mut self) -> Result<ast::Statement, String> {
        if self.current_token_is(&Token::Return) {
            self.next_token();

            let expression = self.parse_expression()?;

            self.next_token();

            if self.current_token_is(&Token::Semicolon) {
                self.next_token();
                Ok(ast::Statement::Return(expression))
            } else {
                Err(format!("Expected ; but found {:?}", self.current_token))
            }
        } else {
            Err(format!(
                "Expected RETURN but found {:?}",
                self.current_token
            ))
        }
    }

    /// Returns the precedence of a given operator.
    fn get_precedence(&self, binary_operator: &Token) -> Result<i64, String> {
        if let Some(i) = self.precedences.get(binary_operator) {
            Ok(*i)
        } else {
            Err(String::from("Token not present in precedences table"))
        }
    }

    /// Returns true if the current token is a
    /// binary operator
    fn is_binary_operator(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Add | Token::Mul | Token::Div | Token::Negation
        )
    }
}
