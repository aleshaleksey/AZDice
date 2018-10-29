//Draft module for drawing line graph.

extern crate conrod;

use conrod::UiCell;
use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};
use conrod::widget::Id;
use gui::Ids;
use gui;
use std::str::Chars;

		//graph_canvas,
			//title,
			//x_axis,
				//x_axis_title,
				//x_axis_labels,
			//y_axis,
				//y_axis_title,
				//y_axis_labels,
			//line,
			//x_pointer,
			//y_pointer,
			//coord_box,
				//coord_text,	

//The master function for drawing and setting a graph.
//Currently this function is not generic enough.
fn set_xy_line_graph_marker(){}
pub fn set_xy_line_graph(ui: &mut UiCell,ids:&Ids,
				 data:&Vec<[f64;2]>,
				 axes:&[String;2],
				 title:&String,
				 axth: f64,
				 size:&[f64;2],
				 no_lab:usize,
				 pos:(Id,char))->String {
	//Set graph canvas.
	let mut can = widget::Canvas::new()
						.color(gui::TEXTBOX_COLOUR)
						.wh([size[0],size[1]]);
	
	//Position it according to stuff.				
	if		pos.1=='u' {can = can.up_from(pos.0,gui::VER_MAR_BOX);}
	else if pos.1=='d' {can = can.down_from(pos.0,gui::VER_MAR_BOX);}
	else if pos.1=='l' {can = can.left_from(pos.0,gui::VER_MAR_BOX);}
	else			   {can = can.right_from(pos.0,gui::VER_MAR_BOX);};
	can.set(ids.graph_canvas,ui);
	
	widget::Text::new(title).font_size(gui::font_size_chooser(&size)*2)
							.mid_top_with_margin_on(ids.graph_canvas,gui::VER_MAR_BOX)
							.set(ids.title,ui);
	
	
	let cog = ui.xy_of(ids.graph_canvas).unwrap(); //CentreOfGraph
	let blog = [cog[0]-size[0]/2.0,cog[1]-size[1]/2.0];  //BottomLeftOfGraph.
	let gm = 40.0; //GraphMargins (experimental)
	
	let (xs,xe,ys,ye) = (blog[0]+2.0*gm,blog[0]+size[0]-gm,blog[1]+2.0*gm,blog[1]+size[1]-gm);
	
	
	let dl = data.len() as f64;
	let (xxl,yxl) = (xe-xs,ye-ys);
	let (mmx,mmy) = find_minmax_data(data);
	let (xdr,ydr) = (xxl/(mmx[1]-mmx[0]),yxl/(mmy[1]-mmy[0]));
	
	let graph_labels = generate_axes_labels(data,no_lab,&mmx,&mmy);
	let scaled_points = scale_to_graph(data,xdr,ydr,xs,ys,mmx[0],mmy[0]);		
	
	//Draw the line
	widget::PointPath::abs(scaled_points.clone())
		.thickness(axth)
		.color(color::BLUE)
		.set(ids.line,ui);
		
	//Draw the y axis.
	widget::Line::abs([xs,ys],[xs,ye])
				.thickness(axth)
				.color(gui::BORDER_COLOUR)
				.set(ids.y_axis,ui);
	
	//Draw the x axis.			
	widget::Line::abs([xs,ys],[xe,ys])
				.thickness(axth)
				.color(gui::BORDER_COLOUR)
				.set(ids.x_axis,ui);
	
	// A little bit of magic to get everything to align.
	let correction:f64 = no_lab as f64/(no_lab-1) as f64;
	let mut posx = ui.xy_of(ids.x_axis).unwrap();
	let mut posy = ui.xy_of(ids.y_axis).unwrap();
	posx[1] = posx[1]-conrod::text::pt_to_px(gui::font_size_chooser(&size)) as f64;
	posy = [posy[0]-gm,posy[1]-gm/2.0];

	//Insert x_axis labels
	let mut x_label_matrix = widget::Matrix::new(no_lab,1)
		//.color(color::BLACK.with_alpha(0.0))
		.w((xe-xs)*correction)
		.h(conrod::text::pt_to_px(gui::font_size_chooser(&size)) as f64)
		.xy(posx)
		.set(ids.x_axis_labels,ui);
		
	//Insert y_axis labels
	let mut y_label_matrix = widget::Matrix::new(1,no_lab)
		//.color(color::BLACK.with_alpha(0.0))
		.h((ye-ys)*correction)
		.w(gm)
		.xy(posy)
		.set(ids.y_axis_labels,ui);
	
	//Functionalise labels.		
	while let Some(x_lab) = x_label_matrix.next(ui) {
		let r = x_lab.col;
		if r<graph_labels.len() {
			let label = widget::Text::new(&graph_labels[r][0].1)
									.center_justify()
									.font_size(gui::font_size_chooser(&size));
			x_lab.set(label,ui);
		};
	};
	
	while let Some(y_lab) = y_label_matrix.next(ui) {
		let mut r = y_lab.row;
		r =  if no_lab-1-r>=graph_labels.len() {graph_labels.len()-1}else{no_lab-1-r};
		let label = widget::Text::new(&graph_labels[r][1].1)
								.right_justify()
								.font_size(gui::font_size_chooser(&size));
		y_lab.set(label,ui);
	};
	
	//Set x axis title.
	widget::Text::new(&axes[0])
				.center_justify()
				.font_size(gui::font_size_chooser(&size))
				.down_from(ids.x_axis_labels,0.0)
				.x(xe-gm)
				.set(ids.x_axis_title,ui);
				
	//Set y axis title.
	widget::Text::new(&axes[1])
				.center_justify()
				.font_size(gui::font_size_chooser(&size))
				.up_from(ids.y_axis,0.0)
				.set(ids.y_axis_title,ui);
				
	
	//Get mouse coords.
	let mut xy = ui.global_input().current.mouse.xy;
	
	//Convert to x-y of nearest graph pixel-coord.
	get_nearest(&mut xy,&scaled_points,ids.y_axis,ids.x_axis,ui,[xs,xe],[ys,ye]);
	
	//Draw pointers if pointer is within axes.
	if (xy[0]>xs) & (xy[0]<xe) & (xy[1]>ys) & (xy[1]<ye) {
		//Draw the x axis pointer line.		
		widget::Line::abs([xs,xy[1]],[xy[0],xy[1]])
					.thickness(1.0)
					.color(color::BLUE)
					.set(ids.x_pointer,ui);
				
		//Draw the y axis pointer line.			
		widget::Line::abs([xy[0],ys],[xy[0],xy[1]])
					.thickness(1.0)
					.color(color::BLUE)
					.set(ids.y_pointer,ui);
	};
	
	//Convert graph pixel-coord to graph data point.
	let m = unscale_from_pixels(xy,[xs,ys],[xdr,ydr],[mmx[0],mmy[0]]);
	
	//Output datapoint to print.
	format!("Bonus={:#.0}\n% Success={:#.1}",m[0],m[1])
}

