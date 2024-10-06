package parser

import (
	"github.com/buraksenn/fizzle/ast"
	"github.com/buraksenn/fizzle/token"
)

const (
	_ int = iota
	LOWEST
	EQUALS          // ==
	LESS_OR_GREATER // > or <
	SUM             // +
	PRODUCT         // *
	PREFIX          // -X or !X
	CALL            // myFunction(X)
)

func (p *Parser) parseIdentifier() ast.Expression {
	return &ast.Identifier{Token: p.currentToken, Value: p.currentToken.Literal}
}

func (p *Parser) parseExpressionStatement() *ast.ExpressionStatement {
	stmt := &ast.ExpressionStatement{Token: p.currentToken}
	stmt.Expression = p.parseExpression(LOWEST)
	if p.peekTokenIs(token.SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseLetStatement() *ast.LetStatement {
	stmt := &ast.LetStatement{Token: p.currentToken}
	if !p.expectPeek(token.IDENT) {
		return nil
	}
	stmt.Name = &ast.Identifier{Token: p.currentToken, Value: p.currentToken.Literal}
	if !p.expectPeek(token.ASSIGN) {
		return nil
	}
	// TODO: We're skipping the expressions until we
	// encounter a semicolon
	for !p.currentTokenIs(token.SEMICOLON) {
		p.nextToken()
	}
	return stmt
}

func (p *Parser) parseReturnStatement() *ast.ReturnStatement {
	stmt := &ast.ReturnStatement{Token: p.currentToken}
	p.nextToken()
	// TODO: We're skipping the expressions until we
	// encounter a semicolon
	for !p.currentTokenIs(token.SEMICOLON) {
		p.nextToken()
	}
	return stmt
}
