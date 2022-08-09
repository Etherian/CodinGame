use std::io;
use std::iter::repeat;

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

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 * ---
 * Hint: You can use the debug stream to print initialTX and initialTY, if Thor seems not follow your orders.
 **/
fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();
    let light_x = parse_input!(inputs[0], i32); // the X position of the light of power
    let light_y = parse_input!(inputs[1], i32); // the Y position of the light of power
    let initial_tx = parse_input!(inputs[2], i32); // Thor's starting X position
    let initial_ty = parse_input!(inputs[3], i32); // Thor's starting Y position

    let dx = light_x - initial_tx;
    let dy = light_y - initial_ty;
    let mut horz = repeat(["W","","E"][(1 + dx.signum()) as usize].to_string()).take(dx.abs() as usize);
    let mut vert = repeat(["N","","S"][(1 + dy.signum()) as usize].to_string()).take(dy.abs() as usize);
    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let remaining_turns = parse_input!(input_line, i32); // The remaining amount of turns Thor can move. Do not remove this line.

        // Write an action using println!("message...");
        // To debug: print_err!("Debug message...");

        let vdir =   if let Some(dir) = vert.next() {dir} else {"".to_string()};
        let hdir =   if let Some(dir) = horz.next() {dir} else {"".to_string()};
        // A single line providing the move to be made: N NE E SE S SW W or NW
        println!("{}{}", vdir, hdir);
    }
}
