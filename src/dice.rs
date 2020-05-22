// NB the backend for the graphics version of azdice.
// it now uses the `libazdice` library.
use libazdice::parse::parse;

//Function to parse input and generate a VICTORY probability.
pub(crate) fn parse_input(input:&str,reps:usize) -> Option<(String, Vec<(i32, f64)>)> {
	let title = format!("{} ({} rolls of the dice-bag)", input, reps);
	let mut args = input.to_owned();

	let vs_count = args.matches("vs").count();
	if vs_count==1 {
		// Split once and do opposed distributions.
		let mut args_2 = args.split("vs").map(ToOwned::to_owned).collect::<Vec<String>>();
		if args_2.len() != 2 { return None; }

		let side2 = args_2.pop().expect("We have one!");
		let side1 = args_2.pop().expect("We have one!");

		// Make dicebags.
		let (bag1, bag2) = match (parse(side1), parse(side2)) {
			(Ok(b1), Ok(b2)) => (b1, b2),
			_ => return None,
		};

		// Get statistics for making the cumulative distribution.
		let minmax1 = bag1.get_range();
		let minmax2 = bag2.get_range();
		let min = minmax2[0] - minmax1[1];
		let max = minmax2[1] - minmax1[0];
		let mut rolls = Vec::with_capacity((max - min + 1) as usize);
		println!("bag 1: {:?}", bag1);
		println!("bag 2: {:?}", bag2);
		// Make distribution.
		for adv in (min)..((max+1)) {
			let mut victories = 0;
			for _ in 0..reps {
				if adv + bag1.roll().total() >= bag2.roll().total() { victories+= 1; }
			}
			rolls.push((adv as i32, victories as f64/ reps as f64 * 100.0));
		}

		write_azdice_csv(&title, &rolls);
		Some((title, rolls))

	} else if !args.contains("vs") {
		// Roll a single distribution. In this case we just have a probability distribution. Non cumulative.
		if let Some(dice_bag) = parse(args).ok() {
			let range = dice_bag.get_range();
			let total_reps = reps * (range[1] - range[0]) as usize;
			let dist = dice_bag
				.make_frequency_distribution(total_reps)
				.into_iter()
				.map(|(i,c)| (i as i32, c))
				.collect::<Vec<_>>();
			write_azdice_csv(&title, &dist);
			Some((title, dist))
		}else{
			None
		}
	} else {
		None
	}
}

/// A function to write
fn write_azdice_csv(title:&str, data: &Vec<(i32, f64)>) {
	use std::io::Write;

	let mut out_file = std::fs::File::create(title).unwrap();
	out_file.write("Skillpoint Advantage,Success(%)\n".as_bytes()).expect("Couldn't write title");

	//Iterate through skill point advantages. Writing the result to file
	for (x,y) in data.iter() {
		out_file.write(&format!("{},{:#.5}\n", x, y).as_bytes()).expect(&format!("could not write output."));
	};
}

//A function to parse input file into an internal azdice object.
pub fn parse_azdice_csv_cont(csv: std::fs::File) -> Option<Vec<(i32,f64)>> {
	use std::io::BufRead;

	//Initiate output and split input.
	let mut output_vec:Vec<(i32,f64)> = Vec::with_capacity(10000);
	let mut csv = std::io::BufReader::new(&csv);
	let mut temp_text = String::with_capacity(500);

	//Skip the first line of the input (it contains the column names)
	let mut count = csv.read_line(&mut temp_text).unwrap_or(0);

	//Cycle through the remaining lines, line by line.
	while count>0 {
		let mut temp_text = String::with_capacity(500);
		count = csv.read_line(&mut temp_text).unwrap_or(0);
		//Split into cells.
		let y = temp_text.split(',').collect::<Vec<&str>>();
		println!("{:?}",temp_text);

		//Operate
		if y.len()==2 {
			//if it has two columns, try to process them and add to vector.
			//if they can't be parsed, the data is corrupted and we shouldn't try.
			let adv:i32 = match y[0].trim().parse() {
				Ok(num) => {num},
				_		=> {
					println!("Could not parse line: \"{}\"",temp_text);
					return None
				},
			};
			let succ:f64 = match y[1].trim().parse::<f64>() {
				Ok(num) => {num},
				_		=> {
					println!("Could not parse line: \"{}\"",temp_text);
					return None
				},
			};
			output_vec.push((adv,succ));
		}else if temp_text.len()==0 {
		}else{
			//If not two columns then this is not the right type of file, and we should stop trying to work with it.
			println!("More (or less) than 2 entries per line found. This is not an AZDice file.");
			println!("Here is the offending passage: {:?}",temp_text);
			return None
		};
	}

	Some(output_vec)
}


//Help blurb to print if one of the args==["--help"].
pub(crate) const HELP:&str = "\nWelcome to AZDice.
This is a dice roll simulator which works out probability of success on opposed dice rolls.

It rolls opposed checks with a bonus difference, outputting a probability table. \
The table contains advantage in one column and probability of success in opposed roll in the second. \
So for A:(1d20+10) vs B:(1d20+5) the advantage of A over B is five.
The program prints an abbreviated table, and saves the output under a CSV table named \"Dice output d###.csv\".

Arguments:

--help        Print this message.

Useage on linux:

./azdice [dice] [repeats]

eg: ./azdice 20 10000

OR

./azdice [complex roll]

eg: ./azdice 2d20+5d12+d100 vs 3d100

TIP: If you want to do a constant vs a variable use d0 vs ??? (eg armor class vs attack use \"d1 vs 1d20\")

Useage on windows: My guess is as good as yours.
";
