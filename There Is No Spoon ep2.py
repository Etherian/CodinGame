import itertools as it
import math
import sys
from enum import IntEnum
from functools import total_ordering
from heapq import *
from operator import attrgetter, itemgetter, methodcaller
from typing import NamedTuple, Optional


# The machines are gaining ground. Time to show them what we're really made of...

# Write an action using print
# To debug: print("Debug messages...", file=sys.stderr)

# num possible bridge states = sum( (n+1)..0 ) where n = num slots (ie max links per bridge)
# ex where n = 2 :
#   L:  L:  L:  L:* L:* L:**
#   C:  C:* C:**C:  C:* C:


def info( *messages, **kargs ):
    print(*messages, file=sys.stderr)
    pass


def debug( *messages, **kargs ):
    print(*messages, file=sys.stderr)
    pass


class InvalidBridgeError(RuntimeError):
    pass


@total_ordering
class Node:
    def __init__( self, graph, coords, quota, ID ):
        self.graph = graph
        self.coords = coords
        self.quota = quota
        self.id = ID
        self.up = None
        self.dn = None
        self.lf = None
        self.rt = None

    def __str__( self ):
        return 'Node ' + str(self.id)

    def __gt__( self, other ):
        return hash(self) > hash(other)

    def _sum_bridges( self, func ):
        values = list(map( lambda j: func(j) if j else 0, self.get_bridges() ))
        # debug(values)
        return sum(values)

    def num_slots_open( self ):
        # debug('slots open:')
        return self._sum_bridges( Bridge.num_open_slots )

    def num_links( self ):
        # debug('slots linked:')
        return self._sum_bridges( Bridge.num_linked_slots )

    def num_slots_closed( self ):
        # debug('slots closed:')
        return self._sum_bridges( Bridge.num_linked_slots )

    def get_bridges( self ):
        return self.up, self.dn, self.lf, self.rt

    def rem_quota( self ):
        return self.quota - self.num_links()

    def apply_quota_rules( self ):
        bridges = self.get_bridges()
        quota = self.quota
        num_open = self.num_slots_open()
        # self.num_slots_closed()  # NOTE: for debug logging
        rem_quota = self.rem_quota()
        rem_close_quota = num_open - rem_quota

        for bridge in bridges:
            if bridge:
                disparity = bridge.num_open_slots() - rem_close_quota
                bridge.fill_slots('link', disparity)

                disparity = bridge.num_open_slots() - rem_quota
                bridge.fill_slots('close', disparity)


class Bridge:
    def __init__( self, graph, nodes ):
        self.graph = graph
        self.nodes = nodes
        # note: used to be represented by a tuple of Bools eg (F,T,T) => {1, 2}
        # Possible number of links this bridge might have. Every closed slot
        #   decreases the maximum number of possible links, and every linked
        #   slot increases the minimum number of possible links
        self.domain = {0, 1, 2}
        self.crosses = []

    def __str__( self ):
        a, b = self.nodes
        return '{}-{}'.format(a, b)

    def _add_to_queue( self ):
        self.graph.add_to_queue( MoveType.Node, *self.nodes )

    def check( self ):
        # domain must contain at least one possible state, and no move can
        #   leave entirely-closed or entirely-linked as the only two options
        if self.domain in (set(), {0, 2}):
            raise InvalidBridgeError('in invalid state: ' + str(self.domain))

    def num_open_slots( self ):
        return len(self.domain) - 1

    def num_linked_slots( self ):
        # the number of current links is the minimum number of possible links
        return min(self.domain)

    def num_closed_slots( self ):
        # the number of closed slots is the max num links - current max num
        return 2 - max(self.domain)

    def close_crossed( self ):
        if self.crosses:
            debug('closing crossed:')
        for b in self.crosses:
            debug(b)
            if 0 not in b.domain:
                raise InvalidBridgeError('attempted crossing of established link(s)')
            b.fill_slots('close', b.num_open_slots())

    def cap_max_links_at_lowest_quota( self ):
        minquota = min(node.quota for node in self.nodes)
        # self.domain = {elem for elem in self.domain if elem <= minquota}
        if minquota < 2:
            self.domain.remove(2)

    def fill_slots( self, how, how_many ):
        if how_many < 1:
            return

        info('{} {} {}'.format(how, how_many, self))

        if len(self.domain) < 2:
            raise InvalidBridgeError('attempted filling of full bridge {}, {}'.format(self,
                                                                                      self.domain))

        if how == 'close':
            self.domain = set(sorted(self.domain)[:-how_many])

            if self.domain == {0}:
                graph.unexternify_bridge(self)

        elif how == 'link':
            self.domain = set(sorted(self.domain)[how_many:])

            self.close_crossed()

            i1, i2 = (graph.get_island(node.id) for node in self.nodes)
            if i1 is not i2:
                graph.merge_islands(i1, i2)

        else:
            raise ValueError('must `link` or `close` slots, got '+repr(how))

        # The connected nodes' states have changed; add them to the queue to be
        #   processed again
        self._add_to_queue()

        self.check()


