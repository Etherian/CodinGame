import sys
import math

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.
abc = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ?'

l = int(input())
h = int(input())
t = input().upper()
print(l,h,t, sep='\n', file=sys.stderr)

charmap = {c: [] for c in abc}
for y in range(h):
    row = input()
    for j in range(len(abc)):
        lineseg = row[j*l : (j+1)*l]
        print(lineseg, file=sys.stderr)
        charmap[abc[j]].append(lineseg)

for y in range(h):
    for c in t:
        print(c, y, file=sys.stderr)
        try:
            print(charmap[c][y], end='')
        except KeyError:
            print(charmap[abc[26]][y], end='')
    print()
