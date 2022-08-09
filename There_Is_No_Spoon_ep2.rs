use std::io;
use std::cell::RefCell;
use std::str::FromStr;
use std::cmp;
use std::fmt;
use std::fmt::{Debug, Display};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::error::Error;
// use std::{thread, time};
use std::rc::{Rc, Weak};
use std::collections::HashSet;


// macro_rules! print_err {
//     ($($arg:tt)*) => (
//         {
//             use std::io::Write;
//             writeln!(&mut ::std::io::stderr(), $($arg)*).ok();
//         }
//     )
// }

fn debug(msg: &str) {
    // writeln!(&mut ::std::io::stderr(), "{}", msg).ok();
    println!("{}", msg);
}
fn info(msg: &str) {
    writeln!(&mut ::std::io::stderr(), "{}", msg).ok();
}



fn parse_line<T>() -> Result<T, Box<Error>>
    where T: FromStr, <T as FromStr>::Err: 'static + Debug + Error,
{
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let res = input_line.trim().parse::<T>()?;
    Ok(res)
}

fn read_line() -> String {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    input_line.trim_right().to_string()
}


enum HashiError {
    InitError(String),
}

struct HashiGraph {
    nodes: Vec<Rc<Node>>,
    bridges: Vec<Rc<Bridge>>,
    islands: Vec<IslandSlot>,
    node_queue: Vec<NdRef>,
}

enum IslandSlot {
    Isl(Island),
    IslPtr(usize),
}

type BrRef = Weak<Bridge>;
type NdRef = Weak<Node>;
type ID = usize;

struct Node {
    graph: *mut HashiGraph,
    coords: (u32, u32),
    quota: u32,
    id: ID,
    up: RefCell<BrRef>,
    dn: RefCell<BrRef>,
    lf: RefCell<BrRef>,
    rt: RefCell<BrRef>,
}

struct Bridge {
    graph: *mut HashiGraph,
    domain: RefCell<Domain>,
    nodes: (NdRef, NdRef),
    crosses: RefCell<Vec<BrRef>>,
    id: ID,
}
type Domain = [bool; 3];

struct Island {
    nodes: HashSet<ID>,
    ext_bridges: HashSet<ID>,
    id: ID,
}

// NB The within-island shared-bridge check depends on the order of this.
//  Specifically, that it be a complementary palindrome, lf to rt, up to dn.
macro_rules! bridge_iter {
    ($node:expr) => {{
        vec![
            $node.up.borrow().upgrade(),
            $node.lf.borrow().upgrade(),
            $node.rt.borrow().upgrade(),
            $node.dn.borrow().upgrade(),
        ].into_iter()
    }};
}

macro_rules! culled_bridge_iter {
    ($node:expr) => {{
        bridge_iter!($node).filter_map(|b| b)
    }};
}

macro_rules! sum_bridges {
    ( $node:expr, $f:ident ) => {{
        culled_bridge_iter!($node).map(|b| b.$f()).sum::<u32>()
    }};
}

trait HashiDomain {
    fn new() -> Self;

    fn num_slots(&self) -> u32;
    fn num_open_slots(&self) -> u32;
    fn num_closed_slots(&self) -> u32;
    fn num_linked_slots(&self) -> u32;

    fn link_slots(&mut self, n: u32) -> Result<(), String>;
    fn close_slots(&mut self, n: u32) -> Result<(), String>;
}

