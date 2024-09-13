package main

import (
	"fmt"
	"github.com/buraksenn/fizzle/repl"
	"os"
)

func main() {
	fmt.Println("Hell from the Fizzle programming language!")
	fmt.Printf("Start typing...\n")
	repl.Start(os.Stdin, os.Stdout)
}
