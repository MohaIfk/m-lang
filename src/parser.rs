use crate::tokens::{Token, TokenType};
use crate::ast::*;
use crate::error::{Span, CompilerError};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    file_source: &'a String,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, file_source: &'a String) -> Self {
        Self {
            tokens,
            current: 0,
            file_source
        }
    }

    fn peek(&self, n: usize) -> Option<Token> {
        self.tokens.get(self.current + n).cloned()
    }

    fn peek_type(&self) -> Option<TokenType> {
        if let Some(t) = self.peek(0) {
            Some(t.token_type.clone())
        } else {
            None
        }
    }

    fn previous(&self) -> Option<Token> {
        self.tokens.get(self.current - 1).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        self.current += 1;
        self.previous()
    }

    fn create_compiler_error(&self, message: String, span: Span) -> CompilerError<'a> {
        CompilerError::new(message, span, self.file_source)
    }

    fn consume(&mut self, t: TokenType, msg: &str) -> Result<Token, CompilerError<'a>> {
        if let Some(_t) = self.tokens.get(self.current) {
            if _t.token_type == t {
                self.current += 1;
                return Ok(_t.clone());
            }
            return Err(self.create_compiler_error(msg.to_string(), _t.span.clone()))
        }
        Err(self.create_compiler_error(msg.to_string(), Span::default()))
    }

    fn matches(&mut self, t: TokenType) -> Option<Token> {
        if let Some(_t) = self.tokens.get(self.current) {
            if _t.token_type == t {
                self.current += 1;
                return Some(_t.clone());
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
    pub fn parse_program(&mut self) -> Result<Program, CompilerError<'a>> {
        let mut modules: Vec<ItemNode> = vec![];
        while let Some(tt) = self.peek_type() {
            let a = match tt {
                TokenType::EOF => break,
                _ => self.parse_top_level()
            }?;
            modules.push(a);
        }
        self.consume(TokenType::EOF, "Unexpected token after end of main module. Expected EOF.")?;
        Ok(Program {
            modules,
        })
    }

    fn parse_top_level(&mut self) -> Result<ItemNode, CompilerError<'a>> {
        let mut attributes: Vec<Attribute> = vec![];
        while let Some(_q) = self.matches(TokenType::At) {
            attributes.push(self.parse_attribute()?);
        }

        if let Some(t) = self.peek(0) {
            let item_kind = match t.token_type {
                TokenType::Import => {
                    if !attributes.is_empty() {
                        Err(self.create_compiler_error("Attributes are not allowed on import statements.".to_string(), t.span.clone()))?
                    }
                    self.parse_import_decl()?
                },
                TokenType::Extern => self.parse_extern(attributes)?,
                TokenType::Fn => self.parse_fn_decl(attributes)?,
                TokenType::Struct => self.parse_struct_decl(attributes)?,
                TokenType::Enum => self.parse_enum_decl(attributes)?,
                TokenType::Const | TokenType::Var => self.parse_global_decl(attributes)?,
                _ => return Err(self.create_compiler_error(format!("Expected a top-level declaration (fn, struct, enum, const, var, extern, import). Found '{:?}'.", t.token_type), t.span.clone())),
            };
            Ok(ItemNode {
                kind: item_kind,
                span: t.span.clone(),
                ty: None,
            })
        } else {
            Err(self.create_compiler_error("Expected a top-level declaration (fn, struct, enum, const, var, extern, import). Found EOF".to_string(), Span::default()))
        }
    }

    fn parse_attribute(&mut self) -> Result<Attribute, CompilerError<'a>> {
        Ok(Attribute {
            name: self.consume(TokenType::Identifier, "Expected attribute name (identifier) after '@'.")?.literal,
            args: {
                let mut args: Vec<ExprNode> = vec![];
                if let Some(_) = self.matches(TokenType::LeftParen) {
                    args = self.parse_arg_list()?;
                    self.consume(TokenType::RightParen, "Expected ')' to close attribute argument list.")?;
                }
                args
            },
        })
    }

    fn parse_import_decl(&mut self) -> Result<Item, CompilerError<'a>> {
        self.consume(TokenType::Import, "Bug")?;
        let name = self.consume(TokenType::String, "Expected string literal after 'import' keyword.")?.literal;
        self.consume(TokenType::Semicolon, "Expected ';' after import declaration.")?;
        Ok(Item::Import(name))
    }

    fn parse_extern(&mut self, attributes: Vec<Attribute>) -> Result<Item, CompilerError<'a>> {
        let mut span = self.consume(TokenType::Extern, "Bug")?.span;
        let abi = self.consume(TokenType::String, "Expected ABI string literal (e.g., \"C\") after 'extern'.")?.literal;
        self.consume(TokenType::Fn, "Expected 'fn' keyword after extern ABI.")?;
        let name = self.consume(TokenType::Identifier, "Expected function name after 'fn'.")?.literal;
        self.consume(TokenType::LeftParen, "Expected '(' to start parameter list.")?;
        let params = self.parse_param_list()?;
        self.consume(TokenType::RightParen, "Expected ')' to end parameter list.")?;
        let mut return_type = TypeSpec::Void;
        if let Some(_) = self.matches(TokenType::Arrow) {
            return_type = self.parse_type()?;
        }
        let span_end = self.consume(TokenType::Semicolon, "Expected ';' after extern function declaration.")?.span;
        span = Span::sum(span, span_end);
        Ok(Item::Extern( ExternDecl {
            attributes,
            abi,
            name,
            params,
            return_type,
            is_varargs: false,
            span,
        }))

    }

    // Types
    fn parse_type(&mut self) -> Result<TypeSpec, CompilerError<'a>> {
        if let Some(t) = self.advance() {
            match t.token_type {
                TokenType::U8 => Ok(TypeSpec::U8),
                TokenType::U16 => Ok(TypeSpec::U16),
                TokenType::U32 => Ok(TypeSpec::U32),
                TokenType::U64 => Ok(TypeSpec::U64),
                TokenType::I8 => Ok(TypeSpec::I8),
                TokenType::I16 => Ok(TypeSpec::I16),
                TokenType::I32 => Ok(TypeSpec::I32),
                TokenType::I64 => Ok(TypeSpec::I64),
                TokenType::F32 => Ok(TypeSpec::F32),
                TokenType::F64 => Ok(TypeSpec::F64),
                TokenType::Usize => Ok(TypeSpec::Usize),
                TokenType::Isize => Ok(TypeSpec::Isize),
                TokenType::CHAR => Ok(TypeSpec::CHAR),
                TokenType::Void => Ok(TypeSpec::Void),
                TokenType::BOOL => Ok(TypeSpec::BOOL),
                TokenType::Identifier => Ok(TypeSpec::Named(t.literal.clone())),
                TokenType::Star => Ok(TypeSpec::Pointer(Box::new(self.parse_type()?))),
                TokenType::LeftBracket => {
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::RightBracket, "Expected ']' to close array type dimension.")?;
                    let a = TypeSpec::Array(Box::new(self.parse_type()?), Box::new(expr));
                    Ok(a)
                },
                TokenType::Fn => {
                    self.consume(TokenType::LeftParen, "Expected '(' to start function type parameter list.")?;
                    let mut params: Vec<TypeSpec> = vec![];
                    if self.matches(TokenType::RightParen).is_none() {
                        params = self.parse_type_list()?;
                        self.consume(TokenType::RightParen, "Expected ')' to end function type parameter list.")?;
                    }
                    let mut return_type = TypeSpec::Void;
                    if let Some(_) = self.matches(TokenType::Arrow) {
                        return_type = self.parse_type()?;
                    }
                    Ok(TypeSpec::Fn {
                        params,
                        ret: Box::new(return_type),
                    })
                },
                _ => Err(self.create_compiler_error(format!("Expected a type specifier. Found '{:?}'.", t.token_type), t.span.clone()))
            }
        } else {
            Err(self.create_compiler_error("Unexpected end of file while parsing type.".to_string(), Span::default()))
        }
    }

    fn parse_type_list(&mut self) -> Result<Vec<TypeSpec>, CompilerError<'a>> {
        let mut types: Vec<TypeSpec> = vec![];
        types.push(self.parse_type()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            types.push(self.parse_type()?);
        }
        Ok(types)
    }

    // Declarations

    fn parse_fn_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, CompilerError<'a>> {
        let mut span = self.consume(TokenType::Fn, "Expected function declaration")?.span;
        let name = self.consume(TokenType::Identifier, "Expected function name after 'fn' keyword.")?.literal;
        self.consume(TokenType::LeftParen, "Expected '(' to start function parameter list.")?;
        let mut params = vec![];
        if let Some(tt) = self.peek_type() {
            if tt != TokenType::RightParen {
                params = self.parse_param_list()?;
            }
        }
        let span_end = self.consume(TokenType::RightParen, "Expected ')' to end function parameter list.")?.span;
        let mut return_type = TypeSpec::Void;
        if let Some(_) = self.matches(TokenType::Arrow) {
            return_type = self.parse_type()?;
        }
        let body = self.parse_block()?;
        span = Span::sum(span, span_end);
        Ok(Item::Function(FnDecl {
            attributes,
            name,
            params,
            return_type,
            body: Box::new(body),
            span,
            scope_id: None,
        }))
    }

    fn parse_param_list(&mut self) -> Result<Vec<(String, TypeSpec)>, CompilerError<'a>> {
        let mut params: Vec<(String, TypeSpec)> = vec![];
        params.push(self.parse_param()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<(String, TypeSpec), CompilerError<'a>> {
        let name = self.consume(TokenType::Identifier, "Expected parameter name.")?.literal;
        self.consume(TokenType::Colon, "Expected ':' after name.")?;
        let ty = self.parse_type()?;
        Ok((name, ty))
    }

    fn parse_struct_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, CompilerError<'a>> {
        let mut span = self.consume(TokenType::Struct, "Expected struct declaration")?.span;
        let name = self.consume(TokenType::Identifier, "Expected struct name after 'struct' keyword.")?.literal;
        let span_end = self.consume(TokenType::LeftBrace, "Expected '{' to begin struct body.")?.span;
        let mut fields: Vec<(String, TypeSpec)> = vec![];
        while let None = self.matches(TokenType::RightBrace) {
            fields.push(self.parse_struct_field()?);
        }
        span = Span::sum(span, span_end);
        Ok(Item::Struct(StructDecl {
            attributes,
            name,
            fields,
            span
        }))
    }

    fn parse_struct_field(&mut self) -> Result<(String, TypeSpec), CompilerError<'a>> {
        let name = self.consume(TokenType::Identifier, "Expected field name.")?.literal;
        self.consume(TokenType::Colon, "Expected ':' after name.")?;
        let ty = self.parse_type()?;
        self.consume(TokenType::Semicolon, "Expected ';' after struct field declaration.")?;
        Ok((name, ty))
    }

    fn parse_enum_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, CompilerError<'a>> {
        let mut span = self.consume(TokenType::Enum, "Expected enum declaration")?.span;
        let name = self.consume(TokenType::Identifier, "Expected enum name after 'enum' keyword.")?.literal;
        let span_end = self.consume(TokenType::LeftBrace, "Expected {")?.span;
        let mut variants: Vec<(String, Option<ExprNode>)> = vec![];
        variants.push(self.parse_enum_item()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            if let Some(tt) = self.peek_type() {
                if tt == TokenType::RightBrace { break };
            }
            variants.push(self.parse_enum_item()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' to end enum body.")?;
        span = Span::sum(span, span_end);
        Ok(Item::Enum(EnumDecl {
            attributes,
            name,
            variants,
            span,
        }))
    }

    fn parse_enum_item(&mut self) -> Result<(String, Option<ExprNode>), CompilerError<'a>> {
        let name = self.consume(TokenType::Identifier, "Expected identifier")?.literal;
        let mut expr: Option<ExprNode> = None;
        if let Some(_) = self.matches(TokenType::Equal) {
            expr = Some(self.parse_expression()?);
        }
        Ok((name, expr))
    }

    fn parse_global_decl(&mut self, attributes: Vec<Attribute>) -> Result<Item, CompilerError<'a>> {
        let is_const: bool;
        let span;
        if let Some(t) = self.peek(0) {
            span = t.span;
            is_const = match t.token_type {
                TokenType::Const => true,
                TokenType::Var => true,
                _ => {
                    return Err(self.create_compiler_error("Bug".to_string(), t.span.clone()));
                },
            }
        } else {
            return Err(self.create_compiler_error("Bug".to_string(), Span::default()));
        }
        self.current += 1; // const|var
        let name = self.consume(TokenType::Identifier, "Expected identifier for global variable.")?.literal;
        self.consume(TokenType::Colon, "Expected ':' to specify type.")?;
        let ty = self.parse_type()?;
        let mut init: Option<ExprNode> = None;
        if let Some(_) = self.matches(TokenType::Equal) {
            init = Some(self.parse_expression()?);
        }
        let span_end = self.consume(TokenType::Semicolon, "Expected ';' after global variable declaration.")?.span;
        Ok(Item::Global(GlobalDecl {
            attributes,
            name,
            ty,
            init,
            is_const,
            span: Span::sum(span, span_end)
        }))
    }

    // Statements
    fn parse_block(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let mut span = self.consume(TokenType::LeftBrace, "Expected {")?.span.clone();
        let mut stmts: Vec<StmtNode> = vec![];
        while let None = self.matches(TokenType::RightBrace) {
            stmts.push(self.parse_statement()?);
        }
        if let Some(stmt) = stmts.last() {
            span = Span::sum(span, stmt.span);
        }
        Ok(StmtNode::new(Stmt::Block(stmts, None), span))
    }

    fn parse_statement(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        if let Some(t) = self.peek(0) {
            match t.token_type {
                TokenType::Let | TokenType::Var => self.parse_var_decl_stmt(),
                TokenType::If => self.parse_if_stmt(),
                TokenType::While => self.parse_while_stmt(),
                TokenType::For => self.parse_for_stmt(),
                TokenType::Return => self.parse_return_stmt(),
                TokenType::Break => {
                    self.current += 1;
                    self.consume(TokenType::Semicolon, "Expected ';' at the end of statement.")?;
                    Ok(StmtNode::new(Stmt::Break, t.span.clone()))
                }
                TokenType::Continue => {
                    self.current += 1;
                    self.consume(TokenType::Semicolon, "Expected ';' at the end of statement.")?;
                    Ok(StmtNode::new(Stmt::Continue, t.span.clone()))
                }
                _ => self.parse_assignment_or_expr_stmt(),
            }
        } else {
            Err(self.create_compiler_error("Expected a type found EOF".to_string(), Span::default()))
        }
    }

    fn parse_var_decl_clause(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let is_mutable: bool;
        if let Some(t) = self.peek(0) {
            is_mutable = match t.token_type {
                TokenType::Let => false,
                TokenType::Var => true,
                _ => {
                    return Err(self.create_compiler_error("Bug".to_string(), t.span.clone()));
                },
            }
        } else {
            return Err(self.create_compiler_error("Bug".to_string(), Span::default()));
        }
        self.current += 1; // let|var
        let name_token = self.consume(TokenType::Identifier, "need identifier after global declaration")?;
        let name = name_token.literal;
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
        }, name_token.span.clone()))
    }

    fn parse_var_decl_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let var_decl_clause = self.parse_var_decl_clause()?;
        self.consume(TokenType::Semicolon, "Expected ;")?;
        Ok(var_decl_clause)
    }

    fn parse_assignment_or_expr_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let target = self.parse_expression()?;
        if let Some(_) = self.matches(TokenType::Equal) {
            let value = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' at the end of statement.")?;
            let span = Span::sum(target.span, value.span);
            Ok(StmtNode::new(Stmt::Assign {
                target,
                value
            }, span))
        } else {
            self.consume(TokenType::Semicolon, "Expected ';' at the end of statement.")?;
            let span= target.span.clone();
            Ok(StmtNode::new(Stmt::Expression(target), span))
        }
    }

    fn parse_assignment_clause(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let target = self.parse_expression()?;
        self.consume(TokenType::Equal, "Expected =")?;
        let value = self.parse_expression()?;
        let span = Span::sum(target.span, value.span);
        Ok(StmtNode::new(Stmt::Assign {
            target,
            value
        }, span))
    }

    fn parse_assignment_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let assignment_clause = self.parse_assignment_clause()?;
        self.consume(TokenType::Semicolon, "Expected ;")?;
        Ok(assignment_clause)
    }

    fn parse_if_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        self.consume(TokenType::If, "Bug in If")?;
        let condition = self.parse_expression()?;
        let then_branch = Box::new(self.parse_block()?);
        let mut else_branch: Option<Box<StmtNode>> = None;
        if let Some(_) = self.matches(TokenType::Else) {
            if let Some(t) = self.peek(0) {
                else_branch = match t.token_type {
                    TokenType::If => Some(Box::new(self.parse_if_stmt()?)),
                    TokenType::LeftBrace => Some(Box::new(self.parse_block()?)),
                    _ => {
                        return Err(self.create_compiler_error("Unexpected token in 'else' clause. Expected 'if' or block.".to_string(), t.span.clone()));
                    }
                };
            } else {
                return Err(self.create_compiler_error("Unexpected EOF".to_string(), Span::default()));
            }
        }

        let span = condition.span.clone();
        Ok(StmtNode::new(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }, span))
    }

    fn parse_while_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        self.consume(TokenType::While, "Bug in While")?;
        let condition = self.parse_expression()?;
        let body = Box::new(self.parse_block()?);
        let span = condition.span.clone();
        Ok(StmtNode::new(Stmt::While {
            condition,
            body
        }, span))
    }

    fn parse_for_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let span = self.consume(TokenType::For, "Expected 'for' keyword")?.span.clone();
        self.consume(TokenType::LeftParen, "Expected '(' after 'for' to begin loop clauses.")?;
        let init: Option<Box<StmtNode>> = if self.matches(TokenType::Semicolon).is_none() {
            let init_stmt = match self.peek_type() {
                Some(TokenType::Let) | Some(TokenType::Var) => {
                    self.parse_var_decl_stmt()?
                }
                _ => {
                    self.parse_assignment_stmt()?
                }
            };
            Some(Box::new(init_stmt))
        } else {
            None
        };

        let condition: Option<ExprNode> = if self.matches(TokenType::Semicolon).is_none() {
            let expr = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after 'for' loop condition.")?;
            Some(expr)
        } else {
            None
        };

        let update: Option<Box<StmtNode>> = if self.matches(TokenType::RightParen).is_none() {
            let stmt = self.parse_assignment_clause()?;
            self.consume(TokenType::RightParen, "Expected ')' after 'for' loop update clause.")?;
            Some(Box::new(stmt))
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(StmtNode::new(Stmt::For {
            init,
            condition,
            update,
            body: Box::new(body),
            scope_id: None,
        }, span))
    }

    fn parse_return_stmt(&mut self) -> Result<StmtNode, CompilerError<'a>> {
        let mut span = self.consume(TokenType::Return, "Bug in Return")?.span.clone();
        let expr: Option<ExprNode>;
        if let None = self.matches(TokenType::Semicolon) {
            expr = Some(self.parse_expression()?);
            let span_a = self.consume(TokenType::Semicolon, "Expected ';' at the end of statement.")?.span;
            span = Span::sum(span, span_a);
        } else {
            expr = None;
        }
        Ok(StmtNode::new(Stmt::Return(expr), span))
    }

    // Expressions
    fn parse_expression(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_logical_and()?;
        while let Some(_) = self.matches(TokenType::PipePipe) {
            let rhs = self.parse_logical_and()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::Or,
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_equality()?;
        while let Some(_) = self.matches(TokenType::AmpersandAmpersand) {
            let rhs = self.parse_equality()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::And,
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_comparison()?;
        while let Some(t) = self.m_matches(vec![TokenType::EqualEqual, TokenType::BangEqual]).cloned() {
            let rhs = self.parse_comparison()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: { if t.token_type == TokenType::BangEqual { BinaryOp::Neq } else { BinaryOp::Eq }},
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_bitwise_or()?;
        while let Some(t) = self.m_matches(vec![TokenType::Less, TokenType::Greater, TokenType::LessEqual, TokenType::GreaterEqual]).cloned() {
            let rhs = self.parse_bitwise_or()?;
            let span = Span::sum(expr.span, rhs.span);
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
            }, span)
        }
        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_bitwise_xor()?;
        while let Some(_) = self.matches(TokenType::Pipe) {
            let rhs = self.parse_bitwise_xor()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::BitOr,
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_bitwise_and()?;
        while let Some(_) = self.matches(TokenType::Caret) {
            let rhs = self.parse_bitwise_and()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::BitXor,
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_shift()?;
        while let Some(_) = self.matches(TokenType::Ampersand) {
            let rhs = self.parse_shift()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: BinaryOp::BitAnd,
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_term()?;
        while let Some(t) = self.m_matches(vec![TokenType::LessLess, TokenType::GreaterGreater]).cloned() {
            let rhs = self.parse_term()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: { if t.token_type == TokenType::LessLess { BinaryOp::Shl } else { BinaryOp::Shr }},
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_factor()?;
        while let Some(t) = self.m_matches(vec![TokenType::Plus, TokenType::Minus]).cloned() {
            let rhs = self.parse_factor()?;
            let span = Span::sum(expr.span, rhs.span);
            expr = ExprNode::new(Expr::Binary {
                lhs: Box::new(expr),
                op: { if t.token_type == TokenType::Plus { BinaryOp::Add } else { BinaryOp::Sub }},
                rhs: Box::new(rhs)
            }, span)
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_cast()?;
        while let Some(t) = self.m_matches(vec![TokenType::Star, TokenType::Slash, TokenType::Percent]).cloned() {
            let rhs = self.parse_cast()?;
            let span = Span::sum(expr.span, rhs.span);
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
            }, span)
        }
        Ok(expr)
    }

    fn parse_cast(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let expr = self.parse_unary()?;
        if let Some(_) = self.matches(TokenType::As) {
            let target = self.parse_type()?;
            let span = Span::sum(expr.span, self.previous().unwrap().span);
            Ok(ExprNode::new(Expr::Cast {
                expr: Box::new(expr),
                target,
            }, span))
        } else {
            Ok(expr)
        }
    }

    fn parse_unary(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        if let Some(t) = self.peek(0) {
            let tt = t.token_type.clone();
            match tt {
                TokenType::Bang => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::Not,
                        rhs: Box::new(self.parse_unary()?),
                    }, t.span))
                },
                TokenType::Minus => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::Neg,
                        rhs: Box::new(self.parse_unary()?),
                    }, t.span))
                },
                TokenType::Star => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::Deref,
                        rhs: Box::new(self.parse_unary()?),
                    }, t.span))
                },
                TokenType::Ampersand => {
                    self.current += 1;
                    Ok(ExprNode::new(Expr::Unary {
                        op: UnaryOp::AddressOf,
                        rhs: Box::new(self.parse_unary()?),
                    }, t.span))
                },
                TokenType::SizeOf => {
                    self.current += 1;
                    self.consume(TokenType::LeftParen, "Expected '(' after 'sizeof'.")?;
                    let current = self.current; // check_point
                    let target: SizeOfTarget;
                    
                    let try_type = self.parse_type();
                    if let Ok(ty) = try_type {
                        if self.peek_type() == Some(TokenType::LeftParen) {
                            target = SizeOfTarget::Type(ty);
                        } else {
                            self.current = current; // Rollback
                            target = SizeOfTarget::Expr(Box::new(self.parse_expression()?));
                        }
                    } else {
                        self.current = current; // Rollback
                        target = SizeOfTarget::Expr(Box::new(self.parse_expression()?));
                    }
                    let span_end = self.consume(TokenType::RightParen, "Expected ')' after 'sizeof' argument.")?.span;
                    Ok(ExprNode::new(Expr::SizeOf { target }, Span::sum(t.span, span_end)))
                },
                _ => self.parse_postfix(),
            }
        } else {
            Err(self.create_compiler_error("Unexpected end of file while parsing expression.".to_string(), Span::default()))
        }
    }

    fn parse_postfix(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        let mut expr = self.parse_primary()?;
        loop {
            let current = self.current;
            expr = self.parse_postfix_op(expr)?;
            if current == self.current { break }
        }
        Ok(expr)
    }

    fn parse_postfix_op(&mut self, expr: ExprNode) -> Result<ExprNode, CompilerError<'a>> {
        if let Some(t) = self.m_matches(vec![TokenType::LeftBracket, TokenType::LeftParen, TokenType::Dot, TokenType::Arrow]).cloned() {
            match t.token_type {
                TokenType::LeftBracket => {
                    let idx = self.parse_expression()?;
                    let s = self.consume(TokenType::RightBracket, "Expected ']' to close array index.")?.span;
                    Ok(ExprNode::new(Expr::Index {
                        array: Box::new(expr),
                        index: Box::new(idx),
                    }, Span::sum(t.span, s)))
                },
                TokenType::LeftParen => {
                    let mut args: Vec<ExprNode> = vec![];
                    if let Some(tt) = self.peek_type() {
                        if tt != TokenType::RightParen {
                            args = self.parse_arg_list()?;
                        }
                    } else {
                        return Err(self.create_compiler_error("Unexpected EOF in argument list".to_string(), Span::default()));
                    }
                    let s = self.consume(TokenType::RightParen, "Expected ')' to close attribute argument list.")?.span;
                    Ok(ExprNode::new(Expr::Call {
                        callee: Box::new(expr),
                        args,
                    }, Span::sum(t.span, s)))
                },
                TokenType::Dot => {
                    let member_token = self.consume(TokenType::Identifier, "Expected member name after '.'.")?;
                    let member = member_token.literal;
                    Ok(ExprNode::new(Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: false,
                    }, Span::sum(t.span, member_token.span)))
                },
                TokenType::Arrow => {
                    let member_token = self.consume(TokenType::Identifier, "Expected member name after '->'.")?;
                    let member = member_token.literal;
                    Ok(ExprNode::new(Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: true,
                    }, Span::sum(t.span, member_token.span)))
                }
                _ => unreachable!()
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_primary(&mut self) -> Result<ExprNode, CompilerError<'a>> {
        if let Some(t) = self.advance() {
            match t.token_type {
                TokenType::Int => {
                    let clean_literal = t.literal.replace('_', "");
                    let val = if clean_literal.starts_with("0x") || clean_literal.starts_with("0X") {
                        u64::from_str_radix(&clean_literal[2..], 16)
                    } else if clean_literal.starts_with("0b") || clean_literal.starts_with("0B") {
                        u64::from_str_radix(&clean_literal[2..], 2)
                    } else {
                        clean_literal.parse::<u64>()
                    };
                    let val = val.map_err(|_|
                        self.create_compiler_error(format!("Integer literal '{}' is too large or invalid", t.literal), t.span)
                    )?;
                    Ok(ExprNode::new(Expr::LiteralInt(val), t.span))
                },
                TokenType::Float => {
                    let clean_literal = t.literal.replace('_', "");
                    let val = clean_literal.parse::<f64>().map_err(|_|
                        self.create_compiler_error(format!("Float literal '{}' is invalid or too large", t.literal), t.span)
                    )?;
                    Ok(ExprNode::new(Expr::LiteralFloat(val), t.span))
                },
                TokenType::String => Ok(ExprNode::new(Expr::LiteralString(t.literal.clone()), t.span)),
                TokenType::Char => {
                    let text = t.literal;
                    let char_byte: u8 = if text.starts_with('\\') {
                        match text.chars().nth(1).unwrap_or('\0') {
                            'n' => b'\n',
                            'r' => b'\r',
                            't' => b'\t',
                            '\\' => b'\\',
                            '\'' => b'\'',
                            '"' => b'"',
                            '0' => b'\0',
                            c => return Err(self.create_compiler_error(format!("Unknown escape sequence '\\{}' in char literal.", c), t.span)),
                        }
                    } else {
                        if text.len() != 1 {
                            return Err(self.create_compiler_error(format!("Invalid character literal '{}'. Char must be exactly one byte.", text), t.span));
                        }
                        text.as_bytes()[0]
                    };
                    Ok(ExprNode::new(Expr::LiteralChar(char_byte), t.span))
                },
                TokenType::True => Ok(ExprNode::new(Expr::LiteralBool(true), t.span)),
                TokenType::False => Ok(ExprNode::new(Expr::LiteralBool(false), t.span)),
                TokenType::Null => Ok(ExprNode::new(Expr::Null, t.span)),
                TokenType::Identifier => {
                    if self.matches(TokenType::LeftBrace).is_some() {
                        let mut fields: Vec<(String, ExprNode)> = vec![];
                        while let Some(_t) = self.matches(TokenType::Identifier) {
                            self.consume(TokenType::Colon, "Expected ':'")?;
                            fields.push((_t.literal.clone(), self.parse_expression()?));
                            if let None = self.matches(TokenType::Comma) {
                                break;
                            }
                        }
                        let span = Span::sum(t.span, self.consume(TokenType::RightBrace, "expected '}'")?.span);
                        Ok(ExprNode::new(Expr::StructInit {
                            name: t.literal,
                            fields,
                        }, span))
                    } else {
                        Ok(ExprNode::new(Expr::Identifier(t.literal), t.span))
                    }
                },
                TokenType::LeftParen => {
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::RightParen, "Expected ')'")?;
                    Ok(expr)
                },
                _ => Err(self.create_compiler_error(format!("Unexpected token {:?}", t), t.span.clone()))
            }
        } else {
            Err(self.create_compiler_error("Unexpected EOF".to_string(), Span::default()))
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<ExprNode>, CompilerError<'a>> {
        let mut args: Vec<ExprNode> = vec![];
        args.push(self.parse_expression()?);
        while let Some(_) = self.matches(TokenType::Comma) {
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }
}