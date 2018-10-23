//NB the backend for the graphics version of azdice.
//Basically the other version.

extern crate rand;
use rand::Rng;
use std::fs;
use std::io::Write;
use std::num;

//Function to parse input and generate 
pub fn parse_input(input:&str,reps:usize)->Option<(String,Vec<(i32,f64)>)> {
	
	let mut reps = reps;
	let mut title:String = String::with_capacity(200);
	let mut data:Vec<(i32,f64)> = Vec::with_capacity(500);
	
	let mut dice:i32 = 100;
	let mut args = input.to_owned();	
	let mut advanced = if input.contains("vs") | input.contains('d') {true}else{false};
	
	if !advanced {
	
		parse_args(args, &mut reps, &mut dice, 100);
		//create output.
		title = format!("Dice output d{}.csv",dice);
		let mut out_file = fs::File::create(&title).unwrap();
		data = Vec::with_capacity(dice as usize*2);
		out_file.write("Skillpoint Advantage,Success(%)\n".as_bytes()).expect("Couldn't write title");
		
		//Iterate through skill point advantages. Writing the result to file
		println!("Rolling a 1d{} vs 1d{} {}-times for each \"advantage\".\n",dice,dice,reps);
		for advantage in -dice..dice+1 {
			let out:(i32,f64) = generate_probability(advantage,reps,dice);
			out_file.write(&format!("{},{:#.5}\n",out.0,out.1).as_bytes()).expect(&format!("could not write output at {} SP",advantage));
			data.push(out);
		};
		return Some((format!("1d{0} vs 1d{0}",dice),data))
	}else{
		
		//get the dice types and numbers for each side.
		let argl = args.len();
		let mut roll_description = String::new();
		let dice:[Vec<(i32,i32)>;2] = match advanced_parser_outer(args) {
			Some(arg_vec) => {
				if arg_vec[0]==arg_vec[1] {
					match inner_parser(arg_vec[0].clone()) {
						Some(x) => {
							roll_description = format!("{0} vs {0}",arg_vec[0]);
							[x.clone(),x.clone()]
						},
						None	=> {
							println!("Incorrectly formatted arguments. Aborting");
							return None;
						},
					}
				}else{
					let mut dice_temp = [Vec::new(),Vec::new()];
					match inner_parser(arg_vec[0].clone()) {
						Some(x) => {dice_temp[0] = x;},
						None	=> {
							println!("Incorrectly formatted arguments. Aborting");
							return None;
						},
					};
					match inner_parser(arg_vec[1].clone()) {
						Some(x) => {dice_temp[1] = x;},
						None	=> {
							println!("Incorrectly formatted arguments. Aborting");
							return None;
						},
					};
					roll_description = format!("{} vs {}",arg_vec[0],arg_vec[1]);
					dice_temp
					
				}
			},
			_ => {
				println!("Incorrectly formatted arguments. Aborting");
				return None;
			},
		};
		
		let minmax = calculate_minmax(&dice);
		
		title = format!("Dice output {}.csv",roll_description);
		data = Vec::with_capacity(minmax as usize*2);
		let mut out_file = fs::File::create(&title).unwrap();
		out_file.write("Skillpoint Advantage,Success(%)\n".as_bytes()).expect("Couldn't write title");
		
		//Iterate through skill point advantages. Writing the result to file
		println!("Rolling {} {}-times for each \"advantage\".\n",roll_description,reps);
		for advantage in -(minmax+1)..minmax+1 {
			let out:(i32,f64) = generate_complex_probability(advantage,reps,&dice,minmax);
			out_file.write(&format!("{},{:#.5}\n",out.0,out.1).as_bytes()).expect(&format!("could not write output at {} SP",advantage));
			data.push(out);
		};
		return Some((roll_description,data))
	};
}

//Function to do a basic parsing of arguments (arg[1]=dice sides, arg[2]=no. of rounds).
fn parse_args(args:String,x:&mut usize,dice:&mut i32,default:i32) {	
	//get dice number (default 100)
	match args.parse() {
		Ok(num) => {*dice = num;},
		_		=> {*dice = default;},
	};
}

//Function to generate a probability of winning at a given advantage, and pass output out.
fn generate_probability(advantage:i32,reps:usize,dice:i32)->(i32,f64) {
	let mut vics = 0;
	let min = if dice==0 {0}else{1};
	for _ in 0..reps {
		let att = rand::thread_rng().gen_range(min,dice+1)+advantage;
		let def = rand::thread_rng().gen_range(min,dice+1);
		
		if att>=def {vics+=1;};
		
	}
	
	let out = (advantage,vics as f64/reps as f64*100.0);

	//Interval for printed output.
	let interval = if dice>20 {dice/10}else{1};
	if advantage%(interval)==0 {
		println!("Advantage={}. Success={:#.4}%.",out.0,out.1);
	};
	
	out
}

