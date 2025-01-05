use std::{
    collections::VecDeque,
    fs::{self},
    path::PathBuf,
};

use logos::{Lexer, Logos};

use crate::{
    ast::{self, Block, Identifier},
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
/// let ast_program : ast::Program = parser.to_ast_program().expect("Should return a program");
/// ```
pub struct Parser {
    /// Queue of tokens
    tokens: VecDeque<Token>,
    /// Current token in token stream
    current_token: Token,
    /// Next token in token stream
    peek_token: Token,
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
        }
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

    //fn peek_token_is(&self, token: &Token) -> bool {
    //    &self.peek_token == token
    //}

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
    /// <function> ::= "int" <identifier> "(" "void" ")" <block>
    fn parse_function(&mut self) -> Result<ast::Function> {
        if self.current_token_is(&Token::Int) {
            self.next_token();

            let identifier = self.parse_identifier()?;
            //matching: (void)
            let expected_structure = vec![Token::LParen, Token::Void, Token::RParen];

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

            let function_body = self.parse_block()?;

            Ok(ast::Function {
                name: identifier,
                body: function_body,
            })
        } else {
            Err(Error::UnexpectedToken {
                expected: Token::Int,
                found: self.current_token.clone(),
                message: Some("Within `parse_function`".into()),
            })
        }
    }

    /// <block> ::= "{" { <block-item> } "}"
    fn parse_block(&mut self) -> Result<Block> {
        if self.current_token_is(&Token::LBrace) {
            self.next_token();
            let mut blocks = Vec::new();

            // FIX: What happens if we dont have an RBrace?
            while !self.current_token_is(&Token::RBrace) {
                blocks.push(self.parse_block_item()?);
            }

            Ok(Block(blocks))
        } else {
            Err(Error::UnexpectedToken {
                message: Some("Within `parse_block`".into()),
                expected: Token::LBrace,
                found: self.current_token.clone(),
            })
        }
    }

    /// <block-item> ::== <statement> | <declaration>
    fn parse_block_item(&mut self) -> Result<ast::BlockItem> {
        // We need a way to tell wether the current block
        // item is a statement or a declaration.
        // To do this, we look at the first token; if it is
        // `Token::Int`, then it's a declaration, otherwise
        // it's a statement.
        if self.current_token_is(&Token::Int) {
            // This is a declaration
            Ok(ast::BlockItem::D(self.parse_declaration()?))
        } else {
            // This is a statement
            Ok(ast::BlockItem::S(self.parse_statement()?))
        }
    }

    /// <declaration> ::== "int" <identifier> [ "=" <exp> ] ";"
    fn parse_declaration(&mut self) -> Result<ast::Declaration> {
        // This should check whether the identifier in the grammar
        // rule is followed by an `=` token, which means the
        // initializer is present, or a `;` token, which means
        // the initilizar is absent.
        if let Token::Int = self.current_token {
            self.next_token();
            // We must have an identifier now.
            let name = self.parse_identifier()?;

            // we check if the initializer is present
            if let Token::Assign = self.current_token {
                self.next_token();
                // We now parse an expression.
                // FIX: Should this be zero?
                let initializer = Some(self.parse_expression(0)?);
                self.next_token();
                if self.current_token_is(&Token::Semicolon) {
                    self.next_token();
                    return Ok(ast::Declaration { name, initializer });
                } else {
                    return Err(Error::UnexpectedToken {
                        message: Some("Within `parse_declaration`".into()),
                        expected: Token::Semicolon,
                        found: self.current_token.clone(),
                    });
                }
            }

            // The initializer is absent
            // We check if the ";" is present
            if self.current_token_is(&Token::Semicolon) {
                self.next_token();
                Ok(ast::Declaration {
                    name,
                    initializer: None,
                })
            } else {
                Err(Error::UnexpectedToken {
                    message: Some("Within `parse_declaration`".into()),
                    expected: Token::Semicolon,
                    found: self.current_token.clone(),
                })
            }
        } else {
            Err(Error::UnexpectedToken {
                message: Some("Within `parse_declaration`".into()),
                expected: Token::Int,
                found: self.current_token.clone(),
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
            Token::Assign => Ok(ast::BinaryOperator::Equal),
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
    /// Advances the token stream if the current token
    /// is an identifier.
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
    /// <exp> ::== <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
    fn parse_expression(&mut self, min_precedence: usize) -> Result<ast::Expression> {
        let mut left = self.parse_factor()?;

        let mut next_token = self.peek_token.clone();

        while self.is_binary_operator(&next_token) && next_token.precedence()? >= min_precedence {
            if matches!(next_token, Token::Assign) {
                // HACK: Is this correct?
                self.next_token();
                self.next_token();
                let right = self.parse_expression(next_token.precedence()?)?;
                left = ast::Expression::Assignment(Box::new(left), Box::new(right));
            } else if matches!(next_token, Token::QuestionMark) {
                self.next_token();
                let middle = self.parse_conditional_middle()?;
                self.next_token();
                let right = self.parse_expression(next_token.precedence()?)?;
                left = ast::Expression::Conditional {
                    condition: Box::new(left),
                    exp1: Box::new(middle),
                    exp2: Box::new(right),
                };
            } else {
                self.next_token();
                let operator = self.parse_binaryop()?;
                self.next_token();
                let right = Box::new(self.parse_expression(next_token.precedence()? + 1)?);
                left = ast::Expression::Binary(operator, Box::new(left), right);
            }
            next_token = self.peek_token.clone()
        }

        Ok(left)
    }

    /// This function just consumes a `?` token, then parses an expression
    /// (with the minimum precedence reset to 0) and finally
    /// consumes the `:` token.
    fn parse_conditional_middle(&mut self) -> Result<ast::Expression> {
        if self.current_token_is(&Token::QuestionMark) {
            self.next_token();
            let expression = self.parse_expression(0);
            if self.next_token_is(&Token::Colon) {
                self.next_token();
                Ok(expression?)
            } else {
                Err(Error::UnexpectedToken {
                    message: Some("Within `parse_conditional_middle`".into()),
                    expected: Token::Colon,
                    found: self.current_token.clone(),
                })
            }
        } else {
            Err(Error::UnexpectedToken {
                message: Some("Within `parse_conditional_statement`".into()),
                expected: Token::QuestionMark,
                found: self.current_token.clone(),
            })
        }
    }

    /// <factor> ::== <int> \ <identifier> \ <unop> <factor> \ "(" <exp> ")"
    fn parse_factor(&mut self) -> Result<ast::Expression> {
        match &self.current_token {
            // <int>
            Token::Constant(i) => Ok(ast::Expression::Constant(*i)),
            Token::Identifier(identifier) => {
                Ok(ast::Expression::Var(Identifier(identifier.clone())))
            }
            // If token is "~" or "-"
            // <unop> <factor>
            Token::Negation | Token::BitComp | Token::Not => {
                let operator = self.parse_unaryop()?;
                let inner_expression = self.parse_factor()?;

                Ok(ast::Expression::Unary(operator, Box::new(inner_expression)))
            }
            // "(" <exp> ")"
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
    ///             | <exp> ";"
    ///             | "if" "(" <exp> ")" <statement> ["else" <statement>]
    ///             | <block>
    ///             | "break" ;
    ///             | "continue" ;
    ///             | "while" "(" <exp> ")" <statement
    ///             | "do" <statement> "while" "(" <exp> ")" ";"
    ///             | "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
    ///             | ";"
    fn parse_statement(&mut self) -> Result<ast::Statement> {
        match &self.current_token {
            Token::Return => {
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
                        message: Some("Within `parse_statement`".into()),
                    })
                }
            }
            Token::Semicolon => Ok(ast::Statement::Null),
            Token::If => {
                self.next_token();

                if self.current_token_is(&Token::LParen) {
                    let condition = self.parse_expression(0)?;

                    if self.current_token_is(&Token::RParen) {
                        self.next_token();
                        let then = Box::new(self.parse_statement()?);

                        let else_statement: Option<Box<ast::Statement>> =
                            if self.current_token_is(&Token::Else) {
                                self.next_token();
                                Some(Box::new(self.parse_statement()?))
                            } else {
                                None
                            };

                        Ok(ast::Statement::If {
                            condition,
                            then,
                            else_statement,
                        })
                    } else {
                        Err(Error::UnexpectedToken {
                            message: Some("Within `parse_statement`".into()),
                            expected: Token::RParen,
                            found: self.current_token.clone(),
                        })
                    }
                } else {
                    Err(Error::UnexpectedToken {
                        message: Some("Within `parse_statement`".into()),
                        expected: Token::LParen,
                        found: self.current_token.clone(),
                    })
                }
            }
            Token::LBrace => {
                let block = self.parse_block()?;

                self.next_token();
                Ok(ast::Statement::Compound(block))
            }
            // "break" ;
            Token::Break => {
                self.next_token();

                if self.current_token_is(&Token::Semicolon) {
                    self.next_token();
                    Ok(ast::Statement::Break { label: None })
                } else {
                    Err(Error::UnexpectedToken {
                        message: Some("Within `parse_statement`".into()),
                        expected: Token::Semicolon,
                        found: self.current_token.clone(),
                    })
                }
            }
            // "continue" ";"
            Token::Continue => {
                self.next_token();

                if self.current_token_is(&Token::Semicolon) {
                    self.next_token();
                    Ok(ast::Statement::Continue { label: None })
                } else {
                    Err(Error::UnexpectedToken {
                        message: Some("Within `parse_statement`".into()),
                        expected: Token::Semicolon,
                        found: self.current_token.clone(),
                    })
                }
            }
            // "while" "(" <exp> ")" <statement>
            Token::While => {
                self.next_token();
                if self.current_token_is(&Token::LParen) {
                    let condition = self.parse_expression(0)?;
                    if self.current_token_is(&Token::RParen) {
                        self.next_token();
                        let body = Box::new(self.parse_statement()?);

                        Ok(ast::Statement::While {
                            condition,
                            body,
                            identifier: None,
                        })
                    } else {
                        self.error_expected(Token::RParen, Some("Within `parse_statement`".into()))
                    }
                } else {
                    self.error_expected(Token::LParen, Some("Within `parse_statement`".into()))
                }
            }
            _ => {
                let expression = self.parse_expression(0)?;

                self.next_token();

                if self.current_token_is(&Token::Semicolon) {
                    self.next_token();
                    Ok(ast::Statement::Expression(expression))
                } else {
                    Err(Error::UnexpectedToken {
                        expected: Token::Semicolon,
                        found: self.current_token.clone(),
                        message: Some("Within `parse_statement`".into()),
                    })
                }
            }
        }
    }

    fn error_expected(&self, expected: Token, message: Option<String>) -> Result<ast::Statement> {
        Err(Error::UnexpectedToken {
            message,
            expected,
            found: self.current_token.clone(),
        })
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
                | Token::Assign
                | Token::QuestionMark // This is a ternary op.
        )
    }
}