class Island:
    def __init__( self, nodes, ext_bridges, iid ):
        self.nodes = nodes
        # self.ext_nodes = ext_nodes
        # self.int_bridges = int_bridges
        self.ext_bridges = ext_bridges
        self.id = iid

    @staticmethod
    def from_node( init_node ):
        nodes = {init_node}
        # int_bridges = set()
        ext_bridges = {bridge for bridge in init_node.get_bridges()
                       if bridge}
        iid = init_node.id
        return Island(nodes, ext_bridges, iid)


class MoveType(IntEnum):
    Node = 1
    SoleExternalBridge = 2
    BtwIsland = 3
    WithinIsland = 4
    Guess = 5


# class Move(NamedTuple):
#     """Docs for Move"""
#     type: MoveType
#     which: Optional[Node]
Move = NamedTuple('Move', [('type', MoveType), ('elem', Optional[Node])])
Move.__str__ = lambda self: 'Move({!s}, {})'.format(self.type, self.elem)


# TODO Add heuristic to MoveQueue? eg min(rem_quota, rem_close_quota)?
class MoveQueue:
    def __init__( self ):
        self.heap = []
        self.set = set()

    def push( self, move ):
        if move not in self.set:
            # debug('pushed ({!s}, {})'.format(move.type, move.elem))
            heappush(self.heap, move)
            self.set.add(move)

    def pop( self ):
        try:
            move = heappop(self.heap)
            self.set.remove(move)
            return move
        except IndexError:
            return None

    def is_empty( self ):
        return len(self.heap) == 0