//A function to snap x to x-axis or y to y-axis, or both to line.
//Currently this function is heavy, requiring 8 boolian comparisons per point of data.
//It also requires iterating through all points of data. I will need to make a more efficient iterator.
fn get_nearest(xy:&mut [f64;2],lic:&Vec<[f64;2]>,xid:Id,yid:Id,ui:&conrod::UiCell,mmx:[f64;2],mmy:[f64;2]) {
	let coord_x = ui.xy_of(xid).unwrap()[1]; //We only care about y-pos of x-axis.
	let coord_y = ui.xy_of(yid).unwrap()[0]; //We only care about x-pos of y-axis.
	
	//assume mouse coordinates are less then minimum.
	let mut nearest_x = [mmx[0],mmy[0]];
	let mut nearest_y = [mmx[0],mmy[0]];
	
	//Check if mouse coordinates fall between point values.
	for i in 1..lic.len() {
		if ((xy[0]>lic[i-1][0]) & (xy[0]<=lic[i][0]))
		|  ((xy[0]<lic[i-1][0]) & (xy[0]>=lic[i][0])) {nearest_x = lic[i];};
		if ((xy[1]>lic[i-1][1]) & (xy[1]<=lic[i][1]))
		|  ((xy[1]<lic[i-1][1]) & (xy[1]>=lic[i][1])) {nearest_y = lic[i];};
	};
	
	if xy[0]>mmx[1] {nearest_x = [mmx[1],mmy[1]];};
	if xy[1]>mmy[1] {nearest_y = [mmx[1],mmy[1]];};
	
	if	nearest_y[0]>=xy[0] {xy[0] = nearest_y[0];
	}else{xy[1] = nearest_x[1];};
}



//A function to put pixels back to what they should be.
fn unscale_from_pixels(xy:[f64;2],origin:[f64;2],scale:[f64;2],min:[f64;2])->[f64;2] {
	
	[(xy[0]-origin[0])/scale[0]+min[0],(xy[1]-origin[1])/scale[1]+min[1]]
}

fn scale_to_graph_1marker(){}
fn scale_to_graph (data:&Vec<[f64;2]>,xdr:f64,ydr:f64,xs:f64,ys:f64,mmx:f64,mmy:f64)->Vec<[f64;2]> {
	let mut output = Vec::with_capacity(data.len());
	
	for p in data.iter() {
		let x = (p[0]-mmx)*xdr+xs;
		let y = (p[1]-mmy)*ydr+ys;
		output.push([x,y]);
	};
	
	output
}

