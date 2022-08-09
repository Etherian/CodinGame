This question is motivated by [this CodinGame puzzle](https://www.codingame.com/training/hard/skynet-revolution-episode-2).

I am implementing a basic pathfinding algorithm using Dijkstra's method. It uses a `boundary` HashMap and a `finished` HashMap to hold pathfinding-related node info. In a particular loop, I find the highest-valued node in `boundary`, remove the node, add the node to `finished`, and add/update the node's neighbors' info in `boundary`.

Attempting to mutate `boundary` while looping over it is making Rust's borrow checker queasy, but the logic of the loop seems sound to me. How do I rewrite it so that the compiler shares my confidence? (Or fix the errors I'm missing, if that's the issue.)

### Code:

[On Rust Playground here](https://play.rust-lang.org/?gist=b71e12ebfd5f2c2ef40b9efbc83803ee&version=stable&backtrace=0)

    use std::io;
    use std::collections::{HashSet, HashMap};
    use std::cmp::Ordering;
    use std::cell::RefCell;

    struct NodeInfo {
        nbrs: HashSet<i32>,
        gwlinks: i32,
    }

    #[derive(PartialEq,PartialOrd)]
    struct PFInfo {
        avg: f32,
        cum: i32,
        dist: i32,
        prev: Option<i32>,
    }

    impl Eq for PFInfo {}

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

    // Find the path that passes the most gateway links per distance traveled,
    // starting at a given node. This is meant to simulate the behavior of an
    // "agent" which traverses the graph in the puzzle mentioned above.
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

        // Keep grabbing the key corresponding the highest value until `boundary` is
        // empty
        while let Some( (currid, _) ) = boundary.iter().max_by_key(|x| x.1) {

            // Move the node from `boundary` to `finished`
            let val = boundary.remove(&currid).unwrap();
            finished.insert(currid.clone(), val);

            // Add or update all adjacent nodes that are not in `finished`
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

        let mut path = Vec::new();
        let mut currid = finished.iter().max_by_key(|x| x.1).unwrap().0.clone();
        path.push(currid.clone());
        while let Some(previd) = finished.get(&currid).unwrap().prev {
            path.push(previd.clone());
            currid = previd.clone();
        }
        path.reverse();

        path
    }



    macro_rules! parse_input {
        ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
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

        let header = inputs.next().unwrap().split_whitespace().collect::<Vec<_>>();
        let n = parse_input!(header[0], i32); // the total number of nodes in the level, including the gateways
        let l = parse_input!(header[1], i32); // the number of links
        let e = parse_input!(header[2], i32); // the number of exit gateways

        let mut graph = Graph::with_capacity(n as usize);
        for node in 0..n {
            graph.insert(node, RefCell::new(NodeInfo{ nbrs: HashSet::new(), gwlinks: 0 }));
        }
        let graph = graph;

        for _ in 0..l as usize {
            let link = inputs.next().unwrap();
            let nodes = link.split(" ").collect::<Vec<_>>();
            let n1 = parse_input!(nodes[0], i32); // N1 and N2 defines a link between these nodes
            let n2 = parse_input!(nodes[1], i32);

            graph.get(&n1).unwrap().borrow_mut().nbrs.insert(n2);
            graph.get(&n2).unwrap().borrow_mut().nbrs.insert(n1);
        }

        let mut gateways = HashSet::new();
        for _ in 0..e as usize {
            let ei = parse_input!(inputs.next().unwrap(), i32); // the index of a gateway node
            gateways.insert(ei);
        }
        let gateways = gateways;

        for gwid in &gateways {
            for gwnbr in &graph.get(gwid).unwrap().borrow().nbrs {
                (&graph).get(&gwnbr).unwrap().borrow_mut().gwlinks += 1;
            }
        }

        assert_eq!(generate_path(&0, &graph), vec![0, 3]);
    }

### Errors:

<!-- language: none -->

    rustc 1.18.0 (03fc9d622 2017-06-06)
    error[E0502]: cannot borrow `boundary` as mutable because it is also borrowed as immutable
      --> <anon>:53:19
       |
    50 |     while let Some( (currid, _) ) = boundary.iter().max_by_key(|x| x.1) {
       |                                     -------- immutable borrow occurs here
    ...
    53 |         let val = boundary.remove(&currid).unwrap();
       |                   ^^^^^^^^ mutable borrow occurs here
    ...
    76 |     }
       |     - immutable borrow ends here

    error[E0502]: cannot borrow `boundary` as mutable because it is also borrowed as immutable
      --> <anon>:66:13
       |
    50 |     while let Some( (currid, _) ) = boundary.iter().max_by_key(|x| x.1) {
       |                                     -------- immutable borrow occurs here
    ...
    66 |             boundary.insert(
       |             ^^^^^^^^ mutable borrow occurs here
    ...
    76 |     }
       |     - immutable borrow ends here

    error: aborting due to 2 previous errors
