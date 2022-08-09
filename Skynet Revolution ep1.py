import sys
import math
from functools import reduce
from random import choice
# from collections import defaultdict

Inf = float("inf")

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

# Write an action using print
# To debug: print("Debug messages...", file=sys.stderr)

# n: the total number of nodes in the level, including the gateways
# l: the number of links
# e: the number of exit gateways
n, l, e = [int(i) for i in input().split()]

nodes = {}
for i in range(n):
    nodes[i] = { "neighbors": set(),
                 "gateways": {} }

for i in range(l):
    # n1: N1 and N2 defines a link between these nodes
    n1, n2 = [int(j) for j in input().split()]
    nodes[n1]["neighbors"].add(n2)
    nodes[n2]["neighbors"].add(n1)

def find_distances( nodes, gateway ):
    unvisited_nodes = set(i for i in range(n)) - {gateway}

    nodes[gateway]["gateways"][gateway]["dist"] = 0
    for gw_nbr in nodes[gateway]["neighbors"]:
        nodes[gw_nbr]["gateways"][gateway]["dist"] = 1
        nodes[gw_nbr]["gateways"][gateway]["prev"] = gateway

    while len(unvisited_nodes) > 0:
        min_node = sorted( unvisited_nodes,
                           key=lambda n: nodes[n]["gateways"][gateway]["dist"] )[0]
        unvisited_nodes.remove(min_node)
        # print("min_node: {}".format(min_node), file=sys.stderr)

        new_dist = nodes[min_node]["gateways"][gateway]["dist"] + 1
        # print("new_dist: {}".format(new_dist), file=sys.stderr)
        # print("neighbors: {}".format((nodes[min_node]["neighbors"] & unvisited_nodes)), file=sys.stderr)
        for nbr in (nodes[min_node]["neighbors"] & unvisited_nodes):
            # print("nbr dist: {}".format(nodes[nbr]["gateways"][gateway]["dist"]), file=sys.stderr)
            if new_dist < nodes[nbr]["gateways"][gateway]["dist"]:
                nodes[nbr]["gateways"][gateway]["dist"] = new_dist
                nodes[nbr]["gateways"][gateway]["prev"] = min_node

gateways = set()
for i in range(e):
    ei = int(input())  # the index of a gateway node
    gateways.add(ei)
    for node in nodes:
        nodes[node]["gateways"][ei] = {"dist": Inf, "prev": None}
    find_distances(nodes, ei)

for gw in gateways:
    print('\n-{}-\n'.format(gw) + '\n'.join('{}: {}, {}'.format( node,
                                                                 nodes[node]["gateways"][gw]["dist"],
                                                                 nodes[node]["gateways"][gw]["prev"] ) for node in nodes), file=sys.stderr)

try:
    trap_gw = [gw for gw in gateways if len(nodes[gw]["neighbors"]) > 9].pop()
    # trap_gw = choice([gw for gw in gateways if len(nodes[gw]["neighbors"]) >= 8])

    # get the outer nodes in trap_gw's wheel, ie nodes next to trap_gw with 3 neighbors
    star_nodes = set( node for node in nodes if nodes[node]["gateways"][trap_gw]["dist"] == 1 and
                                                len(nodes[node]["neighbors"]) == 3 )
    print(star_nodes, file=sys.stderr)
    # filter out star nodes which neighbor non-star nodes
    star_nodes = set(node for node in star_nodes if (nodes[node]["neighbors"] - {trap_gw}) < star_nodes)
    print(star_nodes, file=sys.stderr)
except IndexError:
    trap_gw = None

severed_links = set()

print(trap_gw, file=sys.stderr)
# game loop
while True:
    si = int(input())  # The index of the node on which the Skynet agent is positioned this turn
    neighbor_dists = [ ( node,
                         min(nodes[node]["gateways"][gw]["dist"] for gw in gateways) )
                       for node in nodes[si]["neighbors"] ]

    if all(node[1] > 0 for node in neighbor_dists) and trap_gw is not None:
        curr_node = star_nodes.pop()
        other_node = (nodes[curr_node]["neighbors"] & star_nodes).pop()
        severed_links.add(frozenset((curr_node, other_node)))
        print("{} {}".format(curr_node, other_node))
    else:
        # filter out previously-severed links
        severed_neighbors = reduce(set.union, [link for link in severed_links if si in link], set())
        neighbor_dists = [nbr for nbr in neighbor_dists if nbr[0] not in severed_neighbors]

        most_dangerous_neighbor = sorted(neighbor_dists, key=lambda x: x[1])[0][0]

        severed_links.add(frozenset((si, most_dangerous_neighbor)))
        print("{} {}".format(si, most_dangerous_neighbor))
