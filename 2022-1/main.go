package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	scanner := bufio.NewScanner(os.Stdin)

	var sums []int
	var sum int = 0
	for scanner.Scan() {
		t := scanner.Text()

		if strings.TrimSpace(t) == "" {
			sums = append(sums, sum)
			sum = 0
		} else {
			i, _ := strconv.ParseInt(t, 10, 0)
			sum += int(i)
		}
	}
	if sum != 0 {
		sums = append(sums, sum)
		sum = 0
	}

	maxs := make([]int, 3)
	for _, sum := range sums {
		index := -1
		for maxi, max := range maxs {
			if sum > max {
				index = maxi
				break
			}
		}
		if index != -1 {
			for i := 1; i >= index; i -= 1 {
				maxs[i+1] = maxs[i]
			}
			maxs[index] = sum
		}
	}
	fmt.Println(maxs[0] + maxs[1] + maxs[2])
}