impl HashiGraph {
    fn new<T: AsRef<str>>(puzzle: &[T]) -> Self {
        let mut graph = HashiGraph {
            nodes: Vec::new(),
            bridges: Vec::new(),
            islands: Vec::new(),
            node_queue: Vec::new(),
        };

        if puzzle.is_empty() {
            panic!("puzzle is empty");
        }

        let height = puzzle.len();
        let width = puzzle[0].as_ref().len();

        if !puzzle.iter().all(|row| row.as_ref().len() == width) {
            panic!("puzzle is not rectangular");
        }

        let mut oldys: Vec<NdRef> = vec![Weak::new(); width]; // Weak::new() always holds None
        let mut horz_bridges: Vec<Vec<Rc<Bridge>>> = vec![Vec::with_capacity(width); height];
        let mut vert_bridges: Vec<Vec<Rc<Bridge>>> = vec![Vec::with_capacity(height); width];
        for (y, ref_to_line) in puzzle.iter().enumerate() {
            let line = ref_to_line.as_ref();
            let mut oldx: NdRef = Weak::new(); // Weak::new() always holds None

            for (x, puzz_char) in line.chars().enumerate() {
                if let Some(number) = puzz_char.to_digit(10) {
                    let new_node: NdRef = graph._add_node((x as u32, y as u32), number);
                    // debug("node!");

                    if let Some(oldx_node_rc) = oldx.upgrade() {
                        // debug("horz!");
                        let new_bridge: BrRef = graph._add_bridge((Rc::downgrade(&oldx_node_rc), new_node.clone()));
                        let new_node_rc = new_node.upgrade().unwrap();
                        let new_bridge_rc = new_bridge.upgrade().unwrap();

                        horz_bridges[y].push(new_bridge_rc.clone());

                        *new_node_rc.lf.borrow_mut() = new_bridge.clone();
                        *oldx_node_rc.rt.borrow_mut() = new_bridge.clone();
                    }

                    if let Some(oldy_node_rc) = oldys[x].upgrade() {
                        // debug("vert!");
                        let new_bridge: BrRef = graph._add_bridge((Rc::downgrade(&oldy_node_rc), new_node.clone()));
                        let new_bridge_rc = new_bridge.upgrade().unwrap();
                        let new_node_rc = new_node.upgrade().unwrap();

                        // debug(&format!("vert_bridges len:{}", oldys.len()));
                        vert_bridges[x].push(new_bridge_rc.clone());

                        // Check if bridge crosses any previous bridges
                        for span_y in (oldy_node_rc.coords.1 + 1)..new_node_rc.coords.1 {
                            for hbridge in horz_bridges.get(span_y as usize).unwrap() {
                                let (ref n1, ref n2) = hbridge.nodes;
                                let (x1, x2) = (n1.upgrade().unwrap().coords.0, n2.upgrade().unwrap().coords.0);
                                let (xa, xb) = if x1 < x2 {(x1, x2)} else {(x2, x1)};
                                if xa < (x as u32) && (x as u32) < xb {
                                    new_bridge_rc.crosses.borrow_mut().push(Rc::downgrade(&hbridge));
                                    hbridge.crosses.borrow_mut().push(Rc::downgrade(&new_bridge_rc));
                                }
                            }
                        }

                        *new_node_rc.up.borrow_mut() = new_bridge.clone();
                        *oldy_node_rc.dn.borrow_mut() = new_bridge.clone();
                    }
                    // debug("past oldys");

                    oldx = new_node.clone();
                    oldys[x] = new_node.clone();
                    // debug(&format!("oldys len:{}", oldys.len()));
                }
            }
        }

        graph.node_queue = graph.nodes.iter().rev().map(|ndrc| Rc::downgrade(ndrc)).collect();

        graph
    }

    fn _add_node(&mut self, coords: (u32, u32), quota: u32) -> NdRef {
        let node_id = self.nodes.len();
        let new_node = Rc::new(Node::new(self, coords, quota, node_id));
        self._add_island_for(Rc::downgrade(&new_node));
        let return_ref = Rc::downgrade(&new_node);
        self.nodes.push(new_node);
        return_ref
    }

    fn _add_bridge(&mut self, nodes: (NdRef, NdRef)) -> BrRef {
        let bridge_id = self.bridges.len();
        let new_bridge = Rc::new(Bridge::new(self, nodes, bridge_id));
        let return_ref = Rc::downgrade(&new_bridge);
        self.bridges.push(new_bridge);
        return_ref
    }

