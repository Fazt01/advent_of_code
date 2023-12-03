package main

import (
	"bufio"
	"fmt"
	"os"
)

func main() {
	scanner := bufio.NewScanner(os.Stdin)

	var sum int = 0
	for scanner.Scan() {
		t := scanner.Text()
		sum += scoreCombo(t)
	}

	fmt.Println(sum)
}

func scoreCombo(s string) int {
	switch s {
	case "A X":
		return 3 + 0
	case "A Y":
		return 1 + 3
	case "A Z":
		return 2 + 6
	case "B X":
		return 1 + 0
	case "B Y":
		return 2 + 3
	case "B Z":
		return 3 + 6
	case "C X":
		return 2 + 0
	case "C Y":
		return 3 + 3
	case "C Z":
		return 1 + 6
	default:
		panic("whoops")
	}
}
