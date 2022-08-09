use std::io;
use std::io::BufRead;
use std::collections::{HashSet, HashMap};
use std::cmp::Ordering;
use std::cell::RefCell;

// macro_rules! print_err {
//     ($($arg:tt)*) => (
//         {
//             use std::io::Write;
//             writeln!(&mut ::std::io::stderr(), $($arg)*).ok();
//         }
//     )
// }

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

macro_rules! parse_line {
    ($t:ty) => {parse_line!(::std::io::stdin(); &t)};

    ($input:expr; $t:ty) => {{
        let mut input_line = String::new();
        $input.read_line(&mut input_line).unwrap();
        input_line.trim().parse::<$t>().unwrap()
    }};

    ($($t:ty),+) => {parse_line!(::std::io::stdin(); $($t),+)};

    ($input:expr; $($t:ty),+) => {{
        let mut input_line = String::new();
        $input.read_line(&mut input_line).unwrap();
        let mut frags = input_line.trim().split_whitespace();
        (
            $(
                frags.next()
                    .expect(&format!("line \"{}\" does not contain as many elements as were asked for", input_line))
                    .parse::<$t>().unwrap()
            ),+
        )
    }};
}

// macro_rules! input {
//     () => {{
//         let mut input_line = String::new();
//         io::stdin().read_line(&mut input_line).unwrap();
//         input_line
//     }};
//
//     ( &( &t:ty ),+ ) => {{
//         let mut input_line = String::new();
//         io::stdin().read_line(&mut input_line).unwrap();
//         let mut inputs = input_line.split(" ").collect::<Vec<_>>();
//
//         let mut i = 0usize;
//         ( $( parse_input!(inputs.pop().unwrap(),$t), )+ )
//     }};
// }

 struct NodeInfo {
     nbrs: HashSet<i32>,
     gwlinks: i32,
 }

#[derive(PartialEq,PartialOrd,Debug)]
 struct PFInfo {
     avg: f32,
     cum: i32,
     dist: i32,
     prev: Option<i32>,
 }

impl Eq for PFInfo {
    //  fn eq(&self, other: &PFInfo) -> bool {
    //     match self.partial_eq(other) {
    //         Some(eq) => eq,
    //         None => false
    //     }
    // }
}

impl Ord for PFInfo {
     fn cmp(&self, other: &PFInfo) -> Ordering {
        match self.partial_cmp(other) {
            Some(ord) => ord,
            None => Ordering::Equal
        }
    }
}

 type Graph = HashMap<i32, RefCell<NodeInfo>>;
 type PFGraph = HashMap<i32, PFInfo>;

fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();

    let n = parse_input!(inputs[0], i32); // the total number of nodes in the level, including the gateways
    let l = parse_input!(inputs[1], i32); // the number of links
    let e = parse_input!(inputs[2], i32); // the number of exit gateways
    // let (n, l, e) = input!(i32, i32, i32);

    let mut graph = Graph::with_capacity(n as usize);
    for node in 0..n {
        graph.insert(node, RefCell::new(NodeInfo{ nbrs: HashSet::new(), gwlinks: 0 }));
    }
    let graph = graph; // remove mutability

    for _ in 0..l as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let n1 = parse_input!(inputs[0], i32); // N1 and N2 defines a link between these nodes
        let n2 = parse_input!(inputs[1], i32);

        graph.get(&n1).unwrap().borrow_mut().nbrs.insert(n2);
        graph.get(&n2).unwrap().borrow_mut().nbrs.insert(n1);
    }

    let mut gateways = HashSet::new();
    for _ in 0..e as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let ei = parse_input!(input_line, i32); // the index of a gateway node
        gateways.insert(ei);
    }
    let gateways = gateways; // remove mutability

    for gwid in &gateways {
        for gwnbr in &graph.get(gwid).unwrap().borrow().nbrs {
            (&graph).get(&gwnbr).unwrap().borrow_mut().gwlinks += 1;
        }
    }


    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let si = parse_input!(input_line, i32); // The index of the node on which the Skynet agent is positioned this turn

        let target = if graph.get(&si).unwrap()
                      .borrow()
                      .nbrs.iter()
                      .any(|nbr| gateways.contains(nbr)) {
            si
        } else {
            let path = generate_path(&si, &graph);
            choose_target( &path, &graph ).clone()
        };

        let gw = graph.get(&target).unwrap()
                      .borrow()
                      .nbrs.iter()
                      .filter(|x| gateways.contains(x))
                      .next().unwrap()
                      .clone();

        // Write an action using println!("message...");
        // To debug: print_err!("Debug message...");
        sever(&gw, &target, &graph);
    }
}