    fn _add_island_for(&mut self, node: Weak<Node>) {
        self.islands.push(IslandSlot::Isl(Island::from_node(node)));
    }

    fn _get_island(&self, node_id: usize) -> &Island {
        let mut index = node_id;
        let mut visited = vec![false; self.islands.len()];
        loop {
            match self.islands[index] {
                IslandSlot::IslPtr(new_index) => {
                    visited[index] = true;
                    if visited[new_index] {panic!("Infinite loop!")}
                    index = new_index;
                },
                IslandSlot::Isl(ref isl) => return isl,
            }
        }
    }

    fn _merge_islands(&mut self, i1: &Island, i2: &Island) {
        let (minid, maxid) = if i1.id < i2.id {(i1.id, i2.id)} else {(i2.id, i1.id)};
        let new_island = Island {
            nodes: i1.nodes.union(&i2.nodes).cloned().collect(),
            ext_bridges: i1.ext_bridges.intersection(&i2.ext_bridges).cloned().collect(),
            id: minid,
        };
        self.islands[minid] = IslandSlot::Isl(new_island);
        self.islands[maxid] = IslandSlot::IslPtr(minid);
    }

    fn _num_islands(&self) -> usize {
        self.islands.iter().filter(|slot| match slot {
            &&IslandSlot::Isl(_) => true,
            &&IslandSlot::IslPtr(_) => false
        }).count()
    }

    fn get_bridge_from_id(&self, id: ID) -> Rc<Bridge> {
        self.bridges[id].clone()
    }

    fn get_node_from_id(&self, id: ID) -> Rc<Node> {
        self.nodes[id].clone()
    }