//A function to generate axes labels as strings and their positions.
fn generate_axes_labels_marker(){}
fn generate_axes_labels(data:&Vec<[f64;2]>,label_number:usize,mmx:&[f64;2],mmy:&[f64;2]) -> Vec<[(f64,String);2]> {
	let mut output = Vec::with_capacity(label_number);
	
	let x_gap = (mmx[1]-mmx[0])/(label_number-1) as f64;
	let y_gap = (mmy[1]-mmy[0])/(label_number-1) as f64;
	
	for i in 0..label_number {
		output.push([(mmx[0]+x_gap*i as f64,ff(3,mmx[0]+x_gap*i as f64)),
					 (mmy[0]+y_gap*i as f64,ff(3,mmy[0]+y_gap*i as f64))
		]);
	};
	output
}

//Find minmaxes data=Vec<[x,y]>, output = ([min_x,max_x],[min_y,max_y])
fn find_minmax_data_marker(){}
fn find_minmax_data(data:&Vec<[f64;2]>)->([f64;2],[f64;2]) {
	let mut output = ([data[0][0],data[0][0]],[data[0][1],data[0][1]]);
	
	for z in data.iter() {
		if z[0]<output.0[0] {output.0[0] = z[0];};
		if z[0]>output.0[1] {output.0[1] = z[0];};
		if z[1]<output.1[0] {output.1[0] = z[1];};
		if z[1]>output.1[1] {output.1[1] = z[1];};
	};
	
	output
}



//Borrowing a chmquiz function.
fn ff(figs:usize,old:f64)->String {
	let old_as_string = old.to_string();
	
	//make string of insignificant figures.
	let insigs = "0.+-";
	
	//make a receptacle string for the new number.
	let mut new = String::new();
	//make a signifacant figure counter.
	let mut sig_figs:usize = 0;
	//make an indicaor for whether it's started or not.
	let mut count = false;
	//do the work.
	let mut has_dot = false;
	for x in old_as_string.chars(){
		if !lshash(insigs.chars(),x) {count=true;};
		if x=='.' {has_dot=true;};
		if count {sig_figs+= 1;};
		if (x=='-')||(x=='+'){
		}else if (old*old<1.0) & (sig_figs<=(figs+1)){
			new.push(x)
		}else if (old*old>=1.0) & (sig_figs<=(figs+2)){
			new.push(x)
		}else if !has_dot{
			new.push(x);
		};
	};

	let mut out = String::new();
	let mut lenn = new.chars().count();
	
	//retrieve last character.
	let l = new.chars().nth(lenn-1).unwrap();
	//decide whether to round or return as is.
	let mut round = if sig_figs<=4 {
		if old<0.0 {out.push('-');};		
		out.push_str(&new);
		return out
	}else if (sig_figs>4) & ((l=='5')|(l=='6')|(l=='7')|(l=='8')|(l=='9')) {true}else{false};
	let ln = new.chars().nth(figs);
	round = if !has_dot & ((ln==Some('5'))|(ln==Some('6'))|(ln==Some('7'))|(ln==Some('8'))|(ln==Some('9'))){
		true
	}else if !has_dot {
		false
	}else{
		round
	};
	
	if has_dot & (sig_figs>4) {
		new.pop();
		lenn-= 1;
	};
	
	//construct a final output.
	let mut counter:usize = 0;
	for x in new.chars().rev(){
		counter+= 1;
		if !has_dot & (lenn-counter>figs-1){
			out.push('0')
		}else{
			if !round {
				out.push(x)
			}else{
				let to_do = match_to_round(round,x);
				round = to_do.0;
				if round & (counter==(lenn)){
					out.push(to_do.1);
					out.push('1')
				}else{
					out.push(to_do.1)
				};
			};
		};
	};
	

	if old<0.0 {out.push('-');};		

	//Bleh- can't find function.
	if out.chars().rev().last()==Some('.') {
		let mut a:String = out.chars().rev().collect();
		a.pop();
		a
	}else{
		out.chars().rev().collect()
	}
	//out.chars().rev().collect()
}


fn lshash(a:Chars, b:char)->bool{
	let mut ihaz=false;
	for x in a{
		if x==b{
			ihaz=true;
			return ihaz}
		else{continue}
	}
	ihaz
}

//decimal rounding table.
pub fn match_to_round(round:bool,ch:char)->(bool,char){
	if !round {
		return (false,ch)
	}else{
		match ch {
			'0' => (false,'1'),
			'1' => (false,'2'),
			'2' => (false,'3'),
			'3' => (false,'4'),
			'4' => (false,'5'),
			'5' => (false,'6'),
			'6' => (false,'7'),
			'7' => (false,'8'),
			'8' => (false,'9'),
			'9' => (true,'0'),
			_   => (true,ch),
		}
	}
}
