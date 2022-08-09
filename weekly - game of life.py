import sys
import math
import numpy as np

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

def clamp(x, bound1, bound2):
    return sorted([bound1, x, bound2])[1]

width, height = [int(i) for i in input().split()]
mat = np.zeros( (height, width), np.uint8)
for y in range(height):
    line = input()
    mat[y,...] = np.fromiter((int(n) for n in line), np.uint8, count=width)

newmat = np.zeros_like(mat)
for y in range(height):
    for x in range(width):
        curr = mat[y, x]
        lowy = max(y-1, 0)
        highy = min(y+2, height)
        lowx = max(x-1, 0)
        highx = min(x+2, width)
        nbrs = sum(np.nditer(mat[lowy:highy, lowx:highx])) - curr
        # print('{}:{},{}:{}={}'.format(lowy, highy, lowx, highx, nbrs), file=sys.stderr)
        if curr:
            if 1 < nbrs < 4:
                new = 1
            else:
                new = 0
        else:
            if nbrs == 3:
                new = 1
            else:
                new = 0
        newmat[y, x] = new
        print(new, end='')
    print()

# Write an action using print
# To debug: print("Debug messages...", file=sys.stderr)
