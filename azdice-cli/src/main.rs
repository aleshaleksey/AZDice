//cargo rustc --bin azdice --release --target=x86_64-pc-windows-gnu -- -C linker=x86_64-w64-mingw32-gcc
extern crate rand;
extern crate libazdice;
use std::env;

mod dice;
const DEF_REPS:usize = 20_000;

fn main() {

	let mut args = env::args().collect::<Vec<String>>();
	args.remove(0);

	let mut help = false;

	for x in args.iter() {
		if x=="--help" {
			println!("{}",dice::HELP);
			help = true;
		};
	};

	let input = args.into_iter().collect::<String>();

	if !help {
		let (title, rolls) = if let Some(res) = dice::parse_input(&input, DEF_REPS) {
			res
		} else {
			println!("Invalid dice string ({}).", &input);
			return;
		};

		println!("{}", title);
		for (i, x) in rolls {
			println!(" {}\t\t|\t{}", i, x);
		}

	};

}