//Function to generate a probability of winning at a given advantage, and pass output out.
fn generate_complex_probability_marker(){}
fn generate_complex_probability(advantage:i32,reps:usize,dice:&[Vec<(i32,i32)>;2],minmax:i32)->(i32,f64) {
	let mut vics = 0;
			
	for _ in 0..reps {
		
		let mut att = advantage;
		let mut def = 0;
		
		//"Attack" value.
		for x in dice[0].iter() {
			let min = if x.1==0 {0}else{1};
			for _ in 0..x.0 {att+= rand::thread_rng().gen_range(min,x.1+1)};
		};
		
		//"Defence" value.
		for x in dice[1].iter() {
			let min = if x.1==0 {0}else{1};
			for _ in 0..x.0 {def+= rand::thread_rng().gen_range(min,x.1+1);};
		};	
		
		if att>=def {vics+=1;};
	}
	
	let out = (advantage,vics as f64/reps as f64*100.0);

	//Interval for printed output.
	let interval = if minmax>20 {minmax/10}else{1};
	if advantage%(interval)==0 {
		println!("Advantage={}. Success={:#.4}%.",advantage,out.1);
	};
	
	out
}

//An advanced parser that can parse things like "4d6 + 2d12 vs 10d100",
//This function removes file name, checks for badness (characters other than nums, d and vs),
//removes white spaces and returns a vector of strings vec!["4d6+2d12","10d100"],
//which are then passed to the inner parsers.
//It is assumed that the command name is already deleted.
fn advanced_parser_outer(mut args:String)->Option<Vec<String>> {
	
	//println!("args: {:?}",args);

	args = args.chars().filter(|x| !x.is_whitespace()).collect::<String>();
	//println!("args: {:?}",args);
	
	
	let mut splits:Vec<String> = args.split("vs").map(|x| x.to_owned()).collect::<Vec<String>>();
	//println!("splits: {:?}",splits);
	
	//Crash out if multiple vs detected.
	//Or if the splits contain illegal characters.
	if splits.len()>2 {
		println!("Multiple opposition found (not allowed yet). Please input rolls in the format of: [rolls] vs [rolls]... Eg 1d20 vs 3d6.\n");
		return None
	}else{
		for string in splits.iter() {
			for x in string.chars(){
				if !x.is_numeric() & (x!='d') & (x!='+') { //This can be functionalised.
					println!("Input comparison contains charactes other than numbers, \"d\" or \"+\". Cannot proceed.");
					return None
				};
			};
		}
	};
	
	if splits.len()==1 {
		let s0 = splits[0].clone();
		splits.push(s0);
	}
	
	//println!("splits = {:?}",splits);
	Some(splits)
}

//Parses a dice command to dice.
//eg "5d10+6d12"-> vec![(5,10),(6,12)]
//NB, it is assumed that the outer parser has dealt with all abberants.
fn inner_parser(comm:String)->Option<Vec<(i32,i32)>> {
	
	//split "5d10+6d12" into vec!["5d10","6d12]
	let command = comm.split('+').map(|x| x.to_owned()).collect::<Vec<String>>();
	
	if command.len()==0 {return None};
	let mut output = Vec::new();
	
	for x in command.into_iter() {
		
		//println!("x = {}",x);
		//Split "5d20" into vec![(5,20)]
		let mut n_d:Vec<Result<i32,num::ParseIntError>> = x.split('d').map(|y| y.parse::<i32>()).collect();
		//println!("n_d = {:?}",n_d);
		for (i,x) in n_d.iter_mut().enumerate() {
			if x.is_err() & (i>0) {
				println!("Invalid dice detected. In inner parser");
				//println!("Comm = {}",comm);
				//println!("Command = {:?}",x);
				return None
			} else if x.is_err() & (i==0) {
				*x = Ok(1);
			};
		};
		
		let n_d:Vec<i32> = n_d.into_iter().map(|x| x.unwrap()).collect();
		
		if x.chars().nth(0)==Some('d') {
		//Deal with the case of "d20" without a number prefix.
			if n_d.len()!=2 {
				println!("Something went very wrong with \"{}\".",x);
				return None
			}else{
				output.push((n_d[0],n_d[1]));
			};
		}else{
		//Deal with the case of "5d20" or "1d20" with a number at the front.
			if n_d.len()!=2 {
				println!("Something went very wrong with \"{}\".",x);
				return None
			}else{
				output.push((n_d[0],n_d[1]));
			};
		};
	}
	
	Some(output)
}

fn calculate_minmax_marker(){}
fn calculate_minmax(dice:&[Vec<(i32,i32)>;2])->i32 {
	let min_0 = dice[0].iter().fold(0,|acc,x| acc+x.0);
	let min_1 = dice[1].iter().fold(0,|acc,x| acc+x.0);
	let max_0 = dice[0].iter().fold(0,|acc,x| acc+x.0*x.1);
	let max_1 = dice[1].iter().fold(0,|acc,x| acc+x.0*x.1);
	
	let a = max_1-min_0;
	let b = max_0-min_1;
	if a>b {a}else{b}
}

//Help blurb to print if one of the args==["--help"].
pub const HELP:&str = "\nWelcome to AZDice.
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
