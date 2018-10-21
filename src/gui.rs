//(u8,Option<conrod::widget::Id>)
#![allow(unused_imports)]
extern crate conrod;
extern crate glium;
extern crate winit;
extern crate rand;
extern crate clipboard;

use azgraph;

use conrod::UiCell;
use conrod::{color, widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
use conrod::widget::button::Flat;
use conrod::widget::text_box::Event::Update;
use conrod::widget::Id;

use rand::Rng;
use std::env;
use std::fs::{File,OpenOptions};
use std::io::Write;
use std::path::PathBuf;


//Numeric constant storage size.
pub const DEFAULT_CAN_WH:[f64;2] = [1080.0,760.0];
pub const TEXT_BOX_H:f64 = 45.0;
pub const REP_BOX_W:f64 = 150.0;
pub const ROLL_BOX_W:f64 = 400.0;
pub const BORDER_WIDTH:f64 = 1.0;
pub const BORDER_BUTTON_WIDTH:f64 = 0.0;
pub const HOR_MAR:f64 = 0.0;
pub const VER_MAR:f64 = 0.0;
pub const TEXT_MARGIN:f64 = 3.0;
pub const VER_MAR_BOX:f64 = 5.0;
pub const HOR_MAR_BOX:f64 = 5.0;
pub const BORDER_COLOUR:color::Colour = color::BLACK;
pub const BACKGR_COLOUR:color::Colour = color::LIGHT_GREY;
pub const BACKGR_COLOUR_CALC:color::Colour = color::LIGHT_GREEN;
pub const BACKGR_COLOUR_MISCALC:color::Colour = color::LIGHT_RED;
pub const TEXTBOX_COLOUR:color::Colour = color::WHITE;
pub const BUTTON_COLOUR:color::Colour = color::LIGHT_GREY;
pub const REPS_DEFAULT:usize = 20000;
const AXIS_THICKNESS:f64 = 3.0;

//let m = ui.global_input().current.mouse.xy; useful for later.
//Another useful function: 
//parse_input(input:&str,title:&mut String,data:&mut Vec<(i32,f64)>,reps:&mut usize){}
//A variant of this will be indispensible:
//Switch of the displayed dungeon is needed.
//for _ in ui.widget_input(ids.dungeon_diary_entry).clicks() {
	//*gui_box = GUIBox::GameInspectDungeons(None);
//};

//A persistent structure, that keeps track of things.
pub struct Flow {
	pub init:bool,
	pub input: String,
	pub reps:usize,
	pub data:Vec<(String,Vec<(i32,f64)>)>,
	pub active_data:Option<(String,Vec<[f64;2]>,usize)>,
	pub axes: [String;2],
	pub rep_box: (String,bool),
	pub roll_box: (String,bool),
	pub calculating: bool,
	pub miscalculation: bool,
	
}

impl Flow {
	pub fn new()->Flow {
		Flow {
			init: false,
			input: String::with_capacity(60),
			reps: REPS_DEFAULT,
			data: Vec::with_capacity(20),
			active_data: None,
			axes: ["Bonus (Points)".to_owned(),"% Success".to_owned()],
			rep_box: (REP_BOX.to_owned(),false),
			roll_box: (ROLL_BOX.to_owned(),false),
			calculating: false,
			miscalculation: false,
		}
	}
}

widget_ids!(
	pub struct Ids {
		master,
			intro_canvas,
				intro_text,
			
			intro_button,
				
			calculating_canvas,
				calculating_text,
				
			reps_input,
			roll_input,
			coord_highlighted,
			
			data_canvas,
				datasets_matrix,
			remove_active_dataset_button,
			
			graph_canvas,
				title,
				x_axis,
					x_axis_title,
					x_axis_labels,
				y_axis,
					y_axis_title,
					y_axis_labels,
				line,
				x_pointer,
				y_pointer,
				coord_box,
					coord_text,	
	}
);

//Set the display.
pub fn set_widgets(ref mut ui: UiCell,ids:&Ids,flow:&mut Flow) {
	//Set the main canvas.
	let ui_wh = ui.wh_of(ids.master).unwrap_or(DEFAULT_CAN_WH);
	widget::Canvas::new().color(BACKGR_COLOUR).set(ids.master,ui);
	
	for _click in gen_button(ui,ids,[REP_BOX_W-HOR_MAR_BOX,TEXT_BOX_H])
							.label(HELP_BUTTON)
							.border(BORDER_WIDTH)
							.top_left_with_margin_on(ids.master,HOR_MAR_BOX)
							.set(ids.intro_button,ui){
		flow.init = true;
	};
	
	if flow.init {
		//If help is called for set the help canvas.
		set_intro_canvas(ui,ids,flow);
	}else{
		//Set the reps box.
		for edit in widget::TextBox::new(&flow.rep_box.0).w(REP_BOX_W-HOR_MAR_BOX)
												.h(TEXT_BOX_H)
												.color(TEXTBOX_COLOUR)
												.text_color(color::BLACK)
												.right_from(ids.intro_button,HOR_MAR_BOX)
												.set(ids.reps_input,ui) {
			match edit {
				conrod::widget::text_box::Event::Update(x)	=> {flow.rep_box.0 = x;},
				conrod::widget::text_box::Event::Enter		=> {
					flow.reps = flow.rep_box.0.parse().unwrap_or(REPS_DEFAULT);
					flow.rep_box.0 = REP_BOX.to_owned();
				},
			};
		}
		
		//Set the roll entry box.
		for edit in widget::TextBox::new(&flow.roll_box.0).w(ROLL_BOX_W-VER_MAR_BOX)
												.h(TEXT_BOX_H)
												.color(TEXTBOX_COLOUR)
												.text_color(color::BLACK)
												.right_from(ids.reps_input,HOR_MAR_BOX)
												.set(ids.roll_input,ui) {
			match edit {
				conrod::widget::text_box::Event::Update(x)	=> {flow.roll_box.0 = x;},
				conrod::widget::text_box::Event::Enter		=> {
					flow.roll_box.1 = true;
				},
			};
		}
		
		//if there is data, set the data selector matrix and graph.
		if flow.data.len()>0 {
			
			widget::Canvas::new()
				.w(ROLL_BOX_W-HOR_MAR_BOX)
				.h(ui_wh[1]-REP_BOX_W-VER_MAR_BOX*3.0)
				.down_from(ids.intro_button,VER_MAR_BOX)
				.scroll_kids_vertically()
				.color(BACKGR_COLOUR)
				.border(BORDER_WIDTH)
				.set(ids.data_canvas,ui);
			
			let mut dataset_matrix =  widget::Matrix::new(1,flow.data.len())
				.padded_w_of(ids.data_canvas,1.0)
				.h(TEXT_BOX_H*flow.data.len() as f64)
				.mid_top_with_margin_on(ids.data_canvas,1.0)
				.set(ids.datasets_matrix,ui);
				
			let mut button = gen_button(ui,ids,[ROLL_BOX_W-VER_MAR_BOX,TEXT_BOX_H]);
						   
			while let Some(db) = dataset_matrix.next(ui) {
				let r = db.row as usize;
				for _click in db.set(button.clone().label(&flow.data[r].0),ui) {
					flow.active_data = convert_data(&flow.data[r],r);
				};
			};
			
			//Draw the graph if we have some active data.
			let mut deactivate = false;
			match flow.active_data {
				Some((ref t,ref d, index)) => {
					let mut wh_graph = ui.wh_of(ids.data_canvas).unwrap();
					wh_graph[0] = ui_wh[0]-wh_graph[0]-HOR_MAR_BOX*3.0;
					
					let coord_out:String = azgraph::set_xy_line_graph(ui,ids,
													 d,
													 &flow.axes,
													 t,
													 AXIS_THICKNESS,
													 &wh_graph,
													 lab_no_chooser(&wh_graph),
													 (ids.data_canvas,'r')
					);
					
					widget::Text::new(&coord_out)
						.down_from(ids.graph_canvas,HOR_MAR_BOX)
						.set(ids.coord_highlighted,ui);
						
					for _click in button.clone().label(DELETE_ACTIVE)
												.border(BORDER_WIDTH)
												.down_from(ids.data_canvas,HOR_MAR_BOX)
												.set(ids.remove_active_dataset_button,ui) {
						flow.data.remove(index);
						deactivate = true;
					};
					
				},
				_		=> {},
			};
			if deactivate {flow.active_data = None;};
		};
		
		//If calculating, set calculating canvas.
		if flow.calculating {
			set_cold_and_calculating_canvas(ui,ids);
		}else if flow.miscalculation {
			set_cold_and_miscalculating_canvas(ui,ids,flow);
		};
	};
}

//Set the canvas that says that it's calculating.
fn set_cold_and_calculating_canvas(ui: &mut UiCell, ids: &Ids) {
	let ui_wh:[f64;2] = ui.wh_of(ids.master).unwrap_or(DEFAULT_CAN_WH);
	
	widget::Canvas::new().wh([ui_wh[0]/2.0,ui_wh[1]/2.0])
				 .middle_of(ids.master)
				 .color(BACKGR_COLOUR_CALC)
				 .border(BORDER_WIDTH)
				 .border_color(BORDER_COLOUR)
				 .set(ids.calculating_canvas,ui);
	
	widget::Text::new(COLD_AND_CALCULATING)
		.font_size(font_size_chooser(&ui_wh))
		.middle_of(ids.calculating_canvas)
		.padded_w_of(ids.calculating_canvas,4.0)
		.set(ids.calculating_text,ui);
}

//Set the canvas that says that it's made a pig's dinner out of the calculation.
fn set_cold_and_miscalculating_canvas(ui: &mut UiCell, ids: &Ids, flow: &mut Flow) {
	let ui_wh:[f64;2] = ui.wh_of(ids.master).unwrap_or(DEFAULT_CAN_WH);
	
	widget::Canvas::new().wh([ui_wh[0]/2.0,ui_wh[1]/2.0])
				 .middle_of(ids.master)
				 .color(BACKGR_COLOUR_MISCALC)
				 .border(BORDER_WIDTH)
				 .border_color(BORDER_COLOUR)
				 .set(ids.calculating_canvas,ui);
	
	widget::Text::new(COLD_AND_MISCALCULATING)
		.font_size(font_size_chooser(&ui_wh))
		.middle_of(ids.calculating_canvas)
		.padded_w_of(ids.calculating_canvas,4.0)
		.set(ids.calculating_text,ui);
		
	for _ in ui.widget_input(ids.calculating_canvas).clicks() {
		flow.miscalculation = false;
	};

	for _ in ui.widget_input(ids.calculating_text).clicks() {
		flow.miscalculation = false;
	};
}

//Set the help canvas.
fn set_intro_canvas(ui: &mut UiCell, ids: &Ids, flow: &mut Flow) {
	let ui_wh:[f64;2] = ui.wh_of(ids.master).unwrap_or(DEFAULT_CAN_WH);
	
	widget::Canvas::new().wh(ui_wh)
				 .middle_of(ids.master)
				 .color(BACKGR_COLOUR)
				 .border(BORDER_WIDTH)
				 .border_color(BORDER_COLOUR)
				 .set(ids.intro_canvas,ui);
				 
	widget::Text::new(HELP)
		.font_size(font_size_chooser(&ui_wh))
		.middle_of(ids.intro_canvas)
		.padded_w_of(ids.intro_canvas,4.0)
		.set(ids.intro_text,ui);
	
	for _ in ui.widget_input(ids.intro_text).clicks() {
		flow.init = false;
	};
	
	for _ in ui.widget_input(ids.intro_canvas).clicks() {
		flow.init = false;
	};
}

fn gen_button_marker(){}
fn gen_button<'a>(ui: &mut UiCell, ids: &Ids,wh:[f64;2])->widget::Button<'a,Flat> {
	
	let ui_wh:[f64;2] = ui.wh_of(ids.master).unwrap_or(DEFAULT_CAN_WH);
	
	widget::Button::new().wh(wh)
						 .color(BUTTON_COLOUR)
						 .border(BORDER_BUTTON_WIDTH)
						 .label_font_size(font_size_chooser(&ui_wh))
}

//font size chooser functions in a ui size dependant manner.
fn font_size_choose_marker(){}
pub fn font_size_chooser(wh_mc: &[f64;2]) -> u32 {
	if 		(wh_mc[0]<360.1) | (wh_mc[1]<280.1) {10}
	else if (wh_mc[0]<540.1) | (wh_mc[1]<360.1) {12}
	else if (wh_mc[0]<720.1) | (wh_mc[1]<580.1) {14}
	else if (wh_mc[0]<1080.1)| (wh_mc[1]<760.1) {16}
	else 										{20}
}

fn lab_no_chooser_marker(){}
fn lab_no_chooser(wh_gr: &[f64;2]) -> usize {
	if 		(wh_gr[0]<360.1) | (wh_gr[1]<280.1) {3}
	else if (wh_gr[0]<540.1) | (wh_gr[1]<360.1) {6}
	else if (wh_gr[0]<720.1) | (wh_gr[1]<480.1) {9}
	else										{11}
}

//Convert data to a format the graph drawer will take.
//NB it should be noted that initial data was (y,x), while it must become [x,y].
fn convert_data_marker(){}
fn convert_data(data:&(String,Vec<(i32,f64)>),index:usize) -> Option<(String,Vec<[f64;2]>,usize)> {
	let mut output_data:Vec<[f64;2]> = Vec::with_capacity(data.1.len());
	
	for (i,f) in data.1.iter() {output_data.push([*i as f64,*f])};
	Some((data.0.clone(),output_data,index))
	
}

//wrapper around clipboard copying.
pub fn copy_to_clipboard(text:&str){
	use clipboard::ClipboardProvider;
	let mut clipboard:clipboard::ClipboardContext = ClipboardProvider::new().unwrap();
	clipboard.set_contents(text.to_owned());
}
	

//STRING CONSTANT STORAGE AREA

const DELETE_ACTIVE:&str = "Delete Active Dataset";
const HELP_BUTTON:&str = "Help";
const REP_BOX:&str = "Input repeats";
pub const ROLL_BOX:&str = "Input roll";
const COLD_AND_CALCULATING:&str = "Calculating distribution by rolling lots of virtual dice. This may take some time.";
const COLD_AND_MISCALCULATING:&str = "I regret to inform you that I forgot how to read, write, count and roll dice. \
Therefore I couldn't answer to your expectation. Please burn me at the stake. Or try again with a different roll.";

const HELP:&str = "Welcome to AZDice (v0.0.1).

This is a dice roll probability table generator to help players, DMs and developers understand the balance in various types of rolls.
Thus the user enters the roll they wish to test (default 1d100 vs 1d100) and the number of test rolls the computer must perform to generate the statistics (default 20000).
The program then generates a probability of succeeding in the roll for every meaningful ability/skill/??/whatever bonus. The resulting table is saved as a .csv file and also displayed as a graph.

The y axis will display the probability of success of the roll in percent.
The x axis displays the advantage of the left side of the roll. (Eg in a spot check with spot bonux of +6 vs hide with skill of +8, the advantage of the spot roll is -2).

Example uses:

100:          Generates a 1d100 vs 1d100 table.
20:           Generates a 1d20 vs 1d20 table. (Eg opposed rolls in DnD)
3d6:          Generates a 3d6 vs 3d6 table.
3d6+1d20:     Generates a 3d6+1d20 vs 3d6+1d20 table.
3d6 vs 1d20:  Generates a 3d6 vs 1d20 table.
1d20 vs d0:   Generate a roll of 3d6 vs a range of bonuses without rolling. (Eg attack rolls vs armor in DnD)

GLHF.
~Aleksey Zholobenko, October 2018.
";
