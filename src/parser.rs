use crate::tokens::{Token, TokenType};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
    line: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0
        }
    }

    fn peek(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }

    fn peek_type(&self) -> Option<TokenType> {
        if let Some(t) = self.peek(0) {
            Some(t.token_type.clone())
        } else {
            None
        }
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn advance(&mut self) -> Option<&Token> {
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, t: TokenType, msg: &str) -> Result<Token, ParseError> {
        if let Some(_t) = self.tokens.get(self.current) {
            if _t.token_type == t {
                self.current += 1;
                return Ok(_t.clone());
            }
            println!("found {:?}", _t);
            return Err(ParseError {
                message: msg.to_string(),
                line: _t.line,
            })
        }
        Err(ParseError {
            message: msg.to_string(),
            line: 0,
        })
    }

    fn matches(&mut self, t: TokenType) -> Option<&Token> {
        if let Some(_t) = self.tokens.get(self.current) {
            if _t.token_type == t {
                self.current += 1;
                return Some(_t);
            }
        }
        None
    }
    fn m_matches(&mut self, t: Vec<TokenType>) -> Option<&Token> {
        if let Some(_t) = self.tokens.get(self.current) {
            for tt in t {
                if _t.token_type == tt {
                    self.current += 1;
                    return Some(_t);
                }
            }
        }
        None
    }

    // Program Structure
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut modules: Vec<ItemNode> = vec![];
        while let Some(tt) = self.peek_type() {
            let a = match tt {
                TokenType::EOF => break,
                _ => self.parse_top_level()
            }?;
            modules.push(a);
        }
        self.consume(TokenType::EOF, "program didn't end with EOF")?;
        Ok(Program {
            modules,
        })
    }

    pub fn parse_top_level(&mut self) -> Result<ItemNode, ParseError> {
        let mut attributes: Vec<Attribute> = vec![];
        while let Some(_q) = self.matches(TokenType::At) {
            attributes.push(self.parse_attribute()?);
        }

        if let Some(t) = self.peek(0) {
            let item_kind = match t.token_type {
                TokenType::Import => {
                    if !attributes.is_empty() {
                        return Err(ParseError {
                            message: "Imports cannot have attributes".to_string(),
                            line: t.line,
                        });
                    }
                    self.parse_import_decl()?
                },
                TokenType::Extern => self.parse_extern(attributes)?,
                TokenType::Fn => self.parse_fn_decl(attributes)?,
                TokenType::Struct => self.parse_struct_decl(attributes)?,
                TokenType::Enum => self.parse_enum_decl(attributes)?,
                TokenType::Const | TokenType::Var => self.parse_global_decl(attributes)?,
                _ => return Err(ParseError {
                    message: format!("Expected top-level declaration, found {:?}", t.token_type),
                    line: t.line
                }),
            };
            Ok(ItemNode {
                kind: item_kind,
                ty: None,
            })
        } else {
            Err(ParseError {
                message: "Expected top-level declaration, found EOF".to_string(),
                line: 0,
            })
        }
    }

    fn parse_attribute(&mut self) -> Result<Attribute, ParseError> {
        Ok(Attribute {
            name: self.consume(TokenType::Identifier, "expected identifier before attribute.")?.literal,
            args: {
                let mut args: Vec<ExprNode> = vec![];
                if let Some(_) = self.matches(TokenType::LeftParen) {
                    args = self.parse_arg_list()?;
                    self.consume(TokenType::RightParen, "expected ')'")?;
                }
                args
            },
        })
    }

    fn parse_import_decl(&mut self) -> Result<Item, ParseError> {
        self.consume(TokenType::Import, "Bug")?;
        let name = self.consume(TokenType::String, "need string after import")?.literal;
        self.consume(TokenType::Semicolon, "need ';' after import")?;
        Ok(Item::Import(name))
    }

    fn parse_extern(&mut self, attributes: Vec<Attribute>) -> Result<Item, ParseError> {
        self.consume(TokenType::Extern, "Bug")?;
        let abi = self.consume(TokenType::String, "expected a String after extern")?.literal;
        self.consume(TokenType::Fn, "expected Fn after extern api String")?;
        let name = self.consume(TokenType::Identifier, "need identifier after Fn")?.literal;
        self.consume(TokenType::LeftParen, "expected '('")?;
        let params = self.parse_param_list()?;
        self.consume(TokenType::RightParen, "expected ')'")?;
        let mut return_type = Type::Void;
        if let Some(_) = self.matches(TokenType::Arrow) {
            return_type = self.parse_type()?;
        }
        self.consume(TokenType::Semicolon, "expected ';' at end of extern")?;
        Ok(Item::Extern( ExternDecl {
            attributes,
            abi,
            name,
            params,
            return_type,
            is_varargs: false,
        }))

    }

    // Types
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if let Some(t) = self.advance() {
            match t.token_type {
                TokenType::U8 => Ok(Type::U8),
                TokenType::U16 => Ok(Type::U16),
                TokenType::U32 => Ok(Type::U32),
                TokenType::U64 => Ok(Type::U64),
                TokenType::I8 => Ok(Type::I8),
                TokenType::I32 => Ok(Type::I32),
                TokenType::I64 => Ok(Type::I64),
                TokenType::F32 => Ok(Type::F32),
                TokenType::F64 => Ok(Type::F64),
                TokenType::CHAR => Ok(Type::CHAR),
                TokenType::Void => Ok(Type::Void),
                TokenType::BOOL => Ok(Type::BOOL),
                TokenType::Identifier => Ok(Type::Named(t.literal.clone())),
                TokenType::Star => Ok(Type::Pointer(Box::new(self.parse_type()?))),
                TokenType::LeftBracket => {
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::RightBracket, "expected ']'")?;
                    let a = Type::Array(Box::new(self.parse_type()?), Box::new(expr));
                    Ok(a)
                },
                TokenType::Fn => {
                    self.consume(TokenType::LeftParen, "expected '('")?;
                    let params = self.parse_type_list()?;
                    let mut return_type = Type::Void;
                    if let Some(_) = self.matches(TokenType::Arrow) {
                        return_type = self.parse_type()?;
                    }
                    Ok(Type::Fn {
                        params,
                        ret: Box::new(return_type),
                    })
                },
                _ => Err(ParseError {
                    message: format!("Expected a type found {:?}", t.token_type),
                    line: t.line,
                })

            }
        } else {
            Err(ParseError {
                message: "Expected a type found EOF".to_string(),
                line: 0,
            })
        }
    }

    fn parse_type_list(&mut self) -> Result<Vec<Type>, ParseError> {
        let mut types: Vec<Type> = vec![];
        types.push(self.parse_type()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            types.push(self.parse_type()?);
        }
        Ok(types)
    }

    // Declarations

    fn parse_fn_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, ParseError> {
        self.consume(TokenType::Fn, "Expected function declaration")?;
        let name = self.consume(TokenType::Identifier, "need identifier after fn")?.literal;
        self.consume(TokenType::LeftParen, "expected '('")?;
        let mut params = vec![];
        if let Some(tt) = self.peek_type() {
            if tt != TokenType::RightParen {
                params = self.parse_param_list()?;
            }
        }
        self.consume(TokenType::RightParen, "expected ')'")?;
        let mut return_type = Type::Void;
        if let Some(_) = self.matches(TokenType::Arrow) {
            return_type = self.parse_type()?;
        }
        let body = self.parse_block()?;
        Ok(Item::Function(FnDecl {
            attributes,
            name,
            params,
            return_type,
            body: Box::new(body),
        }))
    }

    fn parse_param_list(&mut self) -> Result<Vec<(String, Type)>, ParseError> {
        let mut params: Vec<(String, Type)> = vec![];
        params.push(self.parse_param()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<(String, Type), ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected identifier")?.literal;
        self.consume(TokenType::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        Ok((name, ty))
    }

    fn parse_struct_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, ParseError> {
        self.consume(TokenType::Struct, "Expected struct declaration")?;
        let name = self.consume(TokenType::Identifier, "need identifier after struct")?.literal;
        self.consume(TokenType::LeftBrace, "Expected {")?;
        let mut fields: Vec<(String, Type)> = vec![];
        while let None = self.matches(TokenType::RightBrace) {
            fields.push(self.parse_struct_field()?);
        }
        Ok(Item::Struct(StructDecl {
            attributes,
            name,
            fields,
        }))
    }

    fn parse_struct_field(&mut self) -> Result<(String, Type), ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected identifier")?.literal;
        self.consume(TokenType::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        self.consume(TokenType::Semicolon, "Expected ;")?;
        Ok((name, ty))
    }

    fn parse_enum_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, ParseError> {
        self.consume(TokenType::Enum, "Expected enum declaration")?;
        let name = self.consume(TokenType::Identifier, "need identifier after enum")?.literal;
        self.consume(TokenType::LeftBrace, "Expected {")?;
        let mut variants: Vec<(String, Option<ExprNode>)> = vec![];
        variants.push(self.parse_enum_item()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            if let Some(tt) = self.peek_type() {
               if tt == TokenType::RightBrace { break };
            }
            variants.push(self.parse_enum_item()?);
        }
        self.consume(TokenType::RightBrace, "Expected }")?;
        Ok(Item::Enum(EnumDecl {
            attributes,
            name,
            variants,
        }))
    }

    fn parse_enum_item(&mut self) -> Result<(String, Option<ExprNode>), ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected identifier")?.literal;
        let mut expr: Option<ExprNode> = None;
        if let Some(_) = self.matches(TokenType::Equal) {
            expr = Some(self.parse_expression()?);
        }
        Ok((name, expr))
    }

    fn parse_global_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, ParseError> {
        let mut is_const: bool = false;
        if let Some(t) = self.peek(0) {
            is_const = match t.token_type {
                TokenType::Const => true,
                TokenType::Var => true,
                _ => {
                    return Err(ParseError {
                        message: "Bug".to_string(),
                        line: t.line,
                    });
                },
            }
        } else {
            return Err(ParseError {
                message: "Bug".to_string(),
                line: 0,
            });
        }
        self.current += 1; // const|var
        let name = self.consume(TokenType::Identifier, "need identifier after global declaration")?.literal;
        self.consume(TokenType::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        let mut init: Option<ExprNode> = None;
        if let Some(_) = self.matches(TokenType::Equal) {
            init = Some(self.parse_expression()?);
        }
        self.consume(TokenType::Semicolon, "Expected ;")?;
        Ok(Item::Global(GlobalDecl {
            attributes,
            name,
            ty,
            init,
            is_const,
        }))
    }

    // Statements
    fn parse_block(&mut self) -> Result<StmtNode, ParseError> {
        self.consume(TokenType::LeftBrace, "Expected {")?;
        let mut stmts: Vec<StmtNode> = vec![];
        while let None = self.matches(TokenType::RightBrace) {
            stmts.push(self.parse_statement()?);
        }
        Ok(StmtNode::new(Stmt::Block(stmts)))
    }

    fn parse_statement(&mut self) -> Result<StmtNode, ParseError> {
        if let Some(t) = self.peek(0) {
            match t.token_type {
                TokenType::Let | TokenType::Var => self.parse_var_decl_stmt(),
                TokenType::If => self.parse_if_stmt(),
                TokenType::While => self.parse_while_stmt(),
                TokenType::Return => self.parse_return_stmt(),
                TokenType::Break => {
                    self.current += 1;
                    self.consume(TokenType::Semicolon, "Expected ;")?;
                    Ok(StmtNode::new(Stmt::Break))
                }
                TokenType::Continue => {
                    self.current += 1;
                    self.consume(TokenType::Semicolon, "Expected ;")?;
                    Ok(StmtNode::new(Stmt::Continue))
                }
                _ => self.parse_assignment_or_expr_stmt(),
            }
        } else {
            Err(ParseError {
                message: "Unexpected EOF.".to_string(),
                line: 0,
            })
        }
    }

    fn parse_var_decl_clause(&mut self) -> Result<StmtNode, ParseError> {
        let mut is_mutable: bool = false;
        if let Some(t) = self.peek(0) {
            is_mutable = match t.token_type {
                TokenType::Let => false,
                TokenType::Var => true,
                _ => {
                    return Err(ParseError {
                        message: "Bug".to_string(),
                        line: t.line,
                    });
                },
            }
        } else {
            return Err(ParseError {
                message: "Bug".to_string(),
                line: 0,
            });
        }
        self.current += 1; // let|var
        let name = self.consume(TokenType::Identifier, "need identifier after global declaration")?.literal;
        self.consume(TokenType::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        let mut init: Option<ExprNode> = None;
        if let Some(_) = self.matches(TokenType::Equal) {
            init = Some(self.parse_expression()?);
        }
        Ok(StmtNode::new(Stmt::VarDecl {
            is_mutable,
            name,
            ty,
            init,
        }))
    }

    fn parse_var_decl_stmt(&mut self) -> Result<StmtNode, ParseError> {
        let var_decl_clause = self.parse_var_decl_clause()?;
        self.consume(TokenType::Semicolon, "Expectedhh ;")?;
        Ok(var_decl_clause)
    }

    fn parse_assignment_or_expr_stmt(&mut self) -> Result<StmtNode, ParseError> {
        let target = self.parse_expression()?;
        if let Some(_) = self.matches(TokenType::Equal) {
            let value = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ;")?;
            Ok(StmtNode::new(Stmt::Assign {
                target,
                value
            }))
        } else {
            self.consume(TokenType::Semicolon, "Expected ;")?;
            Ok(StmtNode::new(Stmt::Expression(target)))
        }
    }

    fn parse_assignment_clause(&mut self) -> Result<StmtNode, ParseError> {
        let target = self.parse_expression()?;
        self.consume(TokenType::Equal, "Expected =")?;
        let value = self.parse_expression()?;
        Ok(StmtNode::new(Stmt::Assign {
            target,
            value
        }))
    }

    fn parse_assignment_stmt(&mut self) -> Result<StmtNode, ParseError> {
        let assignment_clause = self.parse_assignment_clause()?;
        self.consume(TokenType::Semicolon, "Expected ;")?;
        Ok(assignment_clause)
    }

    fn parse_if_stmt(&mut self) -> Result<StmtNode, ParseError> {
        self.consume(TokenType::If, "Bug in If")?;
        let condition = self.parse_expression()?;
        let then_branch = Box::new(self.parse_block()?);
        let mut else_branch: Option<Box<StmtNode>> = None;
        if let Some(_) = self.matches(TokenType::Else) {
             if let Some(t) = self.peek(0) {
                else_branch = match t.token_type {
                    TokenType::If => Some(Box::new(self.parse_if_stmt()?)),
                    TokenType::LeftBrace => Some(Box::new(self.parse_block()?)),
                    _ => { return  Err(ParseError {
                        message: format!("Unexpected token {:?}", t),
                        line: t.line,
                    }) }
                };
            } else {
                return  Err(ParseError {
                    message: "Unexpected EOF".to_string(),
                    line: 0,
                })
            }
        }

        Ok(StmtNode::new(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn parse_while_stmt(&mut self) -> Result<StmtNode, ParseError> {
        self.consume(TokenType::While, "Bug in While")?;
        let condition = self.parse_expression()?;
        let body = Box::new(self.parse_block()?);
        Ok(StmtNode::new(Stmt::While {
            condition,
            body
        }))
    }

    fn parse_return_stmt(&mut self) -> Result<StmtNode, ParseError> {
        self.consume(TokenType::Return, "Bug in Return")?;
        let mut expr: Option<ExprNode> = None;
        if let None = self.matches(TokenType::Semicolon) {
            expr = Some(self.parse_expression()?);
            self.consume(TokenType::Semicolon, "Expected ;")?;
        }
        Ok(StmtNode::new(Stmt::Return(expr)))
    }

    // Expressions
    fn parse_expression(&mut self) -> Result<ExprNode, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_logical_and()?;
        while let Some(t) = self.matches(TokenType::PipePipe).cloned() {
            let rhs = self.parse_logical_and()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::Or,
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_equality()?;
        while let Some(t) = self.matches(TokenType::AmpersandAmpersand).cloned() {
            let rhs = self.parse_equality()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::And,
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_comparison()?;
        while let Some(t) = self.m_matches(vec![TokenType::EqualEqual, TokenType::BangEqual]).cloned() {
            let rhs = self.parse_comparison()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: { if t.token_type == TokenType::BangEqual { BinaryOp::Neq } else { BinaryOp::Eq }},
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_bitwise_or()?;
        while let Some(t) = self.m_matches(vec![TokenType::Less, TokenType::Greater, TokenType::LessEqual, TokenType::GreaterEqual]).cloned() {
            let rhs = self.parse_bitwise_or()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: {
                    match t.token_type {
                        TokenType::Less => BinaryOp::Lt,
                        TokenType::Greater => BinaryOp::Gt,
                        TokenType::LessEqual => BinaryOp::Le,
                        _ => BinaryOp::Ge, // GreaterEqual
                    }
                },
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_bitwise_xor()?;
        while let Some(t) = self.matches(TokenType::Pipe).cloned() {
            let rhs = self.parse_bitwise_xor()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::BitOr,
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_bitwise_and()?;
        while let Some(t) = self.matches(TokenType::Caret).cloned() {
            let rhs = self.parse_bitwise_and()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::BitXor,
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_shift()?;
        while let Some(t) = self.matches(TokenType::Ampersand).cloned() {
            let rhs = self.parse_shift()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::BitAnd,
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_term()?;
        while let Some(t) = self.m_matches(vec![TokenType::LessLess, TokenType::GreaterGreater]).cloned() {
            let rhs = self.parse_term()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: { if t.token_type == TokenType::LessLess { BinaryOp::Shl } else { BinaryOp::Shr }},
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_factor()?;
        while let Some(t) = self.m_matches(vec![TokenType::Plus, TokenType::Minus]).cloned() {
            let rhs = self.parse_factor()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: { if t.token_type == TokenType::Plus { BinaryOp::Add } else { BinaryOp::Sub }},
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ExprNode, ParseError> {
        let mut expr = self.parse_cast()?;
        while let Some(t) = self.m_matches(vec![TokenType::Star, TokenType::Slash, TokenType::Percent]).cloned() {
            let rhs = self.parse_cast()?;
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: {
                    match t.token_type {
                        TokenType::Star => BinaryOp::Mul,
                        TokenType::Slash => BinaryOp::Div,
                        _ => BinaryOp::Mod, // Percent
                    }
                },
                rhs: Box::new(rhs)
            })
        }
        Ok(expr)
    }

    fn parse_cast(&mut self) -> Result<ExprNode, ParseError> {
        let expr = self.parse_unary()?;
        if let Some(t) = self.matches(TokenType::As) {
            let target = self.parse_type()?;
            Ok(ExprNode::new(Expr::Cast {
                expr: Box::new(expr),
                target,
            }))
        } else {
            Ok(expr)
        }
    }

    fn parse_unary(&mut self) -> Result<ExprNode, ParseError> {
        if let Some(tt) = self.peek_type() {
            match tt {
                TokenType::Bang => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::Not,
                        rhs: Box::new(self.parse_unary()?),
                    }))
                },
                TokenType::Minus => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::Neg,
                        rhs: Box::new(self.parse_unary()?),
                    }))
                },
                TokenType::Star => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::Deref,
                        rhs: Box::new(self.parse_unary()?),
                    }))
                },
                TokenType::Ampersand => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::AddressOf,
                        rhs: Box::new(self.parse_unary()?),
                    }))
                },
                TokenType::SizeOf => {
                    self.current += 1;
                    self.consume(TokenType::LeftParen, "Expected (")?;
                    let current = self.current; // check_point
                    let target: SizeOfTarget;
                    let ty = self.parse_type();
                    if ty.is_err() {
                        self.current = current; // Rollback
                        target = SizeOfTarget::Expr(Box::new(self.parse_expression()?));
                    } else {
                        target = SizeOfTarget::Type(ty?);
                    }
                    self.consume(TokenType::RightParen, "Expected ')'")?;
                    Ok(ExprNode::new(Expr::SizeOf { target }))
                },
                _ => self.parse_postfix(),
            }
        } else {
            todo!()
        }
    }

    fn parse_postfix(&mut self) -> Result<ExprNode, ParseError> {
        let expr = self.parse_primary()?;
        if let Some(t) = self.peek(0) {
            match t.token_type {
                TokenType::LeftBracket => {
                    self.current += 1;
                    let idx = self.parse_expression()?;
                    self.consume(TokenType::RightBracket, "Expected ]")?;
                    Ok(ExprNode::new(Expr::Index {
                        array: Box::new(expr),
                        index: Box::new(idx),
                    }))
                },
                TokenType::LeftParen => {
                    self.current += 1;
                    let args = self.parse_arg_list()?;
                    self.consume(TokenType::RightParen, "Expected )")?;
                    Ok(ExprNode::new(Expr::Call {
                        callee: Box::new(expr),
                        args,
                    }))
                },
                TokenType::Dot => {
                    self.current += 1;
                    let member = self.consume(TokenType::Identifier, "Expected Identifier")?.literal;
                    Ok(ExprNode::new(Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: false,
                    }))
                },
                TokenType::Arrow => {
                    self.current += 1;
                    let member = self.consume(TokenType::Identifier, "Expected Identifier")?.literal;
                    Ok(ExprNode::new(Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: true,
                    }))
                }
                _ => Ok(expr)
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_postfix_op(&mut self) -> Result<ExprNode, ParseError> {
        todo!()
    }

    fn parse_primary(&mut self) -> Result<ExprNode, ParseError> {
        if let Some(t) = self.advance() {
            match t.token_type {
                TokenType::Int => {
                    Ok(ExprNode::new(Expr::LiteralInt(t.literal.parse().unwrap())))
                },
                TokenType::Float => {
                    Ok(ExprNode::new(Expr::LiteralFloat(t.literal.parse().unwrap())))
                },
                TokenType::String => Ok(ExprNode::new(Expr::LiteralString(t.literal.clone()))),
                TokenType::Char => Ok(ExprNode::new(Expr::LiteralString(t.literal.clone()))),
                TokenType::True => Ok(ExprNode::new(Expr::LiteralBool(true))),
                TokenType::False => Ok(ExprNode::new(Expr::LiteralBool(false))),
                TokenType::Null => Ok(ExprNode::new(Expr::Null)),
                TokenType::Identifier => Ok(ExprNode::new(Expr::Identifier(t.literal.clone()))),
                TokenType::LeftParen => {
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::RightParen, "Expected ')'")?;
                    Ok(expr)
                },
                _ => {self.current -= 1;self.parse_struct_init()}
            }
        } else {
            todo!()
        }
    }

    fn parse_struct_init(&mut self) -> Result<ExprNode, ParseError> {
        let name = self.consume(TokenType::Identifier, "expected identifier before struct init.")?;
        let mut fields: Vec<(String, ExprNode)> = vec![];
        self.consume(TokenType::LeftBrace, "expected '{'")?;
        fields.push((
            self.consume(TokenType::Identifier, "struct must have at least one field initialed")?.literal,
            self.parse_expression()?
        ));
        Ok(ExprNode::new(Expr::StructInit {
            name: name.literal,
            fields,
        }))
    }

    fn parse_arg_list(&mut self) -> Result<Vec<ExprNode>, ParseError> {
        let mut args: Vec<ExprNode> = vec![];
        args.push(self.parse_expression()?);
        while let Some(t) = self.matches(TokenType::Comma).cloned() {
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }
}