fn generate_path(si: &i32, graph: &Graph) -> Vec<i32> {
    let n = graph.len();
    let mut boundary = PFGraph::with_capacity(n);
    let mut finished = PFGraph::with_capacity(n);

    boundary.insert( si.clone(),
                     PFInfo {
                         avg: 0.,
                         cum: graph.get(&si).unwrap().borrow().gwlinks,
                         dist: 0,
                         prev: None } );

    while !boundary.is_empty() {
        let currid = boundary.iter().max_by_key(|x| x.1).unwrap().0.clone();
        let val = boundary.remove(&currid).unwrap();
        finished.insert(currid.clone(), val);

        for nbrid in graph.get(&currid).unwrap()
                          .borrow()
                          .nbrs.iter()
                          .filter(|x| !finished.contains_key(x)) {
            let currval = finished.get(&currid).unwrap();
            let prev = Some(currid.clone());
            let dist = currval.dist + 1;
            let cum = currval.cum + graph.get(nbrid).unwrap().borrow().gwlinks;
            let avg = cum as f32 / dist as f32;
            boundary.insert(
                nbrid.clone(),
                PFInfo {
                    avg: avg,
                    cum: cum,
                    dist: dist,
                    prev: prev,
                }
            );
        }
    }
    // println!("PF data populated");

    // while let Some( val ) = boundary.iter().max_by_key(|x| x.1).and_then(|item| boundary.remove(item.0)) {
    //     let currid = &1;
    //     finished.insert(currid.clone(), val);
    //
    //     for nbrid in graph.get(&currid).unwrap()
    //                       .borrow()
    //                       .nbrs.iter()
    //                       .filter(|x| !finished.contains_key(x)) {
    //         let currval = finished.get(currid).unwrap();
    //         let prev = Some(currid.clone());
    //         let dist = currval.dist + 1;
    //         let cum = currval.cum + graph.get(nbrid).unwrap().borrow().gwlinks;
    //         let avg = cum as f32 / dist as f32;
    //         boundary.insert(
    //             nbrid.clone(),
    //             PFInfo {
    //                 avg: avg,
    //                 cum: cum,
    //                 dist: dist,
    //                 prev: prev,
    //             }
    //         );
    //     }
    // }

    // while let Some( (currid, _) ) = boundary.iter().max_by_key(|x| x.1) {
    //     let val = boundary.remove(currid).unwrap();
    //     finished.insert(currid.clone(), val);
    //
    //     for nbrid in graph.get(&currid).unwrap()
    //                       .borrow()
    //                       .nbrs.iter()
    //                       .filter(|x| !finished.contains_key(x)) {
    //         let currval = finished.get(currid).unwrap();
    //         let prev = Some(currid.clone());
    //         let dist = currval.dist + 1;
    //         let cum = currval.cum + graph.get(nbrid).unwrap().borrow().gwlinks;
    //         let avg = cum as f32 / dist as f32;
    //         boundary.insert(
    //             nbrid.clone(),
    //             PFInfo {
    //                 avg: avg,
    //                 cum: cum,
    //                 dist: dist,
    //                 prev: prev,
    //             }
    //         );
    //     }
    // }

    // println!("backtracking beginning");
    let mut path = Vec::new();
    let mut currid = finished.iter().max_by_key(|x| x.1).unwrap().0.clone();
    path.push(currid.clone());
    let mut i = 0;
    while let Some(previd) = finished.get(&currid).unwrap().prev {
        path.push(previd.clone());
        currid = previd.clone();
    }
    path.reverse();

    path
}

fn choose_target<'a>(path: &'a [i32], graph: &Graph) -> &'a i32 {
    // find the earliest node on the path with the highest number of gw links
    // `rev` is needed because `max_by_key` returns the LAST found maximum and we need the
    // FIRST found
    path.iter().rev().max_by_key(|x| graph.get(x).unwrap().borrow().gwlinks).unwrap()
}

fn sever(gw: &i32, target: &i32, graph: &Graph) {
    graph.get(&gw).unwrap().borrow_mut().nbrs.remove(target);
    graph.get(&target).unwrap().borrow_mut().nbrs.remove(gw);
    graph.get(&target).unwrap().borrow_mut().gwlinks -= 1;

    // Example: indices of the nodes you wish to sever the link between
    println!("{} {}", gw, target);
}

#[test]
fn test_generate_path() {
    let mut inputs = "8 13 2
6 2
7 3
6 3
5 3
3 4
7 1
2 0
0 1
0 3
1 3
2 3
7 4
6 5
4
5".lines();

    // let header = inputs.next().unwrap().split_whitespace().collect::<Vec<_>>();
    // let n = parse_input!(header[0], i32); // the total number of nodes in the level, including the gateways
    // let l = parse_input!(header[1], i32); // the number of links
    // let e = parse_input!(header[2], i32); // the number of exit gateways
    let header = inputs.next().unwrap();
    let (n, l, e) = parse_line!(header.as_bytes(); i32, i32, i32);

    // println!( "(n,l,e):{:?}", (n,l,e) );

    let mut graph = Graph::with_capacity(n as usize);
    for node in 0..n {
        graph.insert(node, RefCell::new(NodeInfo{ nbrs: HashSet::new(), gwlinks: 0 }));
    }
    let graph = graph;
    // println!("nodes finished");

    for _ in 0..l as usize {
        // let link = inputs.next().unwrap();
        // let nodes = link.split(" ").collect::<Vec<_>>();
        // let n1 = parse_input!(nodes[0], i32);
        // let n2 = parse_input!(nodes[1], i32);

        // N1 and N2 defines a link between these nodes
        let (n1, n2) = parse_line!(inputs.next().unwrap(); i32, i32);

        graph.get(&n1).unwrap().borrow_mut().nbrs.insert(n2);
        graph.get(&n2).unwrap().borrow_mut().nbrs.insert(n1);
    }
    // println!("links finished");

    let mut gateways = HashSet::new();
    for _ in 0..e as usize {
        let ei = parse_input!(inputs.next().unwrap(), i32); // the index of a gateway node
        gateways.insert(ei);
    }
    let gateways = gateways;
    // println!("gateways finished");

    for gwid in &gateways {
        for gwnbr in &graph.get(gwid).unwrap().borrow().nbrs {
            (&graph).get(&gwnbr).unwrap().borrow_mut().gwlinks += 1;
        }
    }
    // println!("gw links counted");

    assert_eq!(generate_path(&0, &graph), vec![0, 3]);
}