    fn solve(&mut self) {
        // unimplemented!();
        'solve: loop {
            if let Some(curr_node) = self.node_queue.pop() {
                curr_node.upgrade().unwrap().apply_quota_rules();
                continue 'solve
            }
            // Sole External Bridge
            if self._num_islands() > 1 {
                for island_slot in &self.islands {
                    if let &IslandSlot::Isl(ref island) = island_slot {
                        if island.ext_bridges.len() == 1 {
                            let &br_id = island.ext_bridges.iter().next().unwrap();
                            self.get_bridge_from_id(br_id).link_slots(1);
                            continue 'solve
                        }
                    }
                }
            }
            // Between Islands
            if self._num_islands() > 2 {
                let islands = self.islands.iter()
                    .filter_map(|slot| match slot {
                        &IslandSlot::Isl(ref isl) => Some(isl),
                        &IslandSlot::IslPtr(_) => None
                    })
                    .collect::<Vec<_>>();
                let ext_bridge_ids = islands.iter()
                    .flat_map(|isl| isl.ext_bridges.iter().cloned())
                    .collect::<HashSet<_>>();
                let ext_node_ids = ext_bridge_ids.iter()
                    .flat_map(|&br| {
                        let nodes = &self.get_bridge_from_id(br).nodes;
                        vec![
                            nodes.0.upgrade().unwrap().id,
                            nodes.1.upgrade().unwrap().id
                        ].into_iter()
                    })
                    .collect::<HashSet<_>>();
                let ext_nodes_by_isl = islands.iter()
                    .map(|isl| &isl.nodes & &ext_node_ids)
                    .collect::<Vec<_>>();
                let lonely_ext_node_ids = ext_nodes_by_isl.iter()
                    .filter_map(|xnodes| {
                        if xnodes.len() == 1 {Some(*xnodes.iter().next().unwrap())} else {None}
                    })
                    .collect::<Vec<_>>();
                for ext_bridge_id in ext_bridge_ids {
                    let ext_bridge = self.get_bridge_from_id(ext_bridge_id);
                    let (ref n1, ref n2) = ext_bridge.nodes;
                    let (n1, n2) = (n1.upgrade().unwrap(), n2.upgrade().unwrap());
                    if lonely_ext_node_ids.contains(&n1.id) &&
                       lonely_ext_node_ids.contains(&n2.id) &&
                       n1.rem_quota() == n2.rem_quota() {
                        let num_to_close = ext_bridge.num_open_slots() - n1.rem_quota() + 1;
                        if num_to_close > 0 {
                            ext_bridge.link_slots(num_to_close);
                            continue 'solve
                        }
                    }
                }
            }
            // Within Islands
            if self._num_islands() > 1 {
                let islands = self.islands.iter()
                    .filter_map(|slot| match slot {
                        &IslandSlot::Isl(ref isl) => Some(isl),
                        &IslandSlot::IslPtr(_) => None
                    })
                    .collect::<Vec<_>>();
                let ext_node_ids = islands.iter()
                    .flat_map(|isl| isl.ext_bridges.iter().cloned())
                    .flat_map(|br_id| {
                        let nodes = &self.get_bridge_from_id(br_id).nodes;
                        vec![
                            nodes.0.upgrade().unwrap().id,
                            nodes.1.upgrade().unwrap().id
                        ].into_iter()
                    })
                    .collect::<HashSet<_>>();
                let ext_nodes_by_isl = islands.iter()
                    .map(|isl| (&isl.nodes & &ext_node_ids).into_iter().collect())
                    .collect::<Vec<Vec<ID>>>();
                let twin_ext_nodes = ext_nodes_by_isl.iter()
                    .filter(|ext_nodes| ext_nodes.len() == 2)
                    .map(|xn_pair| (xn_pair[0], xn_pair[1]))
                    .collect::<Vec<(ID, ID)>>();

                for (n1_id, n2_id) in twin_ext_nodes {
                    let n1 = self.get_node_from_id(n1_id);
                    let n2 = self.get_node_from_id(n2_id);

                    let brs1 = bridge_iter!(n1)
                        .map(|b| b.map(|b| b.id));
                    let brs2 = bridge_iter!(n2)
                        .map(|b| b.map(|b| b.id))
                        .rev();
                    let shared_bridge = brs1.zip(brs2)
                        .find(|&(b1, b2)| b1.is_some() && (b1 == b2));

                    if let Some((Some(sh_br_id), _)) = shared_bridge {
                        let sh_br = self.get_bridge_from_id(sh_br_id);
                        let rem_quota = cmp::max(n1.rem_quota(), n2.rem_quota());
                        let to_close = sh_br.num_open_slots() - rem_quota + 1;
                        if to_close > 0 {
                            sh_br.close_slots(to_close);
                            continue 'solve
                        }
                    }
                }
            }
            // Screw it. Just guess.
            for ref bridge in &self.bridges {
                if bridge.num_open_slots() > 0 {
                    bridge.link_slots(1);
                    // println!("guess used");
                    continue 'solve
                }
            }

            break;
        }
    }

    fn solution_iter<'s>(&'s self) -> impl Iterator<Item=String> + 's {
        fn stringify_bridge(bridge: &Rc<Bridge>) -> Option<String> {
            let (x1, y1) = bridge.nodes.0.upgrade().unwrap().coords;
            let (x2, y2) = bridge.nodes.1.upgrade().unwrap().coords;
            let links = bridge.num_linked_slots();

            if links > 0 {
                // Two coordinates and one integer: a node, one of its neighbors, the number of links
                //   connecting them.
                // println!("0 0 2 0 1");
                Some(format!("{} {} {} {} {}", x1, y1, x2, y2, links))
            } else {
                None
            }
        }
        self.bridges.iter().filter_map(stringify_bridge)
    }

    fn print_solution(&self) {
        for line in self.solution_iter() {
            println!("{}", line);
        }
    }
}

