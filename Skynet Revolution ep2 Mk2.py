from operator import itemgetter
from typing import NamedTuple
from collections import OrderedDict, namedtuple


# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

# Write an action using print
# To debug: print("Debug messages...", file=sys.stderr)

# n: the total number of nodes in the level, including the gateways
# l: the number of links
# e: the number of exit gateways
n, l, e = [int(i) for i in input().split()]

# class NodeInfo(NamedTuple):
#     nbrs
#     gwlinks
#
# class PFInfo(NamedTuple):
#     avg
#     cum
#     dist
#     prev

NodeInfo = namedtuple('NodeInfo', ['nbrs','gwlinks'])
PFInfo = namedtuple('PFInfo', ['avg','cum','dist','prev'])

nodes = {}
for i in range(n):
    nodes[i] = NodeInfo(nbrs=set(), gwlinks=0)

for i in range(l):
    # n1: N1 and N2 defines a link between these nodes
    n1, n2 = [int(j) for j in input().split()]
    nodes[n2].nbrs.add(n1)
    nodes[n1].nbrs.add(n2)

gateways = set()
for i in range(e):
    ei = int(input())  # the index of a gateway node
    gateways.add(ei)

for node in nodes:
    nodes[node] = NodeInfo(nodes[node].nbrs, len(gateways & nodes[node].nbrs))


def generate_path(loc,nodes):
    boundary_set = OrderedDict()
    finished_set = {}
    boundary_set[loc] = PFInfo(0, nodes[loc].gwlinks, 0, None)

    while boundary_set:
        curr_i, curr_v = max(boundary_set.items(), key=itemgetter(1))
        del boundary_set[curr_i]
        finished_set[curr_i] = curr_v
        # if curr_v.dist >= 5:
        #     continue

        for nbr in nodes[curr_i].nbrs.difference(finished_set):
            prev = curr_i
            dist = curr_v.dist + 1
            cum = curr_v.cum + nodes[nbr].gwlinks
            avg = cum / dist
            boundary_set[nbr] = PFInfo(avg, cum, dist, prev)

    curr = max(finished_set.items(), key=itemgetter(1))[0]
    path = []
    while curr is not None:
        path.append(curr)
        curr = finished_set[curr].prev
    path.reverse()

    return path

def choose_target(path, nodes):
    return max(path, key=lambda n: nodes[n].gwlinks)


for gw in gateways:
    print('\n-{}-\n'.format(gw) + '\n'.join('{}: {}'.format( node,
                                                             nodes[node].gwlinks ) for node in nodes), file=sys.stderr)


# game loop
while True:
    si = int(input())  # The index of the node on which the Skynet agent is positioned this turn

    if (nodes[si].nbrs & gateways):
        target = si
    else:
        target = choose_target(generate_path(si, nodes), nodes)

    nbr_gw = (nodes[target].nbrs & gateways).pop()
    nodes[target].nbrs.remove(nbr_gw)
    nodes[nbr_gw].nbrs.remove(target)
    nodes[target] = NodeInfo(nodes[target].nbrs, len(gateways & nodes[target].nbrs))

    print("{} {}".format(nbr_gw, target))