class Graph:
    def __init__( self, puzzle ):
        if not all(len(row) == len(puzzle[0]) for row in puzzle):
            raise ValueError('received puzzle is not rectangular')

        # NOTE/FIXME Proper island merging currently requires
        #   node index == node id because it is assumed
        #   island index == island id, and node id is used to set island id in
        #   island initialization
        self.nodes = []
        self.bridges = []
        self.islands = []
        self._num_islands = None
        self.move_queue = MoveQueue()

        height = len(puzzle)
        width = len(puzzle[0])

        oldys = [None] * width
        nodeID = 0
        horz_bridges = [[] for _ in range(height)]
        vert_bridges = [[] for _ in range(width)]
        for y in range(len(puzzle)):
            line = puzzle[y]  # `width` characters, each either a number or a '.'
            oldx = None

            for x in range(width):
                char = line[x]

                if char != '.':
                    # info('node {} at ({},{})'.format(nodeID, x, y))
                    new = Node( self, (x,y), int(char), nodeID )
                    nodeID += 1
                    self.nodes.append(new)

                    if oldx:
                        new_bridge = Bridge( self, (oldx, new) )
                        # info('horz bridge {}'.format(new_bridge))

                        horz_bridges[y].append(new_bridge)

                        new.lf = new_bridge
                        oldx.rt = new_bridge
                        self.bridges.append(new_bridge)
                    oldx = new

                    oldy = oldys[x]
                    if oldy:
                        new_bridge = Bridge( self, (oldy, new) )
                        # info('vert bridge {}'.format(new_bridge))

                        # Check if bridge crosses any previous bridges, if so add them
                        #   to each other's crosses lists. This check would also occur in
                        #   horizontal bridge formation, but the vertical of the cross
                        #   will always form second, due to the structure of the input.
                        vert_bridges[x].append(new_bridge)
                        # debug((oldy.coords[1] + 1, new.coords[1]))
                        for yspan in range(oldy.coords[1] + 1, new.coords[1]):
                            for hbridge in horz_bridges[yspan]:
                                x1, x2 = sorted(node.coords[0] for node in hbridge.nodes)
                                if x1 < x < x2:
                                    # debug('{} X {}'.format(new_bridge, hbridge))
                                    new_bridge.crosses.append(hbridge)
                                    hbridge.crosses.append(new_bridge)

                        new.up = new_bridge
                        oldy.dn = new_bridge
                        self.bridges.append(new_bridge)
                    oldys[x] = new

        self.init_islands()

        # The basic rules cover this special situation, but this might speed things up
        for bridge in self.bridges:
            bridge.cap_max_links_at_lowest_quota()

        self.add_to_queue(MoveType.Guess, None)
        self.add_to_queue(MoveType.WithinIsland, None)
        self.add_to_queue(MoveType.BtwIsland, None)
        self.add_to_queue(MoveType.SoleExternalBridge, None)
        # start off by queueing all nodes
        self.add_to_queue(MoveType.Node, *self.nodes)

    def init_islands( self ):
        self.islands = [Island.from_node(node) for node in self.nodes]
        self._num_islands = len(self.islands)
        debug('num islands='+str(self._num_islands))

    def add_to_queue( self, move_type, *elems ):
        for elem in elems:
            self.move_queue.push(Move(move_type, elem))

    def pop_next_move( self ):
        return self.move_queue.pop()

    def solve( self ):
        while not self.move_queue.is_empty():
            curr_move = self.pop_next_move()
            info('\npopped {}'.format(curr_move))
            debug('num islands='+str(self._num_islands))

            if curr_move.type == MoveType.Node:
                info('checking {}'.format(curr_move.elem))
                curr_move.elem.apply_quota_rules()

            # If an island has only one external bridge left, that bridge must connect it to other
            #   islands
            elif curr_move.type == MoveType.SoleExternalBridge and self._num_islands > 1:
                for island in self.islands:
                    if ( isinstance(island, Island) and
                         len(island.ext_bridges) == 1 ):
                        xb = next(iter(island.ext_bridges))
                        xb.fill_slots('link', 1)
                        self.add_to_queue(MoveType.SoleExternalBridge, None)
                        break

            # As long as multiple islands exist, each one must have at least one external node, so
            #   no two islands with only one external node each can fill their respective quotas
            #   with each other, as this would leave the resulting island without any external nodes.
            elif curr_move.type == MoveType.BtwIsland and self._num_islands > 2:
                islands = [island for island in self.islands if isinstance(island,Island)]
                xbridges = {xb for island in islands for xb in island.ext_bridges}
                xnodes = {xn for xb in xbridges for xn in xb.nodes}
                # debug('xnodes: {}'.format(xnodes))
                xn_by_isl = [island.nodes & xnodes for island in islands]
                # debug('xn_by_isl: {}'.format(xn_by_isl))
                lonely_xns = {xns.pop() for xns in xn_by_isl if len(xns) == 1}
                # debug('lonely_xns: {}'.format(lonely_xns))
                for xb in xbridges:
                    n1, n2 = xb.nodes
                    if n1 in lonely_xns and n2 in lonely_xns and n1.rem_quota() == n2.rem_quota():
                        to_close = xb.num_open_slots() - n1.rem_quota() + 1
                        if to_close > 0:
                            xb.fill_slots('close', to_close)
                            self.add_to_queue(MoveType.BtwIsland, None)
                            self.add_to_queue(MoveType.SoleExternalBridge, None)
                            break

            # Similar to the above, except it's the last two external nodes on the same island that
            #   can't fill each other
            elif curr_move.type == MoveType.WithinIsland and self._num_islands > 1:
                islands = [island for island in self.islands if isinstance(island, Island)]
                xnodes = {xn for island in islands for xb in island.ext_bridges for xn in xb.nodes}
                xn_by_isl = [island.nodes & xnodes for island in islands]
                xn_pairs = [xns for xns in xn_by_isl if len(xns) == 2]
                for xn_pair in xn_pairs:
                    xn1, xn2 = xn_pair
                    shared_bridge = None
                    try:
                        shared_bridge = ( {b for b in xn1.get_bridges() if b}
                                          & {b for b in xn2.get_bridges() if b} ).pop()
                    except KeyError:
                        pass
                    if shared_bridge:
                        rem_quota = max((xn1.rem_quota(), xn2.rem_quota()))
                        open_slots = shared_bridge.num_open_slots()
                        to_close = open_slots - rem_quota + 1
                        if to_close > 0:
                            shared_bridge.fill_slots('close', to_close)
                            self.add_to_queue(MoveType.WithinIsland, None)
                            self.add_to_queue(MoveType.BtwIsland, None)
                            self.add_to_queue(MoveType.SoleExternalBridge, None)
                            break

            elif curr_move.type == MoveType.Guess:
                for b in self.bridges:
                    if len(b.domain) > 1:
                        b.fill_slots('link', 1)
                        self.add_to_queue(MoveType.Guess, None)
                        self.add_to_queue(MoveType.WithinIsland, None)
                        self.add_to_queue(MoveType.BtwIsland, None)
                        self.add_to_queue(MoveType.SoleExternalBridge, None)
                        break

    def merge_islands( self, i1, i2 ):
        minid, maxid = sorted((i1.id, i2.id))
        debug('merging islands {} & {}'.format(minid, maxid))
        new = Island( nodes = i1.nodes | i2.nodes,
                      ext_bridges = i1.ext_bridges ^ i2.ext_bridges,
                      iid = minid )
        self.islands[minid] = new
        self.islands[maxid] = minid
        self._num_islands -= 1

    def get_island( self, nodeid ):
        val = nodeid
        debug('get island, node '+str(nodeid))
        while isinstance(val, int):
            val = self.islands[val]
        if not isinstance(val, Island):
            raise RuntimeError('not Island or int: ' + str(val))
        debug('island '+str(val.id))
        return val

    def unexternify_bridge(self, bridge):
        for node in bridge.nodes:
            try:
                self.get_island(node.id).ext_bridges.remove(bridge)
            except KeyError:
                pass

    def print_solution( self ):
        for bridge in self.bridges:
            node1, node2 = bridge.nodes
            x1, y1 = node1.coords
            x2, y2 = node2.coords
            l = next(iter(bridge.domain))

            if l:
                # Two coordinates and one integer: a node, one of its neighbors, the number of links
                #   connecting them.
                print('{} {} {} {} {}'.format(x1, y1, x2, y2, l))