impl Node {
    fn new(graph: &mut HashiGraph, coords: (u32, u32), quota: u32, node_id: usize) -> Self {
        Node {
            graph: graph as *mut HashiGraph,
            coords: coords,
            quota: quota,
            id: node_id,
            up: RefCell::new(Weak::new()),
            dn: RefCell::new(Weak::new()),
            lf: RefCell::new(Weak::new()),
            rt: RefCell::new(Weak::new()),
        }
    }

    fn set_bridge(&self, bridge: BrRef, dir: &str) {
        match dir {
            "up" => *self.up.borrow_mut() = bridge,
            "dn" => *self.dn.borrow_mut() = bridge,
            "lf" => *self.lf.borrow_mut() = bridge,
            "rt" => *self.rt.borrow_mut() = bridge,
            _ => panic!("dir must be up, dn, lf, rt"),
        }
    }

    fn num_open_slots(&self) -> u32 {
        sum_bridges!(self, num_open_slots)
    }
    fn num_linked_slots(&self) -> u32 {
        sum_bridges!(self, num_linked_slots)
    }
    fn num_closed_slots(&self) -> u32 {
        sum_bridges!(self, num_closed_slots)
    }

    fn rem_quota(&self) -> u32 {
        self.quota - self.num_linked_slots()
    }
    fn close_quota(&self) -> u32 {
        sum_bridges!(self, num_slots) - self.quota
    }
    fn rem_close_quota(&self) -> u32 {
        self.close_quota() - self.num_closed_slots()
    }
    fn apply_quota_rules(&self) {
        // debug("begin apply_quota_rules");
        // debug(&format!("quota: {}\n open: {}\n linked: {}\n closed: {}\n rem_quota: {}\n close_quota: {}\n rem_close_quota: {}",
        //     self.quota,
        //     self.num_open_slots(),
        //     self.num_linked_slots(),
        //     self.num_closed_slots(),
        //     self.rem_quota(),
        //     self.close_quota(),
        //     self.rem_close_quota()
        // ));

        for mut bridge in culled_bridge_iter!(self) {
            // debug(&format!("applying to {}", bridge));

            let disparity = bridge.num_open_slots().saturating_sub(self.rem_close_quota());
            // debug(&format!("open_slots: {} - rem_close_quota: {} = linking {} slots",
            //     bridge.num_open_slots(),
            //     self.rem_close_quota(),
            //     disparity));
            bridge.link_slots(disparity).unwrap();

            let disparity = bridge.num_open_slots().saturating_sub(self.rem_quota());
            // debug(&format!("open_slots: {} - rem_quota: {} = closing {} slots",
            //     bridge.num_open_slots(),
            //     self.rem_quota(),
            //     disparity));
            bridge.close_slots(disparity).unwrap();
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "N{}", self.id)
    }
}

impl Bridge {
    fn new(graph: &mut HashiGraph, nodes: (NdRef, NdRef), id: ID) -> Self {
        Bridge {
            graph: graph as *mut HashiGraph,
            nodes: nodes,
            domain: RefCell::new(Domain::new()),
            crosses: RefCell::new(Vec::new()),
            id: id,
        }
    }
    fn num_slots(&self) -> u32 {
        self.domain.borrow().num_slots()
    }
    fn num_open_slots(&self) -> u32 {
        self.domain.borrow().num_open_slots()
    }
    fn num_closed_slots(&self) -> u32 {
        self.domain.borrow().num_closed_slots()
    }
    fn num_linked_slots(&self) -> u32 {
        self.domain.borrow().num_linked_slots()
    }

    fn link_slots(&self, n: u32) -> Result<(), String> {
        self.domain.borrow_mut().link_slots(n).unwrap();
        // TODO: add nodes to queue, close crossed, merge islands
        Ok(())
    }
    fn close_slots(&self, n: u32) -> Result<(), String> {
        self.domain.borrow_mut().close_slots(n).unwrap();
        // TODO add nodes to queue, unexternify bridge
        Ok(())
    }
}

