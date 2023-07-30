use std::rc::Rc;

use crate::common::datatypes::{DataType, Variable};
use crate::common::diagnostics::Diagnostics;
use crate::common::errors::CompilerError;
use crate::common::operators::arithmetic::Arithmetic::*;
use crate::common::operators::assignment::Assingment::*;
use crate::common::operators::logical::Logical;
use crate::common::operators::relational::Relational::*;
use crate::common::operators::Operator;
use crate::common::operators::Operator::*;
use crate::lexical_analysis::keywords::Keyword;
use crate::lexical_analysis::lexer::Lexer;
use crate::lexical_analysis::symbols::Symbol::*;
use crate::lexical_analysis::token::{Token, TokenKind};
use crate::syntax_analysis::ast::AbstractSyntaxTree;

use super::block::Block;

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result<Rc<Block>, CompilerError> {
        if self.lexer.get_token_count() == 0 {
            self.lexer.lex()?;
        }
        let mut global_block = Block::new();
        let mut statements = Vec::new();
        while TokenKind::EndOfFileToken != self.lexer.get_current_token().kind {
            let statement = self.parse_expression(&mut global_block)?;
            statements.push(Box::new(statement));
        }
        global_block.statements = statements;
        Ok(Rc::new(global_block))
    }

    fn match_token(&self, kind: TokenKind) -> Rc<Token> {
        let current_token = self.lexer.get_current_token();
        if kind == current_token.kind {
            self.lexer.advance();
            return current_token;
        } else {
            // add diagnostics
            return Rc::new(Token::new(
                TokenKind::FactoryToken,
                current_token.line,
                current_token.column,
            ));
        }
    }

    fn parse_block(&self, parent: Rc<Block>) -> Result<Rc<Block>, CompilerError> {
        self.match_token(TokenKind::SymbolToken(OpenCurlyBracket));
        let mut block = Block::from(parent);
        let mut statements: Vec<Box<AbstractSyntaxTree>> = Vec::new();
        while TokenKind::SymbolToken(CloseCurlyBracket) != self.lexer.get_current_token().kind
            && TokenKind::EndOfFileToken != self.lexer.get_current_token().kind
        {
            let statement = self.parse_expression(&mut block)?;
            statements.push(Box::new(statement));
        }
        self.match_token(TokenKind::SymbolToken(CloseCurlyBracket));
        block.statements = statements;
        Ok(Rc::new(block))
    }

    pub fn parse_expression(&self, block: &mut Block) -> Result<AbstractSyntaxTree, CompilerError> {
        self.parse_assignment_expression(block)
    }

    fn parse_assignment_expression(
        &self,
        block: &mut Block,
    ) -> Result<AbstractSyntaxTree, CompilerError> {
        let identifier_token = self.lexer.get_current_token();
        match &identifier_token.kind {
            TokenKind::KeywordToken(keyword) => match keyword {
                Keyword::Mutable => self.handle_mutable_keyword(block),
                _ => self.parse_arithmetic_expression(0, block),
            },
            TokenKind::IdentifierToken(name) => {
                if let Some((operator, length)) = self.match_operator(1) {
                    if let AssignmentOperator(_) = operator {
                        for _ in 0..length {
                            self.lexer.advance();
                        }
                        let expression = self.parse_assignment_expression(block)?;

                        Ok(AbstractSyntaxTree::AssignmentExpression(
                            name.to_string(),
                            operator,
                            Box::new(expression),
                        ))
                    } else {
                        self.parse_arithmetic_expression(0, block)
                    }
                } else {
                    self.parse_arithmetic_expression(0, block)
                }
            }
            _ => self.parse_arithmetic_expression(0, block),
            // TokenKind::LiteralToken(_) => todo!(),
            // TokenKind::WhitespaceToken(_) => todo!(),
            // TokenKind::NewLineToken => todo!(),
            // TokenKind::SymbolToken(_) => todo!(),
            // TokenKind::FactoryToken => todo!(),
            // TokenKind::EndOfFileToken => todo!(),
        }
    }

    fn parse_arithmetic_expression(
        &self,
        parent_precedence: u8,
        block: &mut Block,
    ) -> Result<AbstractSyntaxTree, CompilerError> {
        let mut left = if let Some((operator, _)) = self.match_operator(0) {
            if operator.get_unery_precedence() >= parent_precedence {
                self.lexer.advance();
                let expression =
                    self.parse_arithmetic_expression(operator.get_unery_precedence(), block)?;
                AbstractSyntaxTree::UnaryExpression(operator, Box::new(expression))
            } else {
                self.parse_factor(block)?
            }
        } else {
            self.parse_factor(block)?
        };

        while let Some((operator, length)) = self.match_operator(0) {
            let precedence = operator.get_binary_precedence();
            if precedence <= parent_precedence {
                break;
            }
            for _ in 0..length {
                self.lexer.advance();
            }
            let right = self.parse_arithmetic_expression(precedence, block)?;
            left = AbstractSyntaxTree::BinaryExpression(Box::new(left), operator, Box::new(right));
        }

        Ok(left)
    }

    fn parse_factor(&self, block: &mut Block) -> Result<AbstractSyntaxTree, CompilerError> {
        let token = self.lexer.get_current_token_and_advance();
        match &token.kind {
            TokenKind::LiteralToken(variable) => Ok(AbstractSyntaxTree::Literal(variable.clone())),
            TokenKind::SymbolToken(symbol) => match symbol {
                OpenParanthesis => {
                    let expression = self.parse_expression(block)?;
                    let next_token = self.lexer.get_current_token_and_advance();
                    if next_token.kind == TokenKind::SymbolToken(CloseParanthesis) {
                        Ok(AbstractSyntaxTree::ParenthesizedExpression(Box::new(
                            expression,
                        )))
                    } else {
                        Err(CompilerError::UnexpectedToken(
                            TokenKind::SymbolToken(CloseParanthesis),
                            token.line,
                            token.column,
                        ))
                    }
                }
                CloseParanthesis => Err(CompilerError::UnexpectedToken(
                    TokenKind::SymbolToken(CloseParanthesis),
                    token.line,
                    token.column,
                )),
                _ => todo!("parse_factor"),
            },
            TokenKind::IdentifierToken(name) => Ok(AbstractSyntaxTree::Identifier(name.clone())),
            kind => Err(CompilerError::UnexpectedToken(
                kind.clone(),
                token.line,
                token.column,
            )),
        }
    }

    fn handle_mutable_keyword(
        &self,
        block: &mut Block,
    ) -> Result<AbstractSyntaxTree, CompilerError> {
        if let TokenKind::IdentifierToken(variable_name) = &self.lexer.peek(1).kind {
            if let Some((operator, length)) = self.match_operator(2) {
                // `mutable` variable_name operator expression
                for _ in 0..length {
                    self.lexer.advance();
                }
                let expression = self.parse_assignment_expression(block)?;
                handle_mutable_assignment(variable_name, operator, expression, block)
            } else {
                // `mutable` variable_name
                if block.contains_symbol(variable_name) {
                    return Err(CompilerError::CannotConvertFromImmutableToMutable);
                } else {
                    Err(CompilerError::UnInitializedVariable(
                        variable_name.to_string(),
                    ))
                }
            }
        } else {
            Err(CompilerError::InvalidUseOfMutableKeyword)
        }
    }

    fn match_operator(&self, offset: usize) -> Option<(Operator, usize)> {
        // TODO: use match instead of if let to include keyword operators
        // like 'and', 'or', 'not', 'xor', 'is', 'in', 'not in', 'is not'
        let (operator, length) = if let TokenKind::SymbolToken(operator_symbol) =
            self.lexer.peek(offset).kind
        {
            match operator_symbol {
                Equals => {
                    // =
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // ==
                            (RelationalOperator(Equality), 2)
                        } else {
                            // =
                            (AssignmentOperator(SimpleAssignment), 1)
                        }
                    } else {
                        // =
                        (AssignmentOperator(SimpleAssignment), 1)
                    }
                }
                Plus => {
                    // +
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // +=
                            (AssignmentOperator(AdditionAssignment), 2)
                        } else {
                            // +
                            (ArithmeticOperator(Addition), 1)
                        }
                    } else {
                        // +
                        (ArithmeticOperator(Addition), 1)
                    }
                }
                Minus => {
                    // -
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // -=
                            (AssignmentOperator(SubtractionAssignment), 2)
                        } else {
                            // -
                            (ArithmeticOperator(Subtraction), 1)
                        }
                    } else {
                        // -
                        (ArithmeticOperator(Subtraction), 1)
                    }
                }
                Asterisk => {
                    // *
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // *=
                            (AssignmentOperator(MultiplicationAssignment), 2)
                        } else if second_symbol == Asterisk {
                            // **
                            if let TokenKind::SymbolToken(third_token) =
                                self.lexer.peek(offset + 2).kind
                            {
                                if third_token == Equals {
                                    // **=
                                    (AssignmentOperator(ExponentiationAssignment), 3)
                                } else {
                                    // **
                                    (ArithmeticOperator(Exponentiation), 2)
                                }
                            } else {
                                // **
                                (ArithmeticOperator(Exponentiation), 2)
                            }
                        } else {
                            // *
                            (ArithmeticOperator(Multiplication), 1)
                        }
                    } else {
                        // *
                        (ArithmeticOperator(Multiplication), 1)
                    }
                }
                Slash => {
                    // /
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // /=
                            (AssignmentOperator(DivisionAssignment), 2)
                        } else {
                            // /
                            (ArithmeticOperator(Division), 1)
                        }
                    } else {
                        // /
                        (ArithmeticOperator(Division), 1)
                    }
                }
                Percent => {
                    // %
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // %=
                            (AssignmentOperator(ModuloAssignment), 2)
                        } else {
                            // %
                            (ArithmeticOperator(Modulo), 1)
                        }
                    } else {
                        // %
                        (ArithmeticOperator(Modulo), 1)
                    }
                }
                Exclamation => {
                    // !
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // !=
                            (RelationalOperator(InEquality), 2)
                        } else {
                            // !
                            (LogicalOperator(Logical::Not), 1)
                        }
                    } else {
                        // !
                        (LogicalOperator(Logical::Not), 1)
                    }
                }
                crate::lexical_analysis::symbols::Symbol::GreaterThan => {
                    // >
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // >=
                            (RelationalOperator(GreaterThanOrEquals), 2)
                        } else {
                            // >
                            (
                                RelationalOperator(
                                    crate::common::operators::relational::Relational::GreaterThan,
                                ),
                                1,
                            )
                        }
                    } else {
                        // >
                        (
                            RelationalOperator(
                                crate::common::operators::relational::Relational::GreaterThan,
                            ),
                            1,
                        )
                    }
                }
                crate::lexical_analysis::symbols::Symbol::LessThan => {
                    // <
                    if let TokenKind::SymbolToken(second_symbol) = self.lexer.peek(offset + 1).kind
                    {
                        if second_symbol == Equals {
                            // <=
                            (RelationalOperator(LessThanOrEquals), 2)
                        } else {
                            // <
                            (
                                RelationalOperator(
                                    crate::common::operators::relational::Relational::LessThan,
                                ),
                                1,
                            )
                        }
                    } else {
                        // <
                        (
                            RelationalOperator(
                                crate::common::operators::relational::Relational::LessThan,
                            ),
                            1,
                        )
                    }
                }
                _ => {
                    return None;
                }
            }
        } else if let TokenKind::KeywordToken(keyword) = &self.lexer.peek(offset).kind {
            match keyword {
                Keyword::Is => {
                    if let TokenKind::KeywordToken(keyword) = &self.lexer.peek(offset + 1).kind {
                        if Keyword::Not == *keyword {
                            (RelationalOperator(InEquality), 2)
                        } else {
                            (RelationalOperator(Equality), 1)
                        }
                    } else {
                        (RelationalOperator(Equality), 1)
                    }
                }
                Keyword::And => (LogicalOperator(Logical::And), 1),
                Keyword::Or => (LogicalOperator(Logical::Or), 1),
                Keyword::Not => (LogicalOperator(Logical::Not), 1),
                Keyword::Xor => (LogicalOperator(Logical::Xor), 1),
                _ => {
                    return None;
                }
            }
        } else {
            return None;
        };
        Some((operator, offset + length))
    }
}

