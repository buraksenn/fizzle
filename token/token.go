package token

type Type string

const (
	ILLEGAL = "ILLEGAL"
	EOF     = "EOF"

	IDENT = "IDENT" // add, foobar, x, y, ...
	INT   = "INT"   // 123456

	ASSIGN = "="
	PLUS   = "+"
	MINUS  = "-"

	COMMA     = ","
	SEMICOLON = ";"

	LPAREN = "("
	RPAREN = ")"
	LBRACE = "{"
	RBRACE = "}"

	FUNCTION = "FUNCTION"
	LET      = "LET"
)

var (
	tokenStringsToTokenType = map[string]Type{
		"=": ASSIGN,
		"+": PLUS,
		"-": MINUS,
		",": COMMA,
		";": SEMICOLON,
		"(": LPAREN,
		")": RPAREN,
		"{": LBRACE,
		"}": RBRACE,
	}

	keywords = map[string]Type{
		"fn":  FUNCTION,
		"let": LET,
	}
)

type Token struct {
	Type    Type
	Literal string
}

func NewFromKeyword(keyword byte) Token {
	t, ok := tokenStringsToTokenType[string(keyword)]
	if !ok {
		return Token{Type: ILLEGAL, Literal: string(keyword)}
	}

	return Token{Type: t, Literal: string(keyword)}
}

func New(tokenType Type, ch byte) Token {
	return Token{Type: tokenType, Literal: string(ch)}
}

func LookupIdent(ident string) Type {
	if tok, ok := keywords[ident]; ok {
		return tok
	}
	return IDENT
}
