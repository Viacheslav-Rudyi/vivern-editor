use std::{collections::HashMap, fs, hash::Hash, path::{Path, PathBuf}};

use eframe::{Frame, egui::{self, Pos2, Rect, Sense, response}};
use serde::de;
use serde::{Deserialize, Serialize};
use rfd;

use path_absolutize::*;

use crate::{myapp::MyApp};
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum Value {
	text(String),
	integer(isize),
	float(f64),
	dialogue(Vec<String>),
	choice(Vec<String>),
	destination(Vec<Option<usize>>),
	path(PathBuf),
	properties(HashMap<String, f64>),
	expressions(Vec<String>),
	paths(Vec<PathBuf>),
	next(Option<usize>),
	rgb([f32; 3]),
	id(usize),
	flag(bool),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct GraphNode {
	pub id: usize,
	pub next: Option<usize>,
	pub title: String,
	pub pos: egui::Pos2,
	pub data: HashMap<String, Value>,
	rect: Option<egui::Rect>,
	titleLength: Option<f32>,
	pub selectedAt: Option<Pos2>,
	pub releasedAt: Option<Pos2>,
	pub inputPos: Pos2,
	pub outPos: Pos2,
	
	pub fromChoice: Option<usize>,
	pub choicePos: Vec<Pos2>,

	pub queueDelete: bool,

	pub expressionPaths: Vec<PathBuf>,
	//pub defaultExpressionPath: PathBuf,
	pub dir: Option<PathBuf>,
	pub visible: bool,
}

impl Default for GraphNode {
	fn default() -> Self {
		Self {
			id: 1,
			title: "Dialogue".to_owned(),
			pos: egui::Pos2 { x: (120.), y: (340.) },
			data: HashMap::new(),
			rect: None,
			titleLength: None,
			next: None,
			selectedAt: None,
			releasedAt: None,
			inputPos: egui::Pos2::ZERO,
			outPos: egui::Pos2::ZERO,
			fromChoice: None,
			choicePos: vec![],
			queueDelete: false,
			expressionPaths: vec![],
			//defaultExpressionPath: PathBuf::new(),
			dir: None,
			visible: true,
		}
	}
}

impl GraphNode {
	pub fn new(id: usize, title: String, pos: egui::Pos2) -> Self {
		Self {
			id: id,
			title: title,
			pos: pos,
			next: None,
			data: HashMap::new(),
			rect: None,
			titleLength: None,
			selectedAt: None,
			releasedAt: None,
			inputPos: egui::Pos2::ZERO,
			outPos: egui::Pos2::ZERO,
			fromChoice: None,
			choicePos: vec![],
			queueDelete: false,

			expressionPaths: vec![PathBuf::new()],

			//defaultExpressionPath: PathBuf::new(),

			dir: None,
			visible: true,
		}
	}

	pub fn display(&mut self, ui: &mut egui::Ui, ctx:&egui::Context, bg: Option<Vec<String>>, app: &mut MyApp) {
		let offset = &app.offset.to_owned();
		let mut area = egui::Area::new(egui::Id::new(&self.id))
		.order(egui::Order::Background)
		.current_pos(self.pos + *offset)
		.movable(false)
		.constrain(false)
		.sense(Sense::drag())
		.show(ctx, |ui| {
			ui.group(|ui| {
				let header = ui.horizontal(|ui| {
					let shoudlAddSpace = self.rect.is_some() && self.titleLength.is_some();

					let mut input = None;
					if self.title.contains("Define") == false && self.title != "Root" {
						input = Some(self.addInputButton(ui));
					}
					let mut inputRectWidth = 0.;
					if input.is_some() {
						inputRectWidth = input.unwrap().rect.width();
					}
					
					if shoudlAddSpace {
						let amount = self.rect.unwrap().width() - self.titleLength.unwrap() - inputRectWidth * 2.;
						ui.add_space(amount / 2. - 15.);
					}

					let title = self.addNodeTitle(ui, offset, app);
					self.titleLength = Some(title.rect.width());

					if shoudlAddSpace {
						let amount = self.rect.unwrap().width() - self.titleLength.unwrap() - inputRectWidth * 2.;
						ui.add_space(amount / 2. - 15.);
					}

					if (self.title == "Choice"
						|| self.title == "Switch Scene"
						|| self.title.contains("Define")) == false { self.addOutputButton(ui); }
				}).response;
				
				self.addNodeBody(ui, app);
			})
		}).response;

		if area.dragged() {
			//self.pos += area.drag_delta();
		}

		if self.rect.is_none() {
			self.rect = Some(area.rect);
		}
	}

	pub fn addNodeTitle(&mut self, ui: &mut egui::Ui, offset: &egui::Vec2, app: &mut MyApp) -> egui::Response {
		let title = egui::Label::new(&self.title).selectable(false).sense(egui::Sense::click_and_drag());
		let drag = ui.add(title);

		if drag.dragged_by(egui::PointerButton::Primary) {
			// self.pos = drag.interact_pointer_pos().unwrap_or(self.pos) - *offset;
			// self.pos.x -= drag.rect.width() / 2. + but.rect.width();
			self.pos += drag.drag_delta();
		}
		if drag.dragged_by(egui::PointerButton::Secondary) {
			app.force_movement = Some((self.id, drag.drag_delta()));
		}

		if drag.clicked() {
			//dbg!(&self);
		}

		if drag.double_clicked() {
			if self.title != "Root" {
				self.queueDelete = true
			}
		}
		if drag.double_clicked_by(egui::PointerButton::Secondary) {
			if self.title != "Root" {
				app.chain_delete = Some(self.id);
			}
		}

		drag
	}

	pub fn addInputButton(&mut self, ui: &mut egui::Ui) -> egui::Response {
		let button = egui::Button::new("◀");
		let response = ui.add(button);

		if ui.input(|input| {
			input.pointer.primary_released()
		}) {
			if response.hovered() {
				self.releasedAt = Some(response.rect.center());
			}
		}

		self.inputPos = response.rect.center();

		response
	}

	pub fn addOutputButton(&mut self, ui: &mut egui::Ui) -> egui::Response {
		let button = egui::Button::new("▶");
		let response = ui.add(button);

		if response.is_pointer_button_down_on() {
			self.selectedAt = Some(response.rect.center());
		}

		if response.clicked() {
			self.next = None;
		}

		self.outPos = response.rect.center();

		response
	}

	fn addNodeBody(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		match self.title.as_str() {
			"Dialogue" => {
				self.populateDialogue(ui);
			}
			"Choice" => {
				ui.separator();
				self.populateChoice(ui);
			}
			"Set Background" => {
				ui.separator();
				self.populateSetBackground(ui, app);
			}
			"Set Background Color" => {
				ui.separator();
				self.populateSetBackgroundColor(ui);
			}
			"Play Music" => {
				ui.separator();
				self.populateMusic(ui, app);
			}
			"Stop Music" => {
				// ui.separator();
				// self.populateMusic(ui, app);
			}
			"Play SFX" => {
				ui.separator();
				self.populateSFX(ui, app);
			}
			"Define Background" => {
				ui.separator();
				self.populateDefineBackground(ui, app);
			}
			"Show Character" => {
				ui.separator();
				self.populateShowSprite(ui, app);
			}
			"Define Character" => {
				ui.separator();
				self.populateDefineCharacter(ui, app);
			}
			"Define SFX" => {
				ui.separator();
				self.populateDefineSFX(ui, app);
			}
			"Define Music" => {
				ui.separator();
				self.populateDefineMusic(ui, app);
			}
			"Wait" => {

			}
			"Wait For" => {
				ui.separator();
				self.populateDefineWaitFor(ui, app);
			}
			"Hide Messagebox" => {

			}
			"Hide Character" => {
				ui.separator();
				self.populateHideCharacter(ui, app);
			}
			"Switch Scene" => {
				ui.separator();
				self.populateSwitchScene(ui, app);
			}
			"Slide" | "Pulse" | "Breathe" | "Clear Effects" => {
				ui.separator();
				self.populateVFX(ui, app);
			}
			_ => {

			}
		}
	}

	fn populateDialogue(&mut self, ui: &mut egui::Ui) {
		let mut ss: &mut String = &mut "".to_owned();
		match self.data.get_mut("speaker").unwrap() {
			Value::text(s) => {
				ss = s;
			}
			_ => {}
		}
		let speaker = egui::TextEdit::singleline(ss).hint_text("Speaker: ");
		ui.add(speaker);
		ui.separator();

		let mut dialogue: Option<&mut Vec<String>> = match self.data.get_mut("text").unwrap() {
			Value::dialogue(v) => {
				Some(v)
			}
			_ => { None }
		};

		if dialogue.is_some() {
			let mut remove = None;
			let mut counter = 0usize;
			for line in dialogue.as_mut().unwrap().iter_mut() {
				let edit = egui::TextEdit::multiline(line).hint_text("Enter dialogue line");
				let resp = ui.add(edit);
				if resp.double_clicked() {
					remove = Some(counter);
				}
				counter += 1;
			}
			if remove.is_some() {
				dialogue.as_mut().unwrap().remove(remove.unwrap());
			}
		}

		ui.separator();
		ui.horizontal(|ui| {
			if self.rect.is_some() {
				ui.add_space(self.rect.unwrap().width() / 2. - 10.);
			}
			if ui.button("+").clicked() {
				if dialogue.is_some() {
					dialogue.unwrap().push(String::new());
				}
			}	
		});
	}

	fn populateChoice(&mut self, ui: &mut egui::Ui) {
		self.choicePos.clear();
		let mut choices = match self.data.get_mut("choices").unwrap() {
			Value::choice(c) => {
				Some(c)
			}
			_ => {
				None
			}
		};

		let mut toDelete = None;
		let mut toRemoveConnection = None;
		if choices.is_some() {
			let length = choices.as_mut().unwrap().len();
			let choices_mut = choices.as_mut().unwrap();
			for i in 0..length {
				ui.horizontal(|ui| {
					if ui.button("x").clicked() {
						toDelete = Some(i);
					}
					
					let edit = egui::TextEdit::singleline(choices_mut.get_mut(i).unwrap()).desired_width(100.).hint_text("Option:");
					ui.add(edit);

					// ADD CHOICE BUTTON
					let button = egui::Button::new("▶");
					let response = ui.add(button);

					if response.is_pointer_button_down_on() {
						self.selectedAt = Some(response.rect.center());
						self.fromChoice = Some(i);
					}

					if response.clicked() {
						toRemoveConnection = Some(i);
					}


					self.choicePos.push(response.rect.center());

					self.outPos = response.rect.center();
				});
			}
		}

		if toDelete.is_some() {
			if choices.is_some() {
				choices.as_mut().unwrap().remove(toDelete.unwrap());
			}
			
			match self.data.get_mut("destinations").unwrap() {
				Value::destination(c) => {
					c.remove(toDelete.unwrap());
				}
				_ => {
					
				}
			}
		}

		if toRemoveConnection.is_some() {
			match self.data.get_mut("destinations").unwrap() {
				Value::destination(c) => {
					c[toRemoveConnection.unwrap()] = None;
				}
				_ => {
					
				}
			}
		}

		ui.separator();
		ui.horizontal(|ui| {
			if self.rect.is_some() {
				ui.add_space(self.rect.unwrap().width() / 2. - 10.);
			}

			if ui.button("+").clicked() {
				if self.data.get_mut("choices").is_some() {
					match self.data.get_mut("choices").unwrap() {
						Value::choice(c) => {
							c.push(String::new());
						}
						_ => {

						}
					}

					match self.data.get_mut("destinations").unwrap() {
						Value::destination(c) => {
							c.push(None);
						}
						_ => {
							
						}
					}
				}
			}
		});
		
	}

	pub fn addChoiceButton(&mut self, ui: &mut egui::Ui) -> egui::Response {
		let button = egui::Button::new("▶");
		let response = ui.add(button);

		if response.is_pointer_button_down_on() {
			self.selectedAt = Some(response.rect.center());
		}

		self.outPos = response.rect.center();

		response
	}

	fn populateSetBackground(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		let mut current = match self.data.get("sprite").unwrap() {
			Value::id(id) => {*id}
			_ => { 0 }
		};

		let before = current.clone();
		let text = if app.backgrounds.contains_key(&current) { &app.backgrounds[&current]}
					else { "No background selected!" };
		ui.label("Background");
		let combo = egui::ComboBox::from_label("")
			.selected_text(format!("{:?}", text))
			.show_ui(ui, |ui| {
				let list = &app.backgrounds;
				for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) {
								ui.selectable_value(&mut current, *index, name);
							}
						}
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) == false {
								ui.selectable_value(&mut current, *index, name);
							}
						}
			});

		if before != current {
			let b = self.data.get_mut("sprite").unwrap();
			*b = Value::id(current);
		}
	}

	fn populateMusic(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		let mut current = match self.data.get("sfx").unwrap() {
			Value::id(id) => {*id}
			_ => { 0 }
		};

		let before = current.clone();
		let text = if app.music.contains_key(&current) { &app.music[&current]}
					else { "No music selected!" };
		ui.label("Music");
		let combo = egui::ComboBox::from_label("")
			.selected_text(format!("{:?}", text))
			.show_ui(ui, |ui| {
				let list = &app.music;
				for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) {
								ui.selectable_value(&mut current, *index, name);
							}
						}
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) == false {
								ui.selectable_value(&mut current, *index, name);
							}
						}
			});
		
		let volume = match self.data.get_mut("volume").unwrap() {
			Value::float(f) => { Some(f) }
			_ => { None }
		};
		if volume.is_some() {
			let slider = egui::Slider::new(volume.unwrap(), 0. ..= 5.);
			ui.add(slider);
		}
		

		if before != current {
			let b = self.data.get_mut("sfx").unwrap();
			*b = Value::id(current);
		}
	}

	fn populateSFX(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		let mut current = match self.data.get("sfx").unwrap() {
			Value::id(id) => {*id}
			_ => { 0 }
		};

		let before = current.clone();
		let text = if app.effects.contains_key(&current) { &app.effects[&current]}
					else { "No sfx selected!" };
		ui.label("Effect");
		let combo = egui::ComboBox::from_label("")
			.selected_text(format!("{:?}", text))
			.show_ui(ui, |ui| {
				let list = &app.effects;
				for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) {
								ui.selectable_value(&mut current, *index, name);
							}
						}
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) == false {
								ui.selectable_value(&mut current, *index, name);
							}
						}
			});

		let volume = match self.data.get_mut("volume").unwrap() {
			Value::float(f) => { Some(f) }
			_ => { None }
		};
		if volume.is_some() {
			let slider = egui::Slider::new(volume.unwrap(), 0. ..= 5.);
			ui.add(slider);
		}


		if before != current {
			let b = self.data.get_mut("sfx").unwrap();
			*b = Value::id(current);
		}
	}

	fn populateSetBackgroundColor(&mut self, ui: &mut egui::Ui) {
		let mut current = match self.data.get_mut("color") {
			Some(v) => {
				match v {
					Value::rgb(v) => { Some(v) }
					_ => { None }
				}
			}
			_ => { None }
		};
		
		if current.is_some() {
			egui::Grid::new(self.id)
				.num_columns(2)
				.show(ui, |ui| {
					ui.label("Pick Color: ");
					let picker = egui::color_picker::color_edit_button_rgb(ui, current.unwrap());
				});
		}
	}

	fn populateDefineBackground(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		ui.horizontal(|ui| {
			let mut ss: &mut String = &mut "".to_owned();
			match self.data.get_mut("sprite").unwrap() {
				Value::text(s) => {
					ss = s;
				}
				_ => {}
			}
			let prev = ss.to_string();
			let bg = egui::TextEdit::singleline(ss).hint_text("Background: ").desired_width(120.);

			let response = ui.add(bg);

			if response.changed() {
				let val = app.backgrounds.get_mut(&self.id);
				if val.is_some() {
					let val = val.unwrap();
					*val = ss.to_string()
				}
			}

			if ui.button("Select file").clicked()
				&& let Some(path) = rfd::FileDialog::new().add_filter("Image", &["jpg", "png", "jpeg"]).pick_file() {
				let data = match self.data.get_mut("path").unwrap() {
					Value::path(p) => {
						Some(p)
					}
					_ => {None }
				};
				if data.is_some() {
					println!("{:?}", path);
					if self.dir.is_some() {
						let relative =  pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
						if relative.is_some() {
							println!("Relative path constructed");
							*data.unwrap() = relative.unwrap();
						}
						else {
							*data.unwrap() = path;
						}
					}
					else {
						*data.unwrap() = path;
					}
				}
			}
		});
		

		let path = match self.data.get("path").unwrap() {
			Value::path(p) => { Some(p) }
			_ => { None }
		};

		if path.is_some() {
			let mut p = "file://".to_string();
			if self.dir.is_some() {
				let abs = path.unwrap().absolutize_from(self.dir.as_ref().unwrap());
				if abs.is_ok() {
					p.push_str(abs.unwrap().to_str().unwrap());
				}
				else {
					p.push_str(path.unwrap().to_str().unwrap());	
				}
			}
			else {
				p.push_str(path.unwrap().to_str().unwrap());
			}
			let texture = egui::ImageSource::Uri(p.into());

			ui.separator();
			let response = ui.image(texture);
		}
	}

	fn populateDefineCharacter(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		let mut sprite = match self.data.get_mut("sprite").unwrap() {
			Value::text(t) => { t }
			_ => { &mut "".to_owned() }
		};
		let prev = sprite.to_string();

		let edit = egui::TextEdit::singleline(sprite).hint_text("Character Name:");
		let response = ui.add(edit);

		if response.changed() {
			let val = app.sprites.get_mut(&self.id).unwrap();
			*val = sprite.to_owned();
		}

		let sprite = sprite.to_string();

		ui.separator();
		ui.label("Expresions:");

		ui.horizontal(|ui| {
			ui.label("Default:");
			if ui.button("Select file").clicked()
				&& let Some(path) = rfd::FileDialog::new().add_filter("Image", &["jpg", "png", "jpeg"]).pick_file() {
					if path != self.expressionPaths[0] {
						let mut uri = "file://".to_string();
						uri.push_str(path.to_str().unwrap());
						ui.ctx().forget_image(&uri);
					}

					if self.dir.is_some() {
						let relative = pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
						if relative.is_some() {
							self.expressionPaths[0] = relative.unwrap();
						}
						else {
							self.expressionPaths[0] = path;	
						}
					}
					else {
						self.expressionPaths[0] = path;
					}
					
			}
		});
		let defPreview = egui::CollapsingHeader::new("Preview").id_salt(-1).show(ui, |ui| {
			let mut p = "file://".to_string();
			if self.dir.is_some() {
				let abs = self.expressionPaths[0].absolutize_from(self.dir.as_ref().unwrap());
				if abs.is_ok() {
					p.push_str(abs.unwrap().to_str().unwrap());
				}
				else {
					p.push_str(self.expressionPaths[0].to_str().unwrap());	
				}
			}
			else {
				p.push_str(self.expressionPaths[0].to_str().unwrap());
			}
			let texture = egui::ImageSource::Uri(p.into());
			let response = ui.image(texture);
		});
		ui.separator();
		
		let mut index: usize = 0;
		let mut indexToRemove = None;
		let mut updateValues = false;
		for exp in app.expressions.get_mut(&self.id).unwrap_or(&mut vec![]) {
			if index == 0 {
				index += 1;
				continue;
			}
			ui.horizontal(|ui| {
				let edit = egui::TextEdit::singleline(exp).hint_text("Expression:").desired_width(120.);
				let response = ui.add(edit);

				if response.changed() {
					//updateValues = true;
				}

				if ui.button("Select file").clicked()
					&& let Some(path) = rfd::FileDialog::new().add_filter("Image", &["jpg", "png", "jpeg"]).pick_file() {
						if path != self.expressionPaths[index] {
							let mut uri = "file://".to_string();
							uri.push_str(path.to_str().unwrap());
							ui.ctx().forget_image(&uri);
						}

						if self.dir.is_some() {
							let relative = pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
							if relative.is_some() {
								self.expressionPaths[index] = relative.unwrap();
							}
							else {
								self.expressionPaths[index] = path;	
							}
						}
						else {
							self.expressionPaths[index] = path;
						}

						//updateValues = true;
					}

				if ui.button("x").clicked() {
					indexToRemove = Some(index);

					//updateValues = true;
				}
			});

			let collapse = egui::CollapsingHeader::new("Preview").id_salt(index).show(ui, |ui| {
				let mut p = "file://".to_string();
				if self.dir.is_some() {
				let abs = self.expressionPaths[index].absolutize_from(self.dir.as_ref().unwrap());
				if abs.is_ok() {
					p.push_str(abs.unwrap().to_str().unwrap());
				}
				else {
					p.push_str(self.expressionPaths[index].to_str().unwrap());	
				}
			}
			else {
				p.push_str(self.expressionPaths[index].to_str().unwrap());
			}
				let texture = egui::ImageSource::Uri(p.into());
				let response = ui.image(texture);
			});
			
			ui.separator();
			index += 1;
		}

		if indexToRemove.is_some() {
			let mut uri = "file://".to_string();
			uri.push_str(self.expressionPaths[indexToRemove.unwrap()].to_str().unwrap());
			ui.ctx().forget_image(&uri);

			self.expressionPaths.remove(indexToRemove.unwrap());
			//self.expressions.remove(indexToRemove.unwrap());
			app.expressions.get_mut(&self.id).unwrap().remove(indexToRemove.unwrap());

			app.deleteExpression = (self.id, indexToRemove.unwrap());
			//  TODO check for deleted expressions
		}

		if updateValues {

			let mut paths = match self.data.get_mut("paths").unwrap() {
				Value::paths(p) => { p }
				_ => { &mut vec![] }
			};
			*paths = self.expressionPaths.clone();
		}

		ui.horizontal(|ui| {
			if self.rect.is_some() {
				ui.add_space(self.rect.unwrap().width() / 2. - 40.);
			}
			if ui.button("Add Expression").clicked() {
				let val = app.expressions.get_mut(&self.id).unwrap();
				val.push(String::new());
				self.expressionPaths.push(PathBuf::new());
			}
		});
	}

	fn populateDefineSFX(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		ui.horizontal(|ui| {
			let mut ss: &mut String = &mut "".to_owned();
			match self.data.get_mut("sfx").unwrap() {
				Value::text(s) => {
					ss = s;
				}
				_ => {}
			}
			let prev = ss.to_string();
			let bg = egui::TextEdit::singleline(ss).hint_text("Sound: ").desired_width(120.);

			let response = ui.add(bg);

			if response.changed() {
				let val = app.effects.get_mut(&self.id);
				if val.is_some() {
					let val = val.unwrap();
					*val = ss.to_string()
				}
			}

			if ui.button("Select file").clicked()
				&& let Some(path) = rfd::FileDialog::new().add_filter("Sound", &["mp3", "wav"]).pick_file() {
				let data = match self.data.get_mut("path").unwrap() {
					Value::path(p) => {
						Some(p)
					}
					_ => {None }
				};
				if data.is_some() {
					println!("{:?}", path);
					if self.dir.is_some() {
						let relative =  pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
						if relative.is_some() {
							println!("Relative path constructed");
							*data.unwrap() = relative.unwrap();
						}
						else {
							*data.unwrap() = path;
						}
					}
					else {
						*data.unwrap() = path;
					}
				}
			}
		});
		

		let path = match self.data.get("path").unwrap() {
			Value::path(p) => { Some(p) }
			_ => { None }
		};

		if path.is_some() {
			let path = path.unwrap();
			if path.exists() {
				ui.label(path.file_name().unwrap().to_str().unwrap());
			}
		}
	}

	fn populateDefineMusic(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		ui.horizontal(|ui| {
			let mut ss: &mut String = &mut "".to_owned();
			match self.data.get_mut("sfx").unwrap() {
				Value::text(s) => {
					ss = s;
				}
				_ => {}
			}
			let prev = ss.to_string();
			let bg = egui::TextEdit::singleline(ss).hint_text("Music: ").desired_width(120.);

			let response = ui.add(bg);

			if response.changed() {
				let val = app.music.get_mut(&self.id);
				if val.is_some() {
					let val = val.unwrap();
					*val = ss.to_string()
				}
			}

			if ui.button("Select file").clicked()
				&& let Some(path) = rfd::FileDialog::new().add_filter("Sound", &["mp3", "wav"]).pick_file() {
				let data = match self.data.get_mut("path").unwrap() {
					Value::path(p) => {
						Some(p)
					}
					_ => {None }
				};
				if data.is_some() {
					println!("{:?}", path);
					if self.dir.is_some() {
						let relative =  pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
						if relative.is_some() {
							println!("Relative path constructed");
							*data.unwrap() = relative.unwrap();
						}
						else {
							*data.unwrap() = path;
						}
					}
					else {
						*data.unwrap() = path;
					}
				}
			}
		});
		

		let path = match self.data.get("path").unwrap() {
			Value::path(p) => { Some(p) }
			_ => { None }
		};

		if path.is_some() {
			let path = path.unwrap();
			if path.exists() {
				ui.label(path.file_name().unwrap().to_str().unwrap());
			}
		}
	}

	fn populateDefineWaitFor(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		egui::Grid::new(self.id)
			.num_columns(2)
			.show(ui, |ui|{
				let val = match self.data.get_mut("timeout").unwrap() {
					Value::float(f) => { Some(f) }
					_ => { None }
				};
				if val.is_some() {
					ui.label("Timeout:");
					let d = egui::DragValue::new(val.unwrap()).speed(0.5).range(0. ..= f64::MAX);
					ui.add(d);
				}
			});
	}

	fn populateShowSprite(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		egui::Grid::new(self.id)
			.num_columns(2)
			.show(ui, |ui| {
				// SELECT CHRACTER
				let mut currentSprite = match self.data.get("sprite").unwrap() {
					Value::id(t) => { *t }
					_ => { 0 }
				};

				let before = currentSprite.clone();
				let text = if app.sprites.contains_key(&currentSprite) { &app.sprites[&currentSprite]}
							else { "No Character!" };

				ui.label("Select Sprite:");
				let combo = egui::ComboBox::from_label("")
					.selected_text(format!("{:?}", text))
					.show_ui(ui, |ui| {
						let list = &app.sprites;
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) {
								ui.selectable_value(&mut currentSprite, *index, name);
							}
						}
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) == false {
								ui.selectable_value(&mut currentSprite, *index, name);
							}
						}
					});

				if before != currentSprite {
					let s = self.data.get_mut("sprite").unwrap();
					*s = Value::id(currentSprite);
				}
				ui.end_row();

				// SELECT EXPRESSION
				let mut currentExpression = match self.data.get("expression").unwrap() {
					Value::id(id) => { *id }
					_ => { 0 }
				};

				let before = currentExpression.clone();
				ui.label("Select expression:");

				if (app.sprites.contains_key(&currentSprite)) {
					let combo = egui::ComboBox::from_label(" ")
						.selected_text(app.expressions[&currentSprite][currentExpression].clone())
						.show_ui(ui, |ui| {
							let mut counter = 0usize;
							for expression in (&app).expressions.get(&currentSprite).unwrap() {
								ui.selectable_value(&mut currentExpression, counter, expression.to_owned());
								counter += 1;
							}
						});

					if before != currentExpression {
						let e = self.data.get_mut("expression").unwrap();
						*e = Value::id(currentExpression);
					}
				}
				
				ui.end_row();

				ui.label("Configure properties:");
				let config = match self.data.get_mut("configure").unwrap() {
					Value::flag(f) => { f }
					_ => {&mut false }
				};
				ui.checkbox(config, "");
				
				ui.end_row();
				if *config == false { return }
				// CONFIGURE PROPERTIES
				let mut properties = match self.data.get_mut("properties").unwrap() {
					Value::properties(prop) => { Some(prop) }
					_ => { None }
				};
				if properties.is_some() {
					let properties = properties.unwrap();
					let xpos = egui::Slider::new(properties.get_mut("x").unwrap(), -0.5 ..= 1.5);
					ui.label("X:");
					ui.add(xpos);
					ui.end_row();

					let ypos = egui::Slider::new(properties.get_mut("y").unwrap(), -0.5 ..= 1.5);
					ui.label("Y:");
					ui.add(ypos);
					ui.end_row();
					
					let scalex = egui::DragValue::new(properties.get_mut("scaleX").unwrap()).speed(1);
					ui.label("Scale X, %:");
					ui.add(scalex);
					ui.end_row();
					if *properties.get_mut("scaleX").unwrap() < 0. { *properties.get_mut("scaleX").unwrap() = 0. }

					let scaley = egui::DragValue::new(properties.get_mut("scaleY").unwrap()).speed(1);
					ui.label("Scale Y, %:");
					ui.add(scaley);
					ui.end_row();
					if *properties.get_mut("scaleY").unwrap() < 0. { *properties.get_mut("scaleY").unwrap() = 0. }

					let layer = egui::DragValue::new(properties.get_mut("layer").unwrap()).speed(1).range(1..=32);
					ui.label("Layer");
					ui.add(layer);
					ui.end_row();

					let alpha = egui::Slider::new(properties.get_mut("opacity").unwrap(), 0. ..= 100.);
					ui.label("Opacity:");
					ui.add(alpha);
					ui.end_row();
				}
			});
	}

	fn populateHideCharacter(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		let mut currentSprite = match self.data.get("sprite").unwrap() {
			Value::id(t) => { *t }
			_ => { 0 }
		};
		let before = currentSprite.clone();

		let text = if (app.sprites.contains_key(&currentSprite)) { &app.sprites[&currentSprite]}
					else { &"No Character!".to_owned() };

		egui::Grid::new(self.id)
			.num_columns(2)
			.show(ui, |ui| {
				ui.label("Select Character:");
				let combo = egui::ComboBox::from_label("")
					.selected_text(text)
					.show_ui(ui, |ui| {
						let list = &app.sprites;
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) {
								ui.selectable_value(&mut currentSprite, *index, name);
							}
						}
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) == false {
								ui.selectable_value(&mut currentSprite, *index, name);
							}
						}
					});
			});

		if before != currentSprite {
			let s = self.data.get_mut("sprite").unwrap();
			*s = Value::id(currentSprite);
		}
	}

	fn populateSwitchScene(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		let mut current = match self.data.get("scene").unwrap() {
			Value::id(id) => {*id}
			_ => { 0 }
		};
		let before = current.clone();
		let text = &app.scenes[current];
		ui.label("Scene");
		let combo = egui::ComboBox::from_label("")
			.selected_text(format!("{:?}", text))
			.show_ui(ui, |ui| {
				let list = &app.scenes;
				let mut index = 0usize;
				for name in list {
					ui.selectable_value(&mut current, index, name);
					index += 1;
				}
			});

		if before != current {
			let b = self.data.get_mut("scene").unwrap();
			*b = Value::id(current);
		}
	}

	fn populateVFX(&mut self, ui: &mut egui::Ui, app: &mut MyApp) {
		egui::Grid::new(self.id)
			.num_columns(2)
			.show(ui, |ui| {
				// SELECT CHRACTER
				let mut currentSprite = match self.data.get("sprite").unwrap() {
					Value::id(t) => { *t }
					_ => { 0 }
				};

				let before = currentSprite.clone();
				let text = if app.sprites.contains_key(&currentSprite) { &app.sprites[&currentSprite]}
							else { "No Character!" };

				ui.label("Select Sprite:");
				let combo = egui::ComboBox::from_label("")
					.selected_text(format!("{:?}", text))
					.show_ui(ui, |ui| {
						let list = &app.sprites;
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) {
								ui.selectable_value(&mut currentSprite, *index, name);
							}
						}
						for (index, name) in list {
							if app.nodes[app.currentScene].contains_key(index) == false {
								ui.selectable_value(&mut currentSprite, *index, name);
							}
						}
					});

				if before != currentSprite {
					let s = self.data.get_mut("sprite").unwrap();
					*s = Value::id(currentSprite);
				}
				ui.end_row();

				// CONFIGURE PROPERTIES
				let mut properties = match self.data.get_mut("properties").unwrap() {
					Value::properties(prop) => { Some(prop) }
					_ => { None }
				};
				if properties.is_some() {
					let properties = properties.unwrap();
					for (name, value) in properties {
						ui.label(name);
						let mut range = 0. ..= 10.;
						if name.contains("Target") {
							range = -1. ..= 2.;
							let slider = egui::Slider::new(value, range);
							ui.add(slider);
						}
						else if name == "Duration" {
							let d = egui::DragValue::new(value).range(0. ..= 256.).speed(0.5);
							ui.add(d);
						}
						else if name == "Alpha" {
							range = 0. ..= 1.;
							let slider = egui::Slider::new(value, range);
							ui.add(slider);
						}
						else if name == "Repeat" {
							let mut val = if *value == 1. { true } else { false };
							let check = ui.checkbox(&mut val, "");
							if check.changed() {
								if val == true {
									*value = 1.;
								}
								else {
									*value = 0.;
								}
							}
						}
						else {
							range = 0. ..= 10.;
							let slider = egui::Slider::new(value, range);
							ui.add(slider);
						}
						ui.end_row();
					}
				}
			});
	}
}