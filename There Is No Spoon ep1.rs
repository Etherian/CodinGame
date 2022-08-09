use std::io;
use std::fmt;

macro_rules! print_err {
    ($($arg:tt)*) => (
        {
            use std::io::Write;
            writeln!(&mut ::std::io::stderr(), $($arg)*).ok();
        }
    )
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Clone,Copy)]
struct Point {
    x: i32,
    y: i32
}

enum RCDir {
    Vert,
    Horz
}

// impl fmt::Display for (Point,Point,Point) {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{} {} {} {} {} {}", self.0.x, self.0.y, self.1.x, self.1.y, self.2.x, self.2.y)
//     }
// }

/**
 * Don't let the machines win. You are humanity's last hope...
 **/
fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let width = parse_input!(input_line, i32); // the number of cells on the X axis

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let height = parse_input!(input_line, i32); // the number of cells on the Y axis

    let mut matrix: Vec<Vec<char>> = Vec::new();
    for i in 0..height as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let line = input_line.trim_right().to_string(); // width characters, each either 0 or .
        matrix.push( line.chars().collect() );
    }

    // Write an action using println!("message...");
    // To debug: print_err!("Debug message...");


    let mut output_triples: Vec<(Point,Point,Point)> = Vec::new();
    for (y, row) in matrix.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if *cell == '0' {
                let node_coord = Point{ x: x as i32, y: y as i32 };
                output_triples.push( (node_coord,
                                      ray_cast(node_coord, &matrix, RCDir::Horz),
                                      ray_cast(node_coord, &matrix, RCDir::Vert)) );
            }
        }
    }

    // Three coordinates: a node, its right neighbor, its bottom neighbor
    for triple in output_triples {
        println!("{} {} {} {} {} {}", triple.0.x, triple.0.y, triple.1.x, triple.1.y, triple.2.x, triple.2.y);
    }
}

fn ray_cast(curr_point: Point, matrix: &Vec<Vec<char>>, direction: RCDir) -> Point {
    match direction {
        RCDir::Horz => {
            let x = matrix.get(curr_point.y as usize)
                          .expect(&format!("ray_cast horz: row {} does not exist", curr_point.y))
                          .iter()
                          .enumerate()
                          .skip(curr_point.x as usize + 1)
                          .find(|cell| (*cell).1 == &'0');
            match x {
                Some( (x, _) ) => Point{ x: x as i32, y: curr_point.y },
                None => Point{ x: -1, y: -1 }
            }
        },
        RCDir::Vert => {
            let y = matrix.iter()
                          .enumerate()
                          .skip(curr_point.y as usize + 1)
                          .find(|row| {
                              (*row).1.get(curr_point.x as usize)
                                      .expect(&format!("ray_cast vert: {} is out of bounds", curr_point.x)) == &'0'
                           });
            match y {
                Some( (y, _) ) => Point{ x: curr_point.x, y: y as i32 },
                None => Point{ x: -1, y: -1 }
            }
        }
    }
}
