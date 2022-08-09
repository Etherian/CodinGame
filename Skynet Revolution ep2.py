import sys
import math
from functools import reduce
from itertools import permutations
from queue import Queue

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
    boundary_queue = Queue(len(nodes))

    nodes[gateway]["gateways"][gateway]["dist"] = 0
    for gw_nbr in nodes[gateway]["neighbors"]:
        nodes[gw_nbr]["gateways"][gateway]["dist"] = 1
        nodes[gw_nbr]["gateways"][gateway]["prev"] = gateway

    while len(unvisited_nodes) > 0:
        min_node = min( unvisited_nodes,
                        key=lambda n: nodes[n]["gateways"][gateway]["dist"] )
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

# for gw in gateways:
#     print('\n-{}-\n'.format(gw) + '\n'.join('{}: {}, {}'.format( node,
#                                                                  nodes[node]["gateways"][gw]["dist"],
#                                                                  nodes[node]["gateways"][gw]["prev"] ) for node in nodes), file=sys.stderr)

severable_links = {gw: nodes[gw]['neighbors'] for gw in gateways}

twice_linked_nodes = {tw_l_n: {gwa,gwb} for gwa, gwb in permutations(gateways,r=2)
                                        for tw_l_n in (severable_links[gwa] & severable_links[gwb])}

# game loop
while True:
    si = int(input())  # The index of the node on which the Skynet agent is positioned this turn
    neighbor_dists = [ ( node,
                         min(nodes[node]["gateways"][gw]["dist"] for gw in gateways) )
                       for node in nodes[si]["neighbors"] ]

    # print(severable_links, file=sys.stderr)

    if si in reduce(set.union, severable_links.values()):
        # sever link to adjacent gateway
        dist, gw = min( (nodes[si]['gateways'][gw]['dist'], gw) for gw in gateways if si in severable_links[gw] )
        node = si
    elif twice_linked_nodes:
        # sever link of closest twice-linked node
        dist, closest_gw, closest_2L_node = min( (nodes[si]['gateways'][gw]['dist'], gw, node) for node in twice_linked_nodes for gw in twice_linked_nodes[node])
        gw, node = closest_gw, closest_2L_node
    else:
        # just sever some close-by link
        dist, gw, node = min( (nodes[si]['gateways'][gw]['dist'], gw, node) for gw in gateway for node in severable_links[gw] )

    severable_links[gw].remove(node)
    try:
        twice_linked_nodes.pop(node)
    except KeyError:
        pass
    print("{} {}".format(gw, node))