impl Display for Bridge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.nodes.0.upgrade().unwrap(), self.nodes.1.upgrade().unwrap())
    }
}

impl HashiDomain for Domain {
    fn new() -> Self {
        [true, true, true]
    }

    fn num_slots(&self) -> u32 {
        (self.len() as u32) - 1
    }
    fn num_open_slots(&self) -> u32 {
        self.iter().fold(
            0,
            |acc, &elem| if elem { acc + 1 } else { acc },
        ) - 1
    }
    fn num_linked_slots(&self) -> u32 {
        self.iter().position(|&elem| elem).unwrap() as u32
    }
    fn num_closed_slots(&self) -> u32 {
        self.num_slots() - (self.iter().rposition(|&elem| elem).unwrap() as u32)
    }

    fn link_slots(&mut self, n: u32) -> Result<(), String> {
        let open_slots = self.num_open_slots();
        // println!("before: {:?}", self);
        // println!("n: {:?}", n);
        // println!("open_slots: {:?}", open_slots);
        if n == 0 {
            // debug(&format!(
            //     "attempted to link 0 slots ({} slots open)",
            //     open_slots
            // ));
            return Ok(());
        }
        if open_slots == 0 {
            return Err(format!("attempted to link {} slots in a full bridge", n));
        }
        if open_slots < n {
            return Err(format!(
                "attempted to link {} slots in bridge with {} open slots",
                n,
                open_slots
            ));
        }

        let curr_min = self.iter().position(|&elem| elem).unwrap();
        // debug(&format!("slice: [{}..{}]", curr_min, (curr_min + n as usize)));
        for elem in self[curr_min..(curr_min + n as usize)].iter_mut() {
            *elem = false;
        }
        // println!("after: {:?}", self);
        // println!("open slots after: {}", self.num_open_slots());
        Ok(())
    }
    fn close_slots(&mut self, n: u32) -> Result<(), String> {
        let open_slots = self.num_open_slots();
        if n == 0 {
            // debug(&format!(
            //     "attempted to close 0 slots ({} slots open)",
            //     open_slots
            // ));
            return Ok(());
        }
        if open_slots == 0 {
            return Err(format!("attempted to close {} slots in a full bridge", n));
        }
        if open_slots < n {
            return Err(format!(
                "attempted to close {} slots in bridge with {} open slots",
                n,
                open_slots
            ));
        }

        let curr_max = self.iter().rposition(|&elem| elem).unwrap();
        for elem in self[(curr_max - n as usize + 1)..(curr_max + 1)].iter_mut() {
            *elem = false;
        }
        Ok(())
    }
}

impl Island {
    fn from_node(init_node: NdRef) -> Self {
        let init_node = init_node.upgrade().unwrap();

        let mut nodes: HashSet<ID> = HashSet::new();
        nodes.insert(init_node.id);
        let ext_bridges: HashSet<ID> = culled_bridge_iter!(init_node)
            .map(|x| x.id)
            .collect();
        let id = init_node.id;

        Island {
            nodes: nodes,
            ext_bridges: ext_bridges,
            id: id,
        }
    }
}