###############################
###     Puzzles             ###
###############################

intermediate_1_puzzle = '''4.544
.2...
..5.4
332..'''

multiple_solutions_puzzle = '''3.3
...
3.3'''

advanced_puzzle = '''3.4.6.2.
.1......
..2.5..2
1.......
..1.....
.3..52.3
.2.17..4
.4..51.2'''

cg_puzzle = '''22221
2....
2....
2....
2....
22321
.....
.....
22321
2....
2....
2.131
2..2.
2222.'''

multiple_solutions_2_puzzle = '''.12..
.2421
24442
1242.
..21.'''

expert_puzzle = '''3..2.2..1....3........4
.2..1....2.6.........2.
..3..6....3............
.......2........1..3.3.
..1.............3..3...
.......3..3............
.3...8.....8.........3.
6.5.1...........1..3...
............2..6.31..2.
..4..4.................
5..........7...7...3.3.
.2..3..3..3............
......2..2...1.6...3...
....2..................
.4....5...3............
.................2.3...
.......3.3..2.44....1..
3...1.3.2.3............
.2.....3...6.........5.
................1......
.1.......3.6.2...2...4.
5...............3.....3
4...................4.2'''



###############################
###     Main                ###
###############################

width = int(input())  # the number of cells on the X axis
height = int(input())  # the number of cells on the Y axis
puzzle = [input() for _ in range(height)]

graph = Graph(puzzle)

graph.solve()

graph.print_solution()
