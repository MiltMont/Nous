use std::{
    collections::VecDeque,
    fmt::Debug,
    fs::{self},
    path::PathBuf,
};

use logos::{Lexer, Logos};

use crate::{
    ast::{self, Block, FunctionDeclaration, Identifier},
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

    fn expect_token_then<F, T>(
        &mut self,
        expected: Token,
        context: &'static str,
        then: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        if self.current_token_is(&expected) {
            self.next_token();
            then(self)
        } else {
            self.error_expected(expected, context.into())
        }
    }

    fn expect_sequence_then<F, T>(
        &mut self,
        expected: &[Token],
        context: &'static str,
        then: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        for expected_token in expected {
            if !self.current_token_is(expected_token) {
                return self.error_expected(expected_token.clone(), context.into());
            }
            self.next_token();
        }
        then(self)
    }

    fn parse_delimited<F, T>(
        &mut self,
        opening: Token,
        closing: Token,
        context: &'static str,
        parse_content: F,
    ) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
        T: Debug,
    {
        if !self.current_token_is(&opening) {
            return self.error_expected(opening, context.into());
        }

        self.next_token();

        // Parse the content between the delimiters.
        let result = parse_content(self)?;

        if !self.current_token_is(&closing) {
            return self.error_expected(closing, context.into());
        }

        self.next_token();

        Ok(result)
    }

    fn parse_parenthesized<F, T>(&mut self, context: &'static str, parse_content: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
        T: Debug,
    {
        self.parse_delimited(Token::LParen, Token::RParen, context, parse_content)
    }

    fn parse_braced<F, T>(&mut self, context: &'static str, parse_content: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
        T: Debug,
    {
        self.parse_delimited(Token::LBrace, Token::RBrace, context, parse_content)
    }

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

    /// Parses:
    /// `<program> ::== { <function-declaration> }`
    fn parse_program(&mut self) -> Result<ast::Program> {
        let mut functions: Vec<ast::FunctionDeclaration> = Vec::new();
        // Parses function declarations until token stream
        // is empty.
        while !self.tokens.is_empty() {
            functions.push(self.parse_function_declaration()?);
        }

        Ok(ast::Program(functions))
    }

    /// Parses:
    /// `<variable-declaration> ::= "int" <identifier> [ "=" <exp> ] ";"`
    fn parse_variable_declaration(&mut self) -> Result<ast::VariableDeclaration> {
        let name = self.expect_token_then(
            Token::Int,
            "Within `parse_variable_declaration`, parsing identifier",
            |parser| parser.parse_identifier(),
        )?;

        if self.current_token_is(&Token::Assign) {
            let initializer = Some(self.parse_delimited(
                Token::Assign,
                Token::Semicolon,
                "Within `parse_variable_declaration`, parsing initializer",
                |parser| parser.parse_expression(0),
            )?);

            Ok(ast::VariableDeclaration { name, initializer })
        } else if self.current_token_is(&Token::Semicolon) {
            self.next_token();
            Ok(ast::VariableDeclaration {
                name,
                initializer: None,
            })
        } else {
            Err(Error::UnexpectedToken {
                message: "Within `parse_variable_declaration`, expected Initializer of Semicolon"
                    .into(),
                expected: Token::Semicolon,
                found: self.current_token.clone(),
            })
        }
    }

    /// `<function-declaration> ::= "int" <identifier> "(" <param-list> ")" ( <block> | ";")`
    fn parse_function_declaration(&mut self) -> Result<ast::FunctionDeclaration> {
        let name = self.expect_token_then(
            Token::Int,
            "Within `parse_function_declaration`, parsing identifier",
            |parser| parser.parse_identifier(),
        )?;

        let parameters = self.parse_delimited(
            Token::LParen,
            Token::RParen,
            "Within `parse_function_declaration`, parsing parameter list",
            |parser| parser.parse_param_list(),
        )?;

        // We now have either a block or a Semicolon
        if self.current_token_is(&Token::Semicolon) {
            Ok(FunctionDeclaration {
                name,
                parameters,
                body: None,
            })
        } else {
            let body = Some(self.parse_block()?);

            Ok(FunctionDeclaration {
                name,
                parameters,
                body,
            })
        }
    }

    /// `<param-list> ::= "void" | "int" <identifier> { "," "int" <identifier> }`
    fn parse_param_list(&mut self) -> Result<Vec<Identifier>> {
        if self.current_token_is(&Token::Void) {
            self.next_token();
            return Ok(vec![]);
        }

        let mut params: Vec<Identifier> = Vec::new();
        let name = self.expect_token_then(Token::Int, "Within `parse_param_list`", |parser| {
            parser.parse_identifier()
        })?;

        params.push(name);

        while !self.current_token_is(&Token::RParen) {
            params.push(self.expect_sequence_then(
                &[Token::Comma, Token::Int],
                "Within closure in `parse_param_list`",
                |parser| parser.parse_identifier(),
            )?);
        }

        Ok(params)
    }

    /// <block> ::= "{" { <block-item> } "}"
    fn parse_block(&mut self) -> Result<Block> {
        self.parse_braced("Within `parse_block`", |parser| {
            let mut blocks = Vec::new();

            // FIX: What happens if we dont have an RBrace?
            while !parser.current_token_is(&Token::RBrace) {
                blocks.push(parser.parse_block_item()?);
            }

            Ok(Block(blocks))
        })
    }

    /// <block-item> ::== <statement> | <declaration>
    fn parse_block_item(&mut self) -> Result<ast::BlockItem> {
        // We need a way to tell wether the current block
        // item is a statement or a declaration.
        // To do this, we look at the first token; if it is
        // `Token::Int`, then it's a declaration, otherwise
        // it's a statement.
        if self.current_token_is(&Token::Int) {
            // Now we distinguish between variable or function declarations.
            Ok(ast::BlockItem::D(self.parse_declaration()?))
        } else {
            // This is a statement
            Ok(ast::BlockItem::S(self.parse_statement()?))
        }
    }

    // Distinguishes between variable and function declarations.
    //
    // variable = "int" identifier ["=" exp] ";"
    //
    // or
    //
    // function = "int" identifier "("
    fn parse_declaration(&mut self) -> Result<ast::Declaration> {
        dbg!(&self.tokens);
        if let Some(third_token) = self.tokens.get(1) {
            match third_token {
                Token::LParen => Ok(ast::Declaration::FuncDecl(
                    self.parse_function_declaration()?,
                )),
                _ => Ok(ast::Declaration::VarDecl(
                    self.parse_variable_declaration()?,
                )),
            }
        } else {
            panic!("Something weird is going on here")
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
            Ok(s.into())
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

        let mut next_token = self.current_token.clone();

        while self.is_binary_operator(&next_token) && next_token.precedence()? >= min_precedence {
            if matches!(next_token, Token::Assign) {
                // HACK: Is this correct?
                self.next_token();
                let right = self.parse_expression(next_token.precedence()?)?;
                left = ast::Expression::Assignment(Box::new(left), Box::new(right));
            } else if matches!(next_token, Token::QuestionMark) {
                let middle = self.parse_conditional_middle()?;
                let right = self.parse_expression(next_token.precedence()?)?;
                left = ast::Expression::Conditional {
                    condition: Box::new(left),
                    exp1: Box::new(middle),
                    exp2: Box::new(right),
                };
            } else {
                let operator = self.parse_binaryop()?;
                self.next_token();
                let right = Box::new(self.parse_expression(next_token.precedence()? + 1)?);
                left = ast::Expression::Binary(operator, Box::new(left), right);
            }
            next_token = self.current_token.clone()
        }

        Ok(left)
    }

    /// This function just consumes a `?` token, then parses an expression
    /// (with the minimum precedence reset to 0) and finally
    /// consumes the `:` token.
    ///
    /// '?' exp ':'
    fn parse_conditional_middle(&mut self) -> Result<ast::Expression> {
        self.parse_delimited(
            Token::QuestionMark,
            Token::Colon,
            "Within `parse_conditional_middle`",
            |parser| parser.parse_expression(0),
        )
    }

    fn parse_constant(&mut self) -> Result<ast::Expression> {
        if let Token::Constant(x) = self.current_token {
            self.next_token();
            Ok(ast::Expression::Constant(x))
        } else {
            todo!()
        }
    }

    /// This consumes all of the tokens in the grammar.
    ///
    /// <factor> ::== <int> | <identifier> | <unop> <factor> | "(" <exp> ")"
    ///             | <identifier> "(" [<argument-list>] ")"
    fn parse_factor(&mut self) -> Result<ast::Expression> {
        match &self.current_token {
            // <int>
            Token::Constant(_) => self.parse_constant(),
            Token::Identifier(_) => {
                if !self.next_token_is(&Token::LParen) {
                    Ok(ast::Expression::Var(self.parse_identifier()?))
                } else {
                    let name = self.parse_identifier()?;
                    let arguments = self
                        .parse_parenthesized("Within `parse_factor`", |parser| {
                            parser.parse_argument_list()
                        })?;

                    Ok(ast::Expression::FunctionCall { name, arguments })
                }
            }
            // If token is "~" or "-"
            // <unop> <factor>
            Token::Negation | Token::BitComp | Token::Not => {
                let operator = self.parse_unaryop()?;
                let inner_expression = self.parse_factor()?;

                Ok(ast::Expression::Unary(operator, Box::new(inner_expression)))
            }
            // "(" <exp> ")"
            Token::LParen => self
                .parse_parenthesized("Within `parse_factor`, parsing '(' <exp> ')'", |parser| {
                    parser.parse_expression(0)
                }),
            _ => Err(Error::MalformedFactor {
                missing: None,
                found: self.current_token.clone(),
            }),
        }
    }

    /// `<argument-list> ::= <exp> { "," <exp> }`
    fn parse_argument_list(&mut self) -> Result<Vec<ast::Expression>> {
        let mut arguments: Vec<ast::Expression> = Vec::new();

        arguments.push(self.parse_expression(0)?);
        self.next_token();

        while !self.current_token_is(&Token::RParen) {
            arguments.push(self.expect_token_then(
                Token::Comma,
                "Within `parse_argument_list`",
                |parser| {
                    let result = parser.parse_expression(0);
                    parser.next_token();
                    result
                },
            )?)
        }

        Ok(arguments)
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
            // "return" <exp> ";"
            Token::Return => {
                let expression = self.parse_delimited(
                    Token::Return,
                    Token::Semicolon,
                    "Within `parse_statement`, parsing RETURN",
                    |parser| parser.parse_expression(0),
                )?;

                Ok(ast::Statement::Return(expression))
            }
            Token::Semicolon => Ok(ast::Statement::Null),
            // "if" "(" <exp> ")" <statement> ["else" <statement>]
            Token::If => {
                self.next_token();

                let condition = self
                    .parse_parenthesized("Within `parse_statement`, parsing IF", |parser| {
                        parser.parse_expression(0)
                    })?;

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
                        message: Some("Within `parse_statement`, parsing Break"),
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
                        message: Some("Within `parse_statement`"),
                        expected: Token::Semicolon,
                        found: self.current_token.clone(),
                    })
                }
            }
            // "while" "(" <exp> ")" <statement>
            Token::While => {
                self.next_token();
                let condition = self.parse_parenthesized(
                    "Within `parse_statement`, parsing WHILE",
                    |parser| {
                        let result = parser.parse_expression(0);
                        parser.next_token();
                        result
                    },
                )?;

                let body = Box::new(self.parse_statement()?);

                Ok(ast::Statement::While {
                    condition,
                    body,
                    identifier: None,
                })
            }
            // "do" <statement> "while" "(" <exp> ")" ";"
            Token::Do => {
                self.next_token();

                let body = Box::new(self.parse_statement()?);

                let condition =
                    self.parse_parenthesized("Within `parse_statement`, parsing DO", |parser| {
                        let result = parser.parse_expression(0);
                        parser.next_token();
                        result
                    })?;

                Ok(ast::Statement::While {
                    condition,
                    body,
                    identifier: None,
                })
            }
            // "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
            Token::For => {
                self.next_token();
                if self.current_token_is(&Token::LParen) {
                    self.next_token();
                    let initializer = self.parse_for_init()?;
                    let condition = self.parse_optional_expression(&Token::Semicolon)?;
                    let post = self.parse_optional_expression(&Token::RParen)?;
                    let body = Box::new(self.parse_statement()?);

                    Ok(ast::Statement::For {
                        initializer,
                        condition,
                        post,
                        body,
                        identifier: None,
                    })
                } else {
                    self.error_expected(Token::LParen, Some("Within `parse_statement`"))
                }
            }
            _ => {
                let expression = self.parse_expression(0)?;
                dbg!(&expression);

                if self.current_token_is(&Token::Semicolon) {
                    self.next_token();
                    Ok(ast::Statement::Expression(expression))
                } else {
                    Err(Error::UnexpectedToken {
                        expected: Token::Semicolon,
                        found: self.current_token.clone(),
                        message: Some("Within `parse_statement`, parsing <exp> ';' "),
                    })
                }
            }
        }
    }

    /// If a delimiter is found then it is skiped and returns `None`.
    /// If no delimiter is found then we parse the existing expression
    /// and advance the token stream until we skip the delimiter.
    fn parse_optional_expression(&mut self, delimiter: &Token) -> Result<Option<ast::Expression>> {
        if self.current_token_is(delimiter) {
            self.next_token();
            Ok(None)
        } else {
            let expression = self.parse_expression(0)?;
            if self.current_token_is(delimiter) {
                self.next_token();
                Ok(Some(expression))
            } else {
                self.error_expected(delimiter.clone(), Some("Within `parse_optional_expression"))
            }
        }
    }

    /// `<for-init> ::= <variable-declaration> | [ <exp> ] ";"`
    fn parse_for_init(&mut self) -> Result<ast::ForInit> {
        if self.current_token_is(&Token::Int) {
            Ok(ast::ForInit::InitDecl(self.parse_variable_declaration()?))
        } else {
            Ok(ast::ForInit::InitExp(
                self.parse_optional_expression(&Token::Semicolon)?,
            ))
        }
    }

    /// Returns an error message whenever an expected token is not found.
    fn error_expected<T>(&self, expected: Token, message: Option<&'static str>) -> Result<T> {
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
