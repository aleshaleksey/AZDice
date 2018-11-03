#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![recursion_limit="512"]



//Compile command:
//  cargo rustc --bin azdice-conrod --release --target=x86_64-pc-windows-gnu --features="winit glium libc" -- -C linker=x86_64-w64-mingw32-gcc -C link-args="-Wl,--subsystem,windows"
//	cargo rustc --bin azdice-conrod --release --features="winit glium libc"

#[macro_use]extern crate conrod;
#[macro_use]extern crate glium;
extern crate winit;
extern crate find_folder;
extern crate num;
extern crate rand;
extern crate clipboard;

mod libdice;
mod gui;
mod conrod_support;
mod azgraph;

use gui::*;
//use conrod_support::*;

use glium::Surface;
use std::sync::mpsc::{sync_channel,SyncSender,Receiver};
use std::thread;
use std::time::Duration;


#[allow(unused_mut)]
#[allow(unused_variables)]
#[allow(unused_must_use)]
#[allow(dead_code)]

const TEN:f64=10.0;
const FONT_FOLDER:&str = "NotoSans/NS.ttf";
const ASSETS:&str = "as";
const TITLE:&str = "AZDice";

//Important screen size settings for easy fiddling.
const WIDTH_INIT:u32 = 786;
const WIDTH_HELP:u32 = 1156;
const WIDTH_WARN:u32 = 1248;
const HEIGHT_INIT:u32 = 600;
const WIDTH_MIN:u32 = 786;
const HEIGHT_MIN:u32 = 600;
const F_SIZE:i32 = -15;



fn main(){						
// Build the window.
	println!("A");
	let mut events_loop = glium::glutin::EventsLoop::new();
	println!("el done");
	let window = glium::glutin::WindowBuilder::new()
		.with_title(TITLE)
		.with_dimensions(WIDTH_INIT, HEIGHT_INIT)
		.with_min_dimensions(WIDTH_MIN, HEIGHT_MIN);
	println!("win done");
	let context = glium::glutin::ContextBuilder::new()
		.with_vsync(true)
		.with_multisampling(4);
	println!("context done");
	let mut display = glium::Display::new(window, context, &events_loop).unwrap();
	println!("display done");

	// construct our `Ui`.
	let mut ui = conrod::UiBuilder::new([WIDTH_INIT as f64, HEIGHT_INIT as f64]).build();
	println!("ui done");

	// Add a `Font` to the `Ui`'s `font::Map` from file.
	let assets = find_folder::Search::KidsThenParents(3, 5).for_folder(ASSETS).unwrap();
	println!("font folder found");
	let font_path = assets.join(FONT_FOLDER);
	ui.fonts.insert_from_file(font_path).unwrap();
	println!("font loaded");

	// A type used for converting `conrod::render::Primitives` into `Command`s that can be used
	// for drawing to the glium `Surface`.
	let mut renderer = conrod_support::Renderer::new(&display).unwrap();
	println!("renderer done");

	// The image map describing each of our widget->image mappings (in our case, none).
	let mut image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
	println!("image_map done");

	// Instantiate the generated list of widget identifiers.
	let ids = &mut Ids::new(ui.widget_id_generator());
	println!("ids done");

	// Poll events from the window.
	let mut event_loop = conrod_support::EventLoop::new();
	println!("event_loop done");
	
	//Initiate persistent flow controller.
	let mut flow_controller = Flow::new();
	//a mini controller to say what Ctrl+C does.
	let mut copy_status:(u8,Option<conrod::widget::Id>) = (0,None);
	
	//Make the sender to for sending the calculation.
	let (sender_to_brain,receiver_in_brain)
	:(SyncSender<(usize,String)>,Receiver<(usize,String)>)
	= sync_channel(1);
	
	let (sender_to_body,receiver_in_body)
	:(SyncSender<Option<(String,Vec<(i32,f64)>)>>,Receiver<Option<(String,Vec<(i32,f64)>)>>)
	= sync_channel(1);
	
	let brain_thread = thread::spawn(
		move||{
			'brain_loop: loop {
				match receiver_in_brain.try_recv() {
					Ok(x) => {
						let output = libdice::parse_input(&x.1,x.0);
						sender_to_body.send(output);
					},
					_	  => {thread::sleep(Duration::from_millis(200));},
				};
			};
		}
	);
	
	'main: loop {
		
		//Talking to the brain.
		if flow_controller.roll_box.1 & !flow_controller.calculating {
			//If submission made to brain, turn on indicator and send request to brain.
			flow_controller.calculating = true;
			sender_to_brain.send((flow_controller.reps,flow_controller.roll_box.0.clone()));
			flow_controller.roll_box.0 = gui::ROLL_BOX.to_owned();
			
		}else if flow_controller.calculating {
			//If indicator on, try to beat answer out of the brain.
			match receiver_in_body.try_recv() {
				//If message received check if the answer came out egg shaped.
				Ok(x) => {
					flow_controller.roll_box.1 = false;
					flow_controller.calculating = false;
					match x {
						//If ok, add to the datasets. Else declare miscalc.
						Some(y) => {flow_controller.data.push(y);},
						None	=> {flow_controller.miscalculation = true;},
					};
				},
				_	  => {},
				
			}; 
		};
		
		// Instantiate all widgets in the GUI.
		set_widgets(ui.set_widgets(), ids,&mut flow_controller);

		// Render the `Ui` and then display it on the screen.
		if let Some(primitives) = ui.draw_if_changed() {
			renderer.fill(&display, primitives, &image_map);
			let mut target = display.draw();
			target.clear_color(0.0, 0.0, 0.0, 1.0);
			renderer.draw(&mut display, &mut target, &mut image_map,1.0,1.0).unwrap();
			target.finish().unwrap();
		}
		
		// Handle all events.
		for event in event_loop.next(&mut events_loop) {

			// Use the `winit` backend feature to convert the winit event to a conrod one.
			if let Some(event) = conrod_support::convert_event(event.clone(), &display) {
				ui.handle_event(event);
				event_loop.needs_update();
			}

			match event {
				glium::glutin::Event::WindowEvent { event, .. } => match event {
					
					// Break from the loop upon `Escape`.
					glium::glutin::WindowEvent::Closed |
					glium::glutin::WindowEvent::KeyboardInput {
						input: glium::glutin::KeyboardInput {
							virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
							..
						},
						..
					} => break 'main,
					
					//Controlbutton.
					glium::glutin::WindowEvent::KeyboardInput {
						input: glium::glutin::KeyboardInput {
							virtual_keycode: Some(glium::glutin::VirtualKeyCode::LControl),
							..
						},
						..
					}|
					glium::glutin::WindowEvent::KeyboardInput {
						input: glium::glutin::KeyboardInput {
							virtual_keycode: Some(glium::glutin::VirtualKeyCode::RControl),
							..
						},
						..
					} => {
						//Ctrl+C, Ctrl+V script.
					},
					
					//C button.
					glium::glutin::WindowEvent::KeyboardInput {
						input: glium::glutin::KeyboardInput {
							virtual_keycode: Some(glium::glutin::VirtualKeyCode::C),
							..
						},
						..
					} => {copy_status.0+= 1;},
					_ => (),
				},
				_ => (),
			}
		}
		
		//brain_thread.join();
		//Copy by buttons.
		//Use this function:
		//gui::copy_to_clipboard(&string);

	}
}
			
   


fn lshas(a:String, b:char)->bool{
	for x in a.chars(){
		if x==b {return true};
	}
	false
}
