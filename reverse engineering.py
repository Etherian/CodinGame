import sys
import math

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

init1 = int(input())
init2 = int(input())
init3 = int(input())
print(init1, init2, init3, file=sys.stderr)

# game loop
while True:
    char1 = input()
    char2 = input()
    char3 = input()
    char4 = input()
    print(char1, char2, char3, char4, file=sys.stderr)

    for i in range(init3):
        int1, int2 = [int(j) for j in input().split()]
        print(int1, int2, file=sys.stderr)

    # Write an action using print
    # To debug: print("Debug messages...", file=sys.stderr)

    print("A")
