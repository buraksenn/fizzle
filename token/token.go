package token

type Type string

const (
	ILLEGAL = "ILLEGAL"
	EOF     = "EOF"

	IDENT = "IDENT" // add, foobar, x, y, ...
	INT   = "INT"   // 123456

	ASSIGN   = "="
	PLUS     = "+"
	MINUS    = "-"
	BANG     = "!"
	ASTERISK = "*"
	SLASH    = "/"
	LT       = "<"
	GT       = ">"
	EQ       = "=="
	NOT_EQ   = "!="

	COMMA     = ","
	SEMICOLON = ";"

	LPAREN = "("
	RPAREN = ")"
	LBRACE = "{"
	RBRACE = "}"

	FUNCTION = "FUNCTION"
	LET      = "LET"
	TRUE     = "TRUE"
	FALSE    = "FALSE"
	IF       = "IF"
	ELSE     = "ELSE"
	RETURN   = "RETURN"
)

var (
	tokenStringsToTokenType = map[string]Type{
		"=":  ASSIGN,
		"+":  PLUS,
		"-":  MINUS,
		"!":  BANG,
		"*":  ASTERISK,
		"/":  SLASH,
		"==": EQ,
		"!=": NOT_EQ,
		"<":  LT,
		">":  GT,
		",":  COMMA,
		";":  SEMICOLON,
		"(":  LPAREN,
		")":  RPAREN,
		"{":  LBRACE,
		"}":  RBRACE,
	}

	keywords = map[string]Type{
		"fn":     FUNCTION,
		"let":    LET,
		"true":   TRUE,
		"false":  FALSE,
		"if":     IF,
		"else":   ELSE,
		"return": RETURN,
	}
)

type Token struct {
	Type    Type
	Literal string
}

func NewFromKeyword(keyword byte) Token {
	return NewFromString(string(keyword))
}

func NewFromString(keyword string) Token {
	t, ok := tokenStringsToTokenType[keyword]
	if !ok {
		return Token{Type: ILLEGAL, Literal: keyword}
	}

	return Token{Type: t, Literal: keyword}
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
