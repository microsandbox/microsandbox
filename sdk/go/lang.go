package msb

import "errors"

type LangSandBox interface {
	Starter
	Stopper
	Code() CodeRunner
	Command() CommandRunner
	Metrics() MetricsReader
}

var _ LangSandBox = (*langSandbox)(nil)

type langSandbox struct {
	b *baseMicroSandbox
	l progLang
}

func NewPythonSandbox(options ...Option) *langSandbox {
	return newLangSandbox(langPython, options...)
}

func NewNodeSandbox(options ...Option) *langSandbox {
	return newLangSandbox(langNodeJs, options...)
}

func newLangSandbox(lang progLang, options ...Option) *langSandbox {
	b := newBaseWithOptions(options...)
	n := &langSandbox{
		b: b,
		l: lang,
	}
	return n
}

func (ls *langSandbox) Start(image string, memoryMB int, cpus int) error {
	if image == "" {
		image = ls.l.DefaultImage()
	}
	return starter{ls.b}.Start(image, memoryMB, cpus)
}

func (ls *langSandbox) Stop() error {
	return stopper{ls.b}.Stop()
}

func (ls *langSandbox) Code() CodeRunner {
	return codeRunner{ls.b, ls.l}
}

func (ls *langSandbox) Command() CommandRunner {
	return commandRunner{ls.b}
}

func (ls *langSandbox) Metrics() MetricsReader {
	return metricsReader{ls.b}
}

type progLang int

const (
	langUnspecified progLang = iota
	langPython
	langNodeJs
)

// String should be the language's corresponding RPC parameter.
func (p progLang) String() string {
	switch p {
	case langPython:
		return "python"
	case langNodeJs:
		return "nodejs"
	default:
		panic(ErrUnknownLanguage)
	}
}

func (p progLang) DefaultImage() string {
	switch p {
	case langPython:
		return "microsandbox/python"
	case langNodeJs:
		return "microsandbox/node"
	default:
		panic(ErrUnknownLanguage)
	}
}

// Language-related errors
var (
	ErrUnknownLanguage = errors.New("unknown language")
)
