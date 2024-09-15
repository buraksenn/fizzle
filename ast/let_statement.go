package ast

import (
	"github.com/buraksenn/fizzle/token"
	"strings"
)

type LetStatement struct {
	Token token.Token
	Name  *Identifier
	Value Expression
}

func (ls *LetStatement) statementNode()       {}
func (ls *LetStatement) TokenLiteral() string { return ls.Token.Literal }

func (ls *LetStatement) String() string {
	var sb strings.Builder
	sb.WriteString(ls.TokenLiteral())
	sb.WriteString(" ")
	sb.WriteString(ls.Name.String())
	sb.WriteString(" = ")
	if ls.Value != nil {
		sb.WriteString(ls.Value.String())
	}

	sb.WriteString(";")

	return sb.String()
}
