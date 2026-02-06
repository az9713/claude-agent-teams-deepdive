package main

import "fmt"

func main() {
	// TODO(alice): Add context support
	fmt.Println("hello")

	// FIXME(bob, #123): Fix race condition
	go func() {}()

	/* HACK: This is a workaround */
	x := 42
	_ = x
}
