package ast

import "strings"

type Program struct {
	Statements []Statement
}

func (p *Program) TokenLiteral() string {
	if len(p.Statements) > 0 {
		return p.Statements[0].TokenLiteral()
	} else {
		return ""
	}
}

func (p *Program) String() string {
	var stringBuilder strings.Builder
	for _, s := range p.Statements {
		stringBuilder.WriteString(s.String())
	}

	return stringBuilder.String()
}
