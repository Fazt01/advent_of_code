package main

import (
	"bufio"
	"fmt"
	"os"
)

func main0() {
	scanner := bufio.NewScanner(os.Stdin)

	var sum int = 0
	for scanner.Scan() {
		t := scanner.Text()
		half := len(t) / 2
		set1 := compToSet(t[0:half])
		set2 := compToSet(t[half:])

		res := map[int]struct{}{}
		for v := range set2 {
			if _, ok := set1[v]; ok {
				res[v] = struct{}{}
			}
		}
		for v := range res {
			sum += v
		}
	}

	fmt.Println(sum)
}

func main() {
	scanner := bufio.NewScanner(os.Stdin)

	var sum int = 0
	for scanner.Scan() {
		b1 := scanner.Text()
		scanner.Scan()
		b2 := scanner.Text()
		scanner.Scan()
		b3 := scanner.Text()

		set1 := compToSet(b1)
		set2 := compToSet(b2)
		set3 := compToSet(b3)

		for v := range set1 {
			_, ok := set2[v]
			_, ok2 := set3[v]
			if ok && ok2 {
				sum += v
			}
		}
	}

	fmt.Println(sum)
}

func mapChar(r rune) int {
	if r >= 'a' && r <= 'z' {
		return int(r - 'a' + 1)
	}
	return int(r - 'A' + 27)
}

func compToSet(s string) map[int]struct{} {
	res := map[int]struct{}{}
	for _, r := range s {
		res[mapChar(r)] = struct{}{}
	}
	return res
}
