//cargo rustc --bin azdice --release --target=x86_64-pc-windows-gnu -- -C linker=x86_64-w64-mingw32-gcc

extern crate rand;
use rand::Rng;
use std::env;

mod libdice;
const DEF_REPS:usize = 20_000;

fn main() {
	
	let mut args = env::args().collect::<Vec<String>>();
	args.remove(0);
	
	let mut help = false;

	for x in args.iter() {
		if x=="--help" {
			println!("{}",libdice::HELP);
			help = true;
		};
	};
	
	if !help {
		libdice::parse_input(&args.into_iter().collect::<String>(),DEF_REPS);
	};

}
