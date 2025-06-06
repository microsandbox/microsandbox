package msb

import "errors"

type ProgLang int

const (
	LangUnspecified ProgLang = iota
	LangPython
	LangNodeJs
)

// String should be the language's corresponding RPC parameter.
func (p ProgLang) String() string {
	switch p {
	case LangPython:
		return "python"
	case LangNodeJs:
		return "nodejs"
	default:
		panic(ErrUnknownLanguage)
	}
}

func (p ProgLang) DefaultImage() string {
	switch p {
	case LangPython:
		return "microsandbox/python"
	case LangNodeJs:
		return "microsandbox/node"
	default:
		panic(ErrUnknownLanguage)
	}
}

// Language-related errors
var (
	ErrUnknownLanguage = errors.New("unknown language")
)
