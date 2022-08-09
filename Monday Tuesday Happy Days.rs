use std::io;
use std::slice::Iter;
use std::cmp::Ordering;

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
 **/
fn main() {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let leap_year = parse_input!(input_line, i32);

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();

    let source_day_of_week = inputs[0].trim().to_string();
    let source_month = inputs[1].trim().to_string();
    let source_day_of_month = parse_input!(inputs[2], i32);

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();

    let target_month = inputs[0].trim().to_string();
    let target_day_of_month = parse_input!(inputs[1], i32);

    // Write an action using println!("message...");
    // To debug: print_err!("Debug message...");

    print_err!("leap: {}", leap_year);
    print_err!("src: {} {} {}", source_day_of_week, source_month, source_day_of_month);
    print_err!("target: {} {}", target_month, target_day_of_month);

    struct Month {
        name: &'static str,
        length: i32,
    }

    #[derive(Debug)]
    struct DateInfo {
        month_index: usize,
        month_name: &'static str,
        month_length: i32,
        day_of_month: i32,
    }

    enum Direction {
        SrcEarly,
        SrcLate
    }

    let norm_months = [ ("Jan", 31),
                        ("Feb", 28),
                        ("Mar", 31),
                        ("Apr", 30),
                        ("May", 31),
                        ("Jun", 30),
                        ("Jul", 31),
                        ("Aug", 31),
                        ("Sep", 30),
                        ("Oct", 31),
                        ("Nov", 30),
                        ("Dec", 31), ].iter()
                                      .map( |&tup| Month {name: tup.0, length: tup.1} )
                                      .collect::<Vec<Month>>();

    let leap_months = [ ("Jan", 31),
                        ("Feb", 29),
                        ("Mar", 31),
                        ("Apr", 30),
                        ("May", 31),
                        ("Jun", 30),
                        ("Jul", 31),
                        ("Aug", 31),
                        ("Sep", 30),
                        ("Oct", 31),
                        ("Nov", 30),
                        ("Dec", 31), ].iter()
                                      .map( |&tup| Month {name: tup.0, length: tup.1} )
                                      .collect::<Vec<Month>>();

    let days_of_the_week = [ "Monday",
                             "Tuesday",
                             "Wednesday",
                             "Thursday",
                             "Friday",
                             "Saturday",
                             "Sunday", ];

    let months = if leap_year == 1 {leap_months} else {norm_months};

    let src_m_index = months.iter()
                            .position( |month| month.name == source_month )
                            .expect( &format!("couldn't find source_month '{}' in months", source_month) );
    let tar_m_index = months.iter()
                            .position( |month| month.name == target_month )
                            .expect( &format!("couldn't find target_month '{}' in months", target_month) );

    let src_date_info = DateInfo{ month_index: src_m_index,
                                  month_name: months[src_m_index].name,
                                  month_length: months[src_m_index].length,
                                  day_of_month: source_day_of_month };
    print_err!("{:?}", src_date_info);

    let tar_date_info = DateInfo{ month_index: tar_m_index,
                                  month_name: months[tar_m_index].name,
                                  month_length: months[tar_m_index].length,
                                  day_of_month: target_day_of_month };
    print_err!("{:?}", tar_date_info);

    let target_day_of_week =
        match (tar_m_index, target_day_of_month).cmp(&(src_m_index, source_day_of_month)) {
            Ordering::Greater => {
                let days_of_intervening_months = months[src_date_info.month_index..tar_date_info.month_index]
                                                    .iter()
                                                    .map(|month| month.length)
                                                    .sum::<i32>();
                print_err!("{:?}", days_of_intervening_months);
                let intervening_days = days_of_intervening_months
                                       - src_date_info.day_of_month
                                       + tar_date_info.day_of_month;
                print_err!("{:?}", intervening_days);
                let intervening_days_from_start_of_week = intervening_days
                                                          + days_of_the_week.iter()
                                                                            .position(|&day| day == source_day_of_week)
                                                                            .unwrap() as i32;
                print_err!("{:?}", intervening_days_from_start_of_week);
                days_of_the_week[(intervening_days_from_start_of_week % 7) as usize].to_string()
            },
            Ordering::Less => {
                let days_of_intervening_months = months[tar_date_info.month_index..src_date_info.month_index]
                                                    .iter()
                                                    .map(|month| month.length)
                                                    .sum::<i32>();
                print_err!("{:?}", days_of_intervening_months);
                let intervening_days = days_of_intervening_months
                                       - tar_date_info.day_of_month
                                       + src_date_info.day_of_month;
                print_err!("{:?}", intervening_days);
                let intervening_days_from_start_of_week = intervening_days
                                                          + (7 - days_of_the_week.iter()
                                                                                 .position(|&day| day == source_day_of_week)
                                                                                 .unwrap()
                                                            ) as i32;
                print_err!("{:?}", intervening_days_from_start_of_week);
                days_of_the_week[(6 - intervening_days_from_start_of_week % 7) as usize].to_string()
            },
            Ordering::Equal => {
                source_day_of_week
            }
        };

    println!("{}", target_day_of_week);
}