fn handle_mutable_assignment(
    variable_name: &String,
    operator: Operator,
    expression: AbstractSyntaxTree,
    block: &mut Block,
) -> Result<AbstractSyntaxTree, CompilerError> {
    match operator {
        AssignmentOperator(SimpleAssignment) => {
            if block.contains_symbol(variable_name) {
                let old_variable = block.get_symbol(variable_name).unwrap();
                if old_variable.is_mutable() {
                    // `mutable` variable_name = old_expression
                    // `mutable` variable_name = new_expression
                    // TODO: add diagnostics. saying
                    // you don't need to use mutable keyword twice, once it is declared as mutable it will be mutable forever
                    Diagnostics::add_error(CompilerError::Warnings("You don't need to use mutable keyword twice, once it is declared as mutable it will be mutable forever"));
                    block.add_symbol(
                        variable_name.to_string(),
                        Variable::new_mutable(DataType::InternalUndefined),
                    );
                } else {
                    // variable_name = old_expression
                    // `mutable` variable_name = new_expression
                    return Err(CompilerError::CannotConvertFromImmutableToMutable);
                }
            } else {
                // `mutable` variable_name = expression
                block.add_symbol(
                    variable_name.to_string(),
                    Variable::new_mutable(DataType::InternalUndefined),
                );
            }

            Ok(AbstractSyntaxTree::AssignmentExpression(
                variable_name.to_string(),
                operator,
                Box::new(expression),
            ))
        }
        // mutable a += 10
        AssignmentOperator(_) => {
            if block.contains_symbol(variable_name) {
                let old_variable = block.get_symbol(variable_name).unwrap();
                if old_variable.is_mutable() {
                    // `mutable` variable_name operator old_expression
                    // `mutable` variable_name assignment_operator new expression
                    Diagnostics::add_error(CompilerError::Warnings("You don't need to use mutable keyword twice, once it is declared as mutable it will be mutable forever"));
                    block.add_symbol(
                        variable_name.to_string(),
                        Variable::new_mutable(DataType::InternalUndefined),
                    );
                } else {
                    // variable_name operator expression
                    // `mutable` variable_name assignment_operator new expression
                    return Err(CompilerError::CannotConvertFromImmutableToMutable);
                }
                block.add_symbol(
                    variable_name.to_string(),
                    Variable::new_mutable(block.get_symbol(variable_name).unwrap().value),
                );
            } else {
                return Err(CompilerError::UndefinedVariable(variable_name.to_string()));
            }
            Ok(AbstractSyntaxTree::AssignmentExpression(
                variable_name.to_string(),
                operator,
                Box::new(expression),
            ))
        }
        _ => Err(CompilerError::InvalidOperationAsAssignmentOperation),
    }
}