/**
* The machines are gaining ground. Time to show them what we're really made of...
**/
// Write an action using println!("message...");
// To debug: print_err!("Debug message...");
fn main() {
    // the number of cells on the X axis
    let width = parse_line::<u32>().expect("parsing width failed");

    // the number of cells on the Y axis
    let height = parse_line::<u32>().expect("parsing height failed");

    let mut puzzle: Vec<String> = Vec::with_capacity(height as usize);
    for _ in 0..height {
        let line = read_line(); // width characters, each either a number or a '.'
        puzzle.push(line);
    }

    let mut graph = HashiGraph::new(&puzzle);

    graph.solve();

    graph.print_solution();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node() {
        let mut dummy_graph = HashiGraph {
            nodes: Vec::new(),
            bridges: Vec::new(),
            islands: Vec::new(),
            node_queue: Vec::new(),
        };
        // let dummy_node1a = Rc::new(Node::new(&mut dummy_graph, (5, 3), 3, 9));
        // let dummy_node1b = Rc::new(Node::new(&mut dummy_graph, (5, 3), 3, 9));
        // let dummy_node2a = Rc::new(Node::new(&mut dummy_graph, (8, 7), 2, 19));
        // let dummy_node2b = Rc::new(Node::new(&mut dummy_graph, (8, 7), 2, 19));
        //
        // let dummy_bridge1 = Rc::new(Bridge::new(
        //     &mut dummy_graph,
        //     (Rc::downgrade(&dummy_node1a), Rc::downgrade(&dummy_node1b))
        // ));
        // let dummy_bridge2 = Rc::new(Bridge::new(
        //     &mut dummy_graph,
        //     (Rc::downgrade(&dummy_node2a), Rc::downgrade(&dummy_node2b))
        // ));

        dummy_graph._add_node((5, 3), 2);
        dummy_graph._add_node((8, 7), 1);
        dummy_graph._add_node((5, 7), 3);
        let node0 = Rc::downgrade(&dummy_graph.nodes[0]);
        let node1 = Rc::downgrade(&dummy_graph.nodes[1]);
        let node2 = Rc::downgrade(&dummy_graph.nodes[2]);
        dummy_graph._add_bridge((node0.clone(), node2.clone()));
        dummy_graph.nodes[0].set_bridge(Rc::downgrade(&dummy_graph.bridges[0]), "dn");
        dummy_graph.nodes[2].set_bridge(Rc::downgrade(&dummy_graph.bridges[0]), "up");
        dummy_graph._add_bridge((node2.clone(), node1.clone()));
        dummy_graph.nodes[1].set_bridge(Rc::downgrade(&dummy_graph.bridges[1]), "lf");
        dummy_graph.nodes[2].set_bridge(Rc::downgrade(&dummy_graph.bridges[1]), "rt");
        let test_node = dummy_graph.nodes[2].clone();

        // let mut test_node = Node::new(&mut dummy_graph, (5, 7), 4, 16);
        // test_node.set_bridge(Rc::downgrade(&dummy_bridge1), "up");
        // test_node.set_bridge(Rc::downgrade(&dummy_bridge2), "rt");

        assert_eq!(test_node.num_open_slots(), 4);
        // assert_eq!(dummy_graph.nodes[2].num_open_slots(), 4);

        test_node.apply_quota_rules();
        assert_eq!(test_node.num_open_slots(), 2);
        assert_eq!(test_node.num_linked_slots(), 2);
        assert_eq!(test_node.num_closed_slots(), 0);
    }

    #[test]
    fn test_bridge() {
        unimplemented!()
    }

    #[test]
    fn domain_test() {
        assert_eq!(Domain::new(), [true, true, true]);
        assert_eq!(Domain::new().num_slots(), 2);
        hashidomain_tester::<Domain>();
    }

    fn hashidomain_tester<T: HashiDomain>() {
        let slots = T::new().num_slots();

        let dummy_domain = T::new();
        assert_eq!(dummy_domain.num_slots(), slots);
        assert_eq!(dummy_domain.num_open_slots(), slots);
        assert_eq!(dummy_domain.num_linked_slots(), 0);
        assert_eq!(dummy_domain.num_closed_slots(), 0);

        println!("linked start");
        let mut linked_domain = T::new();
        for i in 1..(slots + 1) {
            linked_domain.link_slots(1).unwrap();
            assert_eq!(linked_domain.num_open_slots(), slots - i);
            assert_eq!(linked_domain.num_linked_slots(), i);
            assert_eq!(linked_domain.num_closed_slots(), 0);
        }
        println!("multilinked start");
        for i in 1..(slots + 1) {
            let mut multilinked_domain = T::new();
            multilinked_domain.link_slots(i).unwrap();
            assert_eq!(multilinked_domain.num_open_slots(), slots - i);
            assert_eq!(multilinked_domain.num_linked_slots(), i);
            assert_eq!(multilinked_domain.num_closed_slots(), 0);
        }

        let mut closed_domain = T::new();
        for i in 1..(slots + 1) {
            closed_domain.close_slots(1).unwrap();
            assert_eq!(closed_domain.num_open_slots(), slots - i);
            assert_eq!(closed_domain.num_linked_slots(), 0);
            assert_eq!(closed_domain.num_closed_slots(), i);
        }
        for i in 1..(slots + 1) {
            let mut multiclosed_domain = T::new();
            multiclosed_domain.close_slots(i).unwrap();
            assert_eq!(multiclosed_domain.num_open_slots(), slots - i);
            assert_eq!(multiclosed_domain.num_linked_slots(), 0);
            assert_eq!(multiclosed_domain.num_closed_slots(), i);
        }

        let mut both_domain = T::new();
        both_domain.link_slots(1).unwrap();
        both_domain.close_slots(1).unwrap();
        assert_eq!(both_domain.num_open_slots(), slots - 2);
        assert_eq!(both_domain.num_linked_slots(), 1);
        assert_eq!(both_domain.num_closed_slots(), 1);
    }

    #[test]
    fn test_hashigraph() {
        let puzzle = INTERMEDIATE_1_PUZZLE.split("\n").collect::<Vec<_>>();
        let mut graph = HashiGraph::new(&puzzle);
        // TODO test that it initialized properly, e.g. all the proper nodes, bridges, & islands
        assert_eq!(graph.nodes.len(), 10);
        assert_eq!(graph.islands.len(), 10);
        assert_eq!(graph.node_queue.len(), 10);
        assert_eq!(graph.bridges.len(), 11);

        let node_first = graph.nodes[0].clone();
        let node_last = graph.nodes.iter().last().unwrap().clone();
        let queue_last = graph.node_queue.iter().last().unwrap().upgrade().unwrap();
        assert_eq!(node_first.quota, 4);
        assert_eq!(node_last.quota, 2);
        assert!(Rc::ptr_eq(&node_first, &queue_last));

        graph.solve();
        assert_eq!(graph.solution_iter().collect::<Vec<_>>().join("\n"), "0 0 2 0 2
2 0 3 0 2
3 0 4 0 2
2 0 2 2 1
2 2 4 2 2
4 0 4 2 2
0 0 0 3 2
0 3 1 3 1
1 1 1 3 2
2 2 2 3 2");
    }

    #[allow(dead_code)]
    const INTERMEDIATE_1_PUZZLE: &str = "\
4.544
.2...
..5.4
332..";

    #[allow(dead_code)]
    const MULTIPLE_SOLUTIONS_PUZZLE: &str = "\
3.3
...
3.3";

    #[allow(dead_code)]
    const ADVANCED_PUZZLE: &str = "\
3.4.6.2.
.1......
..2.5..2
1.......
..1.....
.3..52.3
.2.17..4
.4..51.2";

    #[allow(dead_code)]
    const CG_PUZZLE: &str = "\
22221
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
2222.";

    #[allow(dead_code)]
    const MULTIPLE_SOLUTIONS_2_PUZZLE: &str = "\
.12..
.2421
24442
1242.
..21.";

    #[allow(dead_code)]
    const EXPERT_PUZZLE: &str = "\
3..2.2..1....3........4
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
4...................4.2";
}

// TODO build move-decider, revertible decision tree to hold past moves, method of outputting
//        final configuration

/* TODO Move Decider:
 * heuristics:
 * - if count of reachable (not blocked, full, or absent) neighbors is fewer than `number`,
 *   add a link to each reachable neighbor
 */
