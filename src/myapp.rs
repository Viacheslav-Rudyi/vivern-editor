use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::num::FpCategory::Infinite;
use std::ops::Index;
use std::path::PathBuf;
use std::process::id;
use std::str::FromStr;
use std::{string, vec};

use eframe::egui::accesskit::HasPopup::Grid;
use eframe::egui::{self, Color32, CornerRadius, Frame, ImageSource, InputState, Pos2, ViewportBuilder, ViewportClass, ViewportId, ViewportInfo, text};
use eframe::egui::accesskit::{Point, Rect};
use eframe::epaint::RectShape;
use eframe::epaint::tessellator::Path;
use egui_extras::install_image_loaders;
use path_absolutize::Absolutize;
use pathdiff::diff_paths;
use serde::de::value::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::graphnode::Value::{destination, expressions};
use crate::graphnode::{self, GraphNode, Value};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct MyApp {
	name: String,
	pub nodes: Vec<HashMap<usize, GraphNode>>,
	sceneRect: egui::Rect,
	pub offset: egui::Vec2,
	counter: usize,
	stroke: egui::Stroke,

	pub backgrounds: HashMap<usize, String>,
	pub sprites: HashMap<usize, String>,
	pub expressions: HashMap<usize, Vec<String>>,
	pub deleteExpression: (usize, usize),
	pub effects: HashMap<usize, String>,
	pub music: HashMap<usize, String>,

	dir: Option<PathBuf>,
	pub userData: Vec<String>,
	pub currentScene: usize,
	pub scenes: Vec<String>,
	extensions: HashMap<usize, Vec<String>>,
	pub force_movement: Option<(usize, egui::Vec2)>,
	meta: MetaData,
	pub chain_delete: Option<usize>,
	pub sceneOffsets: Vec<egui::Vec2>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
struct MetaData {
	title: String,
	titleSize: usize,
	width: usize,
	height: usize,
	
	msgBoxColor: [f32; 3],
	msgBoxOpacity: usize,

	fontSize: usize,
	font: Option<PathBuf>,
	fontName: String,
	fontColor: [f32; 3],

	speakerFontSize: usize,
	speakerFont: Option<PathBuf>,
	speakerfontName: String,
	speakerFontColor: [f32; 3],

	choiceFontSize: usize,
	choiceFont: Option<PathBuf>,
	choiceFontName: String,
	choiceFontColor: [f32; 3],
	choiceBg: [f32; 3],
	choiceOpacity: usize,
}

impl Default for MyApp {
	fn default() -> Self {
		Self {
			name: "ViVerN Editor".to_owned(),
			nodes: vec![HashMap::from([(0, GraphNode::new(0, "Root".to_owned(), egui::pos2(512., 256.)))])],
			sceneRect: egui::Rect::ZERO,
			offset: egui::Vec2::ZERO,
			counter: 10,
			stroke: egui::Stroke::new(4., Color32::BLACK),
			backgrounds: HashMap::new(),
			effects: HashMap::new(),
			music: HashMap::new(),
			sprites: HashMap::new(),
			expressions: HashMap::new(),
			dir: None,
			deleteExpression: (0, 0),
			userData: vec!["user var 1".to_owned()],
			currentScene: 0,
			scenes: vec!["main".to_owned()],
			extensions: HashMap::new(),
			force_movement: None,
			meta: MetaData { title: "Your VN Title".to_owned(), titleSize: 64,
				width: (1920), height: (1080),
				msgBoxColor: ([0., 0., 0.,]), msgBoxOpacity: (70), 
				fontSize: (36), font: (None), fontColor: ([196., 196., 196.]), 
				speakerFontSize: (48), speakerFont: (None), speakerFontColor: ([0., 0., 0.]), 
				choiceFontSize: (32), choiceFont: (None), 
				choiceFontColor: ([255., 255., 255.]), choiceBg: ([0., 0., 0.]), choiceOpacity: (70),
				fontName: "garamond".to_string(), speakerfontName: "garamond".to_string(), choiceFontName: "garamond".to_string(),
			},
			chain_delete: None,
			sceneOffsets: vec![egui::Vec2::ZERO],
		}
	}
}

impl MyApp {
	fn defaultStroke() -> egui::Stroke{
		egui::Stroke::new(4., Color32::BLACK)
	}

	fn NewFile(&mut self) {
		*self = MyApp::default();
	}

	fn Save(&mut self) {
		if self.dir.as_ref().is_some() {
			let serialized = serde_json::to_string_pretty(&self);
			if serialized.is_ok() {
				let file = File::create(self.dir.clone().unwrap());
				if file.is_ok() {
					let mut file = file.unwrap();
					file.write(serialized.unwrap().as_bytes());
				}
			}
		}
		else {
			self.SaveAs();
		}
	}

	fn SaveAs(&mut self) {
		let saveTo = rfd::FileDialog::new().add_filter("Project File", &["json"]).set_directory(self.dir.clone().unwrap_or(PathBuf::from_str("/").unwrap()));
		let path = saveTo.save_file();
		if path.is_some() {
			self.dir = Some(path.clone().unwrap());
			let serialized = serde_json::to_string_pretty(&self);
			if serialized.is_ok() {
				let file = File::create(path.as_ref().unwrap());
				if file.is_ok() {
					let mut file = file.unwrap();

					self.dir = Some(path.unwrap());

					for node in self.nodes[self.currentScene].values_mut() {
						if node.title.contains("Define") {
							node.dir = self.dir.clone();
						}
						if node.title == "Define Background"
							|| node.title == "Define Music"
							|| node.title == "Define SFX" {
							let p = match node.data.get_mut("path") {
								Some(v) => {
									match v {
										Value::path(pp) => { Some(pp) }
										_ => { None }
									}	
								}
								_ => { None }
							};
							if node.dir.is_some() {
								if p.is_some() {
									let relative = pathdiff::diff_paths(p.as_ref().unwrap(), node.dir.as_ref().unwrap());
									if relative.is_some() {
										*p.unwrap() = relative.unwrap();
									}
								}
							}
						}
						if node.title == "Define Character" {
							if node.dir.is_some() {
								let rel = pathdiff::diff_paths(node.expressionPaths[0].clone(), node.dir.as_ref().unwrap());
								if rel.is_some() {
									node.expressionPaths[0] = rel.unwrap();
								}
								for exp in node.expressionPaths.iter_mut() {
									let relative = pathdiff::diff_paths(exp.clone(), node.dir.as_ref().unwrap());
									if relative.is_some() {
										*exp = relative.unwrap();
									}
								}
							}
						}
					}

					file.write(serialized.unwrap().as_bytes());
				}
			}
		}
	}
	
	fn Open(&mut self) {
		let openFrom = rfd::FileDialog::new().add_filter("Project File", &["json"]).set_directory(self.dir.clone().unwrap_or(PathBuf::from_str("/").unwrap()));
		let path = openFrom.pick_file();
		if path.as_ref().is_some() {
			let file = File::open(path.as_ref().unwrap());
			if file.is_ok() {
				let mut jstring = "".to_string();
				file.unwrap().read_to_string(&mut jstring);
				let deserialized: Option<MyApp> = serde_json::from_str(&jstring).unwrap_or(None);
				if deserialized.is_some() {
					*self = deserialized.unwrap();
					self.dir = Some(path.unwrap());

					for i in 0..self.scenes.iter().count() {
						for node in self.nodes[i].values_mut() {
							if node.title.contains("Define") {
								node.dir = self.dir.clone();
							}
						}
					}
				}
			}
		}
	}

	fn ExportWeb(&mut self, p: Option<PathBuf>) {
		if p.is_some() {
			fs::remove_dir_all(current_dir().unwrap().join("public"));
			fs::create_dir(current_dir().unwrap().join("public"));
		}
		let dest = if p.is_some() { p.clone() } else {rfd::FileDialog::new().pick_folder() };
		if dest.is_some() {
			let mut exportData = HashMap::new();
			
			let dest = dest.unwrap();
			let createAssets = fs::create_dir(dest.to_owned().join("assets"));
			fs::create_dir(dest.to_owned().join("assets").join("characters"));
			fs::create_dir(dest.to_owned().join("assets").join("backgrounds"));
			fs::create_dir(dest.to_owned().join("assets").join("music"));
			fs::create_dir(dest.to_owned().join("assets").join("sfx"));
			fs::create_dir(dest.to_owned().join("fonts"));

			let mut thingsToLoad: Vec<String> = vec![];

			for i in 0..self.scenes.iter().count() {
				for (id, node) in &self.nodes[i] {
					if node.title == "Define Character" {
						let mut ext = vec![];
						for ex in &node.expressionPaths {
							if ex.extension().is_some() {
								ext.push(ex.extension().unwrap().to_str().unwrap().to_owned());
							}
						}
						self.extensions.insert(*id, ext);
						continue;
					}
					if node.title.contains("Define") {
						let path = match node.data.get("path").unwrap() {
							Value::path(p) => { Some(p.clone()) }
							_ => { None }
						};
						if path.is_some() {
							let path = path.unwrap();
							if path.extension().is_some() {
								self.extensions.insert(*id,
									 vec![path.extension().unwrap().to_str().unwrap().to_owned()]);
							}
						}
					}
				}
			}

			for i in 0..self.scenes.iter().count() {
				let mut exportData = HashMap::new();

				for node in self.nodes[i].values() {
					let mut nodeData = node.data.clone();
					if node.title == "Show Character" {
						let sprite = nodeData.get_mut("sprite").unwrap();
						let id = match sprite {
							Value::id(id) => { *id }
							_ => { 0 }
						};
						
						if id != 0 {
							*sprite = Value::text(self.sprites[&id].clone());
						}

						let expression = nodeData.get_mut("expression").unwrap();
						let expr = match expression {
							Value::id(id) => { *id }
							_ => { 0 }
						};
						if expr != 0 {
							*expression = Value::text(self.expressions[&id][expr].clone());
						}
						else {
							*expression = Value::text("".to_owned());
						}

						if id != 0 {
							nodeData.insert("extension".to_owned(), Value::text(".".to_owned() + &self.extensions[&id][expr]));
						}
						let config = match nodeData.get_mut("configure").unwrap() {
							Value::flag(f) => { f }
							_ => {&mut false }
						};
						if *config == false {
							nodeData.remove("properties");
						}
					}
					if node.title == "Hide Character" {
						let sprite = nodeData.get_mut("sprite").unwrap();
						let id = match sprite {
							Value::id(id) => { *id }
							_ => { 0 }
						};
						
						if id != 0 {
							*sprite = Value::text(self.sprites[&id].clone());
						}
					}
					if node.title == "Set Background" {
						let sprite = nodeData.get_mut("sprite").unwrap();
						let id = match sprite {
							Value::id(id) => { *id } 
							_ => { 0 }
						};
						if id != 0 {
							*sprite = Value::text(self.backgrounds[&id].clone());
							nodeData.insert("extension".to_owned(), Value::text(".".to_owned() + &self.extensions[&id][0]));
						}
					}
					if node.title == "Play Music" || node.title == "Stop Music" {
						let music = nodeData.get_mut("sfx").unwrap();
						let id = match music {
							Value::id(id) => { *id }
							_ => { 0 }
						};
						if id != 0 {
							*music = Value::text(self.music[&id].clone());
							nodeData.insert("extension".to_owned(), Value::text(".".to_owned() + &self.extensions[&id][0]));
						}
					}
					if node.title == "Play SFX" {
						let sfx = nodeData.get_mut("sfx").unwrap();
						let id = match sfx {
							Value::id(id) => { *id }
							_ => { 0 }
						};
						if id != 0 {
							*sfx = Value::text(self.effects[&id].clone());
							nodeData.insert("extension".to_owned(), Value::text(".".to_owned() + &self.extensions[&id][0]));
						}
					}
					if node.title == "Slide"
						|| node.title == "Pulse"
						|| node.title == "Breathe"
						|| node.title == "Clear Effects" {
							let sprite = nodeData.get_mut("sprite").unwrap();
							let id = match sprite {
								Value::id(id) => { *id } 
								_ => { 0 }
							};
							if id != 0 {
								*sprite = Value::text(self.sprites[&id].clone());
							}
						}
					if node.title == "Switch Scene" {
						let scene = nodeData.get_mut("scene").unwrap();
						let id = match scene {
							Value::id(id) => { *id }
							_ => { 0 }
						};
						*scene = Value::text(self.scenes[id].clone());
					}

					if node.title.contains("Define") == false
						{
						nodeData.insert("type".to_owned(), Value::text(node.title.to_lowercase()));
						nodeData.insert("next".to_owned(), Value::next(node.next));
						nodeData.insert("id".to_owned(), Value::id(node.id));
						exportData.insert(node.id, nodeData);
					}
					
					if node.title == "Define Background" {
						let path = match node.data.get("path").unwrap() {
							Value::path(p) => { Some(p) }
							_ => { None }
						};
						let name = match node.data.get("sprite").unwrap() {
							Value::text(t) => { Some(t) }
							_ => { None }
						};

						if path.is_some() && name.is_some() {
							let mut path = path.unwrap().clone();
							let name = name.unwrap();
							let p = path.clone();
							let extension = p.extension().unwrap().to_str().unwrap();

							if node.dir.is_some() {
								let absolute =  path.absolutize_from(node.dir.as_ref().unwrap());
								if absolute.is_ok() {
									path = absolute.unwrap().to_path_buf();
								}
							}

							fs::copy(path,
								dest
								.join("assets")
								.join("backgrounds")
								.join(name)
								.with_extension(extension));

							let mut toLoad = "assets/backgrounds/".to_owned();
							toLoad.push_str(&name);
							toLoad.push('.');
							toLoad.push_str(&extension);
							thingsToLoad.push(toLoad);
						}
					}
					if node.title == "Define Character" {
						let name = match node.data.get("sprite").unwrap() {
							Value::text(t) => { Some(t) }
							_ => { None }
						};
						if name.is_some() {
							let name = name.unwrap();
							let extension = node.expressionPaths[0].extension().unwrap();
							let mut path = node.expressionPaths[0].clone();
							if node.dir.is_some() {
								let absolute = path.absolutize_from(node.dir.as_ref().unwrap());
								if absolute.is_ok() {
									path = absolute.unwrap().to_path_buf();
								}
							}
							fs::copy(path,
								dest
								.join("assets")
								.join("characters")
								.join(name.to_owned())
								.with_extension(extension));

							let mut toLoad = "assets/characters/".to_owned();
							toLoad.push_str(&name);
							toLoad.push('.');
							toLoad.push_str(extension.to_str().unwrap());
							thingsToLoad.push(toLoad);

							for i in 1..node.expressionPaths.iter().count() {
								let extension = node.expressionPaths[i].extension().unwrap();
								let mut path = node.expressionPaths[i].clone();
								if node.dir.is_some() {
									let absolute = path.absolutize_from(node.dir.as_ref().unwrap());
									if absolute.is_ok() {
										path = absolute.unwrap().to_path_buf();
									}
								}
								let expressionName = self.expressions[&node.id][i].clone();

								fs::copy(path,
									dest
									.join("assets")
									.join("characters")
									.join(name.to_owned() + "_" + &expressionName)
									.with_extension(extension));

								let mut toLoad = "assets/characters/".to_owned();
								toLoad.push_str(&name);
								toLoad.push('_');
								toLoad.push_str(&expressionName);
								toLoad.push('.');
								toLoad.push_str(extension.to_str().unwrap());
								thingsToLoad.push(toLoad);
							}
						}
					}
					if node.title == "Define SFX" {
						let path = match node.data.get("path").unwrap() {
							Value::path(p) => { Some(p) }
							_ => { None }
						};
						let name = match node.data.get("sfx").unwrap() {
							Value::text(t) => { Some(t) }
							_ => { None }
						};

						if path.is_some() && name.is_some() {
							let mut path = path.unwrap().clone();
							let name = name.unwrap();
							let p = path.clone();
							let extension = p.extension().unwrap().to_str().unwrap();

							if node.dir.is_some() {
								let absolute =  path.absolutize_from(node.dir.as_ref().unwrap());
								if absolute.is_ok() {
									path = absolute.unwrap().to_path_buf();
								}
							}

							fs::copy(path,
								dest
								.join("assets")
								.join("sfx")
								.join(name)
								.with_extension(extension));

							let mut toLoad = "assets/sfx/".to_owned();
							toLoad.push_str(&name);
							toLoad.push('.');
							toLoad.push_str(&extension);
							thingsToLoad.push(toLoad);
						}
					}
					if node.title == "Define Music" {
						let path = match node.data.get("path").unwrap() {
							Value::path(p) => { Some(p) }
							_ => { None }
						};
						let name = match node.data.get("sfx").unwrap() {
							Value::text(t) => { Some(t) }
							_ => { None }
						};

						if path.is_some() && name.is_some() {
							let mut path = path.unwrap().clone();
							let name = name.unwrap();
							let p = path.clone();
							let extension = p.extension().unwrap().to_str().unwrap();

							if node.dir.is_some() {
								let absolute =  path.absolutize_from(node.dir.as_ref().unwrap());
								if absolute.is_ok() {
									path = absolute.unwrap().to_path_buf();
								}
							}

							fs::copy(path,
								dest
								.join("assets")
								.join("music")
								.join(name)
								.with_extension(extension));

							let mut toLoad = "assets/music/".to_owned();
							toLoad.push_str(&name);
							toLoad.push('.');
							toLoad.push_str(&extension);
							thingsToLoad.push(toLoad);
						}

					}
				}

				let serialize = serde_json::to_string_pretty(&exportData);
				if serialize.is_ok() {
					fs::create_dir(dest.to_owned().join("scenes"));
					let file = File::create(dest.to_owned().join("scenes").join(self.scenes[i].clone()).with_extension("json"));
					if file.is_ok() {
						file.unwrap().write(serialize.unwrap().as_bytes());
					}
				}
			}

			// SAVE FONTS
			for font in [& self.meta.font, & self.meta.speakerFont, & self.meta.choiceFont] {
				if font.is_none() { continue }
				let mut path = font.clone().unwrap();
					
				let p = path.clone();
				let name = p.file_stem().unwrap().to_str().unwrap();
				let extension = p.extension().unwrap().to_str().unwrap();

				if self.dir.is_some() {
					let absolute =  path.absolutize_from(self.dir.as_ref().unwrap());
					if absolute.is_ok() {
						path = absolute.unwrap().to_path_buf();
					}
				}

				fs::copy(path,
					dest
					.join("fonts")
					.join(name)
					.with_extension(extension));

					let mut toLoad = "fonts/".to_owned();
						toLoad.push_str(&name);
						toLoad.push('.');
						toLoad.push_str(&extension);
						thingsToLoad.push(toLoad);
			}

			for node in self.nodes[self.currentScene].values() {
				if node.title.contains("Define") == false {
					let mut nodeData = node.data.clone();

					nodeData.insert("type".to_owned(), Value::text(node.title.to_lowercase()));
					nodeData.insert("next".to_owned(), Value::next(node.next));
					exportData.insert(node.id, nodeData);
				}
				else {
					continue;
				}
			}

			let current = current_dir().unwrap().join("WEB_TEMPLATE");
			fs::create_dir(dest.to_owned().join("hexi"));	
			
			let index = fs::copy(current.join("index").with_extension("html"), dest.join("index").with_extension("html"));
			let style = fs::copy(current.join("style").with_extension("css"), dest.join("style").with_extension("css"));
			let hexi = fs::copy(current.join("hexi").join("hexi").with_extension("js"), 
				dest.join("hexi").join("hexi").with_extension("js"));

			let script = fs::read_to_string(current.join("script").with_extension("js"));
			let mut stringToLoad = "let thingsToLoad = [".to_owned();
			for thing in thingsToLoad {
				stringToLoad.push('\"');
				stringToLoad.push_str(&thing);
				stringToLoad.push_str("\", ");
			}
			let mut counter = 0usize;
			for scene in &self.scenes {
				if counter == 0 {
					counter += 1;
					continue;
				}
				stringToLoad.push('\"');
				stringToLoad.push_str("scenes/");
				stringToLoad.push_str(scene);
				stringToLoad.push_str(".json");
				stringToLoad.push_str("\", ");
			}
			stringToLoad.push_str("\"scenes/main.json\"]\n\n");

			if self.meta.font.is_some() {
				self.meta.fontName = self.meta.font.as_ref().unwrap().file_stem().unwrap().to_str().unwrap().to_owned();
			}
			else {
				self.meta.fontName = "garamond".to_owned();
			}

			if self.meta.speakerFont.is_some() {
				self.meta.speakerfontName = self.meta.speakerFont.as_ref().unwrap().file_stem().unwrap().to_str().unwrap().to_owned();
			}
			else {
				self.meta.speakerfontName = "garamond".to_owned();
			}

			if self.meta.choiceFont.is_some() {
				self.meta.choiceFontName = self.meta.choiceFont.as_ref().unwrap().file_stem().unwrap().to_str().unwrap().to_owned();
			}
			else {
				self.meta.choiceFontName = "garamond".to_owned();
			}
			
			let mut metaClone = self.meta.clone();
			metaClone.choiceFont = None;
			metaClone.font = None;
			metaClone.speakerFont = None;

			let meta = serde_json::to_string_pretty(&metaClone);
			if meta.is_ok() {
				stringToLoad.push_str("let metadataString = `\n");
				stringToLoad.push_str(meta.as_ref().unwrap());
				stringToLoad.push_str("\n`\n");
				let file = File::create(dest.to_owned().join("metadata").with_extension("json"));
				if file.is_ok() {
					file.unwrap().write(meta.as_ref().unwrap().as_bytes());
				}
			}

			if script.is_ok() {
				stringToLoad.push_str(&script.unwrap());
			}

			fs::write(dest.join("script").with_extension("js"), stringToLoad);

			if p.is_none() { open::that(dest); }
		}
	}

	fn ExportAsScenes(&self) {
		let dest = rfd::FileDialog::new().pick_folder();
		if dest.is_some() {
			let dest = dest.unwrap();
			for i in 0..self.scenes.iter().count() {
				let mut exportData = HashMap::new();
				for node in self.nodes[i].values() {
					if node.title.contains("Define") { continue }
					let mut nodeData = node.data.clone();

					nodeData.insert("type".to_owned(), Value::text(node.title.to_lowercase()));
					nodeData.insert("next".to_owned(), Value::next(node.next));
					exportData.insert(node.id, nodeData);
				}

				let serialize = serde_json::to_string_pretty(&exportData);
				if serialize.is_ok() {
					let file = File::create(dest.to_owned().join(&self.scenes[i]).with_extension("json"));
					if file.is_ok() {
						file.unwrap().write(serialize.unwrap().as_bytes());
					}
				}
			}

			open::that(dest);
		}
	}

	fn ExportCurrentScene(&self) {
		let dest = rfd::FileDialog::new().pick_folder();
		if dest.is_some() {
			let dest = dest.unwrap();
			let mut exportData = HashMap::new();

			for node in self.nodes[self.currentScene].values() {
					if node.title.contains("Define") { continue }
					let mut nodeData = node.data.clone();

					nodeData.insert("type".to_owned(), Value::text(node.title.to_lowercase()));
					nodeData.insert("next".to_owned(), Value::next(node.next));
					exportData.insert(node.id, nodeData);
				
			}

			let serialize = serde_json::to_string_pretty(&exportData);
			if serialize.is_ok() {
				let file = File::create(dest.to_owned().join(&self.scenes[self.currentScene]).with_extension("json"));
				if file.is_ok() {
					file.unwrap().write(serialize.unwrap().as_bytes());
				}
			}

			open::that(dest);
		}	
	}

	fn ContextMenu(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
		let pos = ctx.pointer_latest_pos().unwrap_or(egui::Pos2::ZERO) - self.offset;
				ui.menu_button("Dialogue Flow", |ui| {
					if ui.button("Dialogue").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Dialogue".to_owned(), pos);
						
						node.data.insert("speaker".to_owned(), Value::text(String::new()));
						node.data.insert("text".to_owned(), Value::dialogue(vec![String::new()]));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Choice").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Choice".to_owned(), pos);

						node.data.insert("choices".to_owned(), Value::choice(vec![String::new()]));
						node.data.insert("destinations".to_owned(), Value::destination(vec![None]));

						self.nodes[self.currentScene].insert(self.counter, node);
					}

					if ui.button("Hide Messagebox").clicked() {
						self.counter += 1;
						let node = GraphNode::new(self.counter, "Hide Messagebox".to_owned(), pos);
						self.nodes[self.currentScene].insert(self.counter, node);
					}
				});
				ui.menu_button("Sprite Flow", |ui| {
					if ui.button("Show Character").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Show Character".to_owned(), pos);

						node.data.insert("sprite".to_owned(), Value::id(0));
						node.data.insert("expression".to_owned(), Value::id(0));
						node.data.insert("properties".to_owned(), Value::properties(HashMap::from([
							("x".to_owned(), 0.5),
							("y".to_owned(), 1.),
							("scaleX".to_owned(), 100.),
							("scaleY".to_owned(), 100.),
							("layer".to_owned(), 1.),
							("opacity".to_owned(), 100.)
						])));
						node.data.insert("configure".to_owned(), Value::flag(false));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Hide Character").clicked() {
						self.counter += 1;

						let mut node = GraphNode::new(self.counter, "Hide Character".to_owned(), pos);
						node.data.insert("sprite".to_owned(), Value::text("".to_owned()));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Clear").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Clear".to_owned(), pos);
						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Set Background").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Set Background".to_owned(), pos);

						node.data.insert("sprite".to_owned(), Value::id(0));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Background Color").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Set Background Color".to_owned(), pos);
						node.data.insert("color".to_owned(), Value::rgb([0., 0., 0.]));
						self.nodes[self.currentScene].insert(self.counter, node);
					}
				});
				ui.menu_button("Sprite Tweens", |ui| {
					if ui.button("Slide").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Slide".to_owned(), pos);

						node.data.insert("sprite".to_owned(), Value::id(0));
						node.data.insert("properties".to_owned(), Value::properties(HashMap::from([
							("Target X".to_owned(), 0.5),
							("Target Y".to_owned(), 1.),
							("Duration".to_owned(), 1.),
							("Repeat".to_owned(), 1.),
						])));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Pulse").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Pulse".to_owned(), pos);
						node.data.insert("sprite".to_owned(), Value::id(0));
						node.data.insert("properties".to_owned(), Value::properties(HashMap::from([
							("Duration".to_owned(), 1.),
							("Alpha".to_owned(), 0.1),
							("Repeat".to_owned(), 1.),
						])));
						self.nodes[self.currentScene].insert(self.counter, node);
						
					}
					if ui.button("Breathe").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Breathe".to_owned(), pos);
						node.data.insert("sprite".to_owned(), Value::id(0));
						node.data.insert("properties".to_owned(), Value::properties(HashMap::from([
							("Final X".to_owned(), 1.4),
							("Final Y".to_owned(), 1.4),
							("Duration".to_owned(), 1.),
							("Repeat".to_owned(), 1.),
						])));
						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Clear Effects").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Clear Effects".to_owned(), pos);
						node.data.insert("sprite".to_owned(), Value::id(0));
						node.data.insert("properties".to_owned(), Value::properties(HashMap::new()));
						self.nodes[self.currentScene].insert(self.counter, node);
					}
				});
				ui.menu_button("Sound", |ui| {
					if ui.button("Play music").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Play Music".to_owned(), pos);

						node.data.insert("sfx".to_owned(), Value::id(0));
						node.data.insert("volume".to_owned(), Value::float(1.0));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Stop music").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Stop Music".to_owned(), pos);

						node.data.insert("sfx".to_owned(), Value::id(0));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Play SFX").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Play SFX".to_owned(), pos);

						node.data.insert("sfx".to_owned(), Value::id(0));
						node.data.insert("volume".to_owned(), Value::float(1.0));

						self.nodes[self.currentScene].insert(self.counter, node);
					}
				});
				ui.menu_button("Navigation", |ui| {
					if ui.button("Wait").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Wait".to_owned(), pos);
						self.nodes[self.currentScene].insert(self.counter, node);
					}

					if ui.button("Wait For").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Wait For".to_owned(), pos);
						node.data.insert("timeout".to_owned(), Value::float(1.0));
						self.nodes[self.currentScene].insert(self.counter, node);
					}

					if ui.button("Switch Scene").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Switch Scene".to_owned(), pos);
						node.data.insert("scene".to_owned(), Value::id(0));
						node.next = None;
						self.nodes[self.currentScene].insert(self.counter, node);
					}
				});
				ui.menu_button("Define", |ui| {
					if ui.button("Define Character").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Define Character".to_owned(), pos);

						node.data.insert("sprite".to_owned(), Value::text(format!("Character {}", self.counter).to_owned()));
						//node.data.insert("expressions".to_owned(), Value::expressions(vec![]));
						node.data.insert("paths".to_owned(), Value::paths(vec![]));

						node.dir = self.dir.clone();
						node.next = None;

						self.sprites.insert(self.counter, format!("Character {}", self.counter).to_owned());
						self.expressions.insert(self.counter, vec!["Default".to_owned()]);

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Define Background").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Define Background".to_owned(), pos);

						node.data.insert("sprite".to_owned(), Value::text(format!("Background {}", self.counter).to_owned()));
						node.data.insert("path".to_owned(), Value::path(PathBuf::new()));

						node.next = None;
						self.backgrounds.insert(self.counter, format!("Background {}", self.counter).to_owned());

						node.dir = self.dir.clone();

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Define SFX").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Define SFX".to_owned(), pos);

						node.data.insert("sfx".to_owned(), Value::text(format!("SFX {}", self.counter).to_owned()));
						node.data.insert("path".to_owned(), Value::path(PathBuf::new()));

						node.next = None;
						self.effects.insert(self.counter, format!("SFX {}", self.counter).to_owned());

						node.dir = self.dir.clone();

						self.nodes[self.currentScene].insert(self.counter, node);
					}
					if ui.button("Define Music").clicked() {
						self.counter += 1;
						let mut node = GraphNode::new(self.counter, "Define Music".to_owned(), pos);

						node.data.insert("sfx".to_owned(), Value::text(format!("Music {}", self.counter).to_owned()));
						node.data.insert("path".to_owned(), Value::path(PathBuf::new()));

						node.next = None;
						self.music.insert(self.counter, format!("Music {}", self.counter).to_owned());

						node.dir = self.dir.clone();

						self.nodes[self.currentScene].insert(self.counter, node);
					}
				});

				
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		install_image_loaders(ctx);
		self.force_movement = None;
		self.chain_delete = None;

		let topPanel = egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			ui.add_space(5.);
			ui.horizontal(|ui| {
				ui.menu_button("File", |ui| {
					if ui.button("New").clicked() {
						self.NewFile();
					}
					if ui.button("Open").clicked() {
						self.Open();
					}
					if ui.button("Save").clicked() {
						self.Save();
					}
					if ui.button("Save As").clicked() {
						self.SaveAs();
					}
				});
				ui.menu_button("Export", |ui| {
					if ui.button("Export to Web").clicked() {
						self.ExportWeb(None);
					}
					if ui.button("Export As Scenes").clicked() {
						self.ExportAsScenes();
					}
					if ui.button("Export Current Scene").clicked() {
						self.ExportCurrentScene();
					}
				});
				if ui.button("Run").clicked() {
					self.ExportWeb(Some(current_dir().unwrap().join("public")));
					
					open::that("http://localhost:7979/index.html");
				}
			});
			ui.add_space(5.);
		}).response;
		let leftPanel = egui::SidePanel::left("scene_control_panel")
		.frame(Frame::new().fill(Color32::LIGHT_GRAY))
		.resizable(true)
		.show(ctx, |ui| {
			ui.collapsing("Custom Settings", |ui| {
				ui.label("Title");
				let tt = egui::TextEdit::singleline(&mut self.meta.title).desired_width(196.);
				ui.add(tt);
				egui::Grid::new("Title size")
					.num_columns(2)
					.show(ui, |ui| {
						ui.label("Title font size");
						let op = egui::DragValue::new(&mut self.meta.fontSize).speed(1).range(0..=196);
						ui.add(op);
						ui.end_row();
					});
				ui.separator();
				ui.label("Window");
				egui::Grid::new(101)
				.num_columns(2)
				.show(ui, |ui| {

					let mut widthBuf = self.meta.width.to_string();
					let wwidth = egui::TextEdit::singleline(&mut widthBuf).hint_text("width").desired_width(96.);
					let mut heightBuf = self.meta.height.to_string();
					let wheight = egui::TextEdit::singleline(&mut heightBuf).hint_text("height").desired_width(96.);
				
					ui.label("W: ");
					let wwidth = ui.add(wwidth);
					if wwidth.changed() {
						if widthBuf.parse::<usize>().is_ok() {
							self.meta.width = widthBuf.parse::<usize>().ok().unwrap();
						}
						if widthBuf.is_empty() {
							self.meta.width = 0;
						}
					}

					ui.end_row();
					ui.label("H: ");
					let wheight = ui.add(wheight);
				
					if wwidth.changed() || wheight.changed() {
						if widthBuf.parse::<usize>().is_ok() {
							self.meta.width = widthBuf.parse::<usize>().ok().unwrap();
						}
						if widthBuf.is_empty() {
							self.meta.width = 0;
						}

						if heightBuf.parse::<usize>().is_ok() {
							self.meta.height = heightBuf.parse::<usize>().ok().unwrap();
						}
						if heightBuf.is_empty() {
							self.meta.height = 0;
						}
					}

				});

				ui.separator();
				ui.label("Message Box");
				egui::Grid::new(11)
					.num_columns(2)
					.show(ui, |ui| {
						ui.label("Color");
						egui::color_picker::color_edit_button_rgb(ui, &mut self.meta.msgBoxColor);

						ui.end_row();
						ui.label("Opacity");
						let op = egui::DragValue::new(&mut self.meta.msgBoxOpacity).speed(1).range(0..=100);
						ui.add(op);
						ui.end_row();
					});

				ui.separator();
				ui.label("Dialogue");
				egui::Grid::new(12)
					.num_columns(2)
					.show(ui, |ui| {
						ui.label("Font color");
						egui::color_picker::color_edit_button_rgb(ui, &mut self.meta.fontColor);
						ui.end_row();

						ui.label("Font size");
						let op = egui::DragValue::new(&mut self.meta.fontSize).speed(1).range(0..=(usize::MAX - 1));
						ui.add(op);
						ui.end_row();

						ui.label("Font");
						if ui.button("Select").clicked()
							&& let Some(path) = rfd::FileDialog::new().add_filter("font", &["ttf"]).pick_file() {
								if self.dir.is_some() {
									let relative = pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
									if relative.is_some() {
										self.meta.font = Some(relative.unwrap());
									}
									else {
										self.meta.font = Some(path);
									}
								}	
								else {
									self.meta.font = Some(path);
								}
							}
						ui.end_row();
						if self.meta.font.is_some() {
							let f = self.meta.font.clone();
							ui.label(f.unwrap().file_name().unwrap().to_str().unwrap());
							if ui.button("Clear").clicked() {
								self.meta.font = None;
							}
						}
						else {
							ui.label("Not font selected");
						}
						ui.end_row();
					});

					ui.separator();
				ui.label("Speaker");
				egui::Grid::new(13)
					.num_columns(2)
					.show(ui, |ui| {
						ui.label("Font color");
						egui::color_picker::color_edit_button_rgb(ui, &mut self.meta.speakerFontColor);
						ui.end_row();

						ui.label("Font size");
						let op = egui::DragValue::new(&mut self.meta.speakerFontSize).speed(1).range(0..=(usize::MAX - 1));
						ui.add(op);
						ui.end_row();

						ui.label("Font");
						if ui.button("Select").clicked()
							&& let Some(path) = rfd::FileDialog::new().add_filter("font", &["ttf"]).pick_file() {
								if self.dir.is_some() {
									let relative = pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
									if relative.is_some() {
										self.meta.speakerFont = Some(relative.unwrap());
									}
									else {
										self.meta.speakerFont = Some(path);
									}
								}	
								else {
									self.meta.speakerFont = Some(path);
								}
							}
						ui.end_row();
						if self.meta.speakerFont.is_some() {
							let f = self.meta.speakerFont.clone();
							ui.label(f.unwrap().file_name().unwrap().to_str().unwrap());
							if ui.button("Clear").clicked() {
								self.meta.font = None;
							}
						}
						else {
							ui.label("Not font selected");
						}
						ui.end_row();
					});

					ui.separator();

					ui.label("Choice box");
				egui::Grid::new(14)
					.num_columns(2)
					.show(ui, |ui| {
						ui.label("Font color");
						egui::color_picker::color_edit_button_rgb(ui, &mut self.meta.choiceFontColor);
						ui.end_row();

						ui.label("Font size");
						let op = egui::DragValue::new(&mut self.meta.choiceFontSize).speed(1).range(0..=(usize::MAX - 1));
						ui.add(op);
						ui.end_row();

						ui.label("Font");
						if ui.button("Select").clicked()
							&& let Some(path) = rfd::FileDialog::new().add_filter("font", &["ttf"]).pick_file() {
								if self.dir.is_some() {
									let relative = pathdiff::diff_paths(path.clone(), self.dir.as_ref().unwrap());
									if relative.is_some() {
										self.meta.choiceFont = Some(relative.unwrap());
									}
									else {
										self.meta.choiceFont = Some(path);
									}
								}
								else {
									self.meta.choiceFont = Some(path);
								}
							}
						ui.end_row();
						if self.meta.choiceFont.is_some() {
							let f = self.meta.choiceFont.clone();
							ui.label(f.unwrap().file_name().unwrap().to_str().unwrap());
							if ui.button("Clear").clicked() {
								self.meta.font = None;
							}
						}
						else {
							ui.label("Not font selected");
						}
						ui.end_row();
						ui.end_row();

						ui.label("Background color");
						egui::color_picker::color_edit_button_rgb(ui, &mut self.meta.choiceBg);
						ui.end_row();

						ui.label("Backgroud Opacity");
						let op = egui::DragValue::new(&mut self.meta.choiceOpacity).speed(1).range(0..=100);
						ui.add(op);
						ui.end_row();

					});
			});
			ui.separator();
			ui.label("Scene manager");
			ui.horizontal(|ui| {
				ui.label("main");
					if self.currentScene != 0 {
						if ui.button("open").clicked() {
							self.currentScene = 0;
							self.offset = self.sceneOffsets[0];
						}
					}
			});
			let mut counter = 0usize;
			let mut queue_delete = 0usize;
			for scene in &mut self.scenes {
				if counter == 0 { counter += 1; continue; }

				let edit = egui::TextEdit::singleline(scene).desired_width(96.);
				ui.horizontal(|ui| {
					ui.add(edit);
					if counter == self.currentScene {
						if ui.button("____").clicked() {
							self.currentScene = counter;
							self.offset = self.sceneOffsets[counter];
						}
					}
					else {
						if ui.button("open").clicked() {
							self.currentScene = counter;
							self.offset = self.sceneOffsets[counter];
						}
					}
					if ui.button("del").clicked() {
						queue_delete = counter;
						for i in &mut self.nodes {
							for (id, node) in i {
								if node.title == "Switch Scene" {
									let mut current = match node.data.get_mut("scene").unwrap() {
										Value::id(id) => {id}
										_ => { &mut 0 }
									};
									if *current == queue_delete {
										*current = 0;
									}
									else if *current > queue_delete {
										*current -= 1;
									}
								}
							}
						}
					}
				});
				counter += 1;
			}

			if queue_delete != 0 {
				self.scenes.remove(queue_delete);
			} 
			
			if ui.button("new scene").clicked() {
				self.scenes.push(String::new());
				self.nodes.push(
					HashMap::from(
						[(0, GraphNode::new(0, "Root".to_owned(),
						egui::pos2(512., 256.)))]
					));
				self.sceneOffsets.push(egui::Vec2::ZERO);
			}
		}).response;
		
		//let central = egui::CentralPanel::default().show(ctx, |ui| {}).response;

		let viewportId = ViewportId::from_hash_of("central");

		// let viewport = ctx.show_viewport_immediate(
		// 	viewportId,
		// 	ViewportBuilder::default()
		// 		.with_close_button(false)
		// 		.with_transparent(true)
		// 		.with_maximize_button(false)
		// 		.with_minimize_button(false)
		// 		.with_taskbar(false)
		// 		.with_inner_size(egui::Vec2::new(central.rect.width(), central.rect.height()))
		// 		.with_position(egui::Pos2::new(leftPanel.rect.width(), topPanel.rect.height()))
		// 		.with_resizable(false)
		// 		//.with_always_on_top()
		// 		.with_taskbar(false)
		// 		, |ctx, _| {

		egui::CentralPanel::default().show(ctx, |ui| {
			let scene = egui::Scene::new()
					.max_inner_size([ctx.content_rect().width(), ctx.content_rect().height()])
					//.max_inner_size([320., 100.])
					.zoom_range(1.0..=1.0);
					
			let mut innerRect = egui::Rect::NAN;
				
			let mut segments: Vec<egui::Shape> = vec![];
			let mut rect = self.sceneRect.clone();

			let response = scene.show(
					ui, 
					&mut rect,
					|ui| {
						let mut removeNodeId = None;

						let mut nodes = std::mem::take(&mut self.nodes[self.currentScene]);

						// DISPLAY NODES
						for node in nodes.values_mut() {
							if self.deleteExpression.0 != 0 && node.title == "Show Character" {
								let characterIndex = match node.data.get("sprite").unwrap() {
									Value::id(id) => { *id }
									_ => { 0 }
								};
								let mut expressionIndex = match node.data.get_mut("expression").unwrap() {
									Value::id(id) => { id }
									_ => { &mut 0 }
								};

								if characterIndex == self.deleteExpression.0 {
									if *expressionIndex == self.deleteExpression.1 {
										*expressionIndex = 0;
									}
									else if *expressionIndex > self.deleteExpression.1 {
										*expressionIndex -= 1;
									}
								}
							}
							if node.queueDelete == true {
								if node.title == "Define Background" {
									self.backgrounds.remove(&node.id);
								}

								if node.title == "Define Character" {
									self.sprites.remove(&node.id);
									self.expressions.remove(&node.id);
								}

								removeNodeId = Some(node.id);
							}

							if ((node.pos.x + self.offset.x) > leftPanel.rect.width() / 2.) {
									node.visible = true;
									node.display(ui, ctx, None, self);
								}
							else { 
								node.visible = false;
								// node.visible = true;
								// node.display(ui, ctx, None, self); 
							}
						}
						
						// REMOVE NODE
						self.deleteExpression = (0, 0);
						if removeNodeId.is_some() {
							nodes.remove(&removeNodeId.unwrap());

							for node in nodes.values_mut() {
								if node.title == "Choice" {
									let ids: &mut Vec<Option<usize>> = match node.data.get_mut("destinations").unwrap() {
										Value::destination(d) => { d }
										_ => { &mut vec![] }
									};
									let mut counter = 0;

									for id in ids.iter() {
										if id.is_some() && id.unwrap() == removeNodeId.unwrap() {
											break;
										}
										counter += 1;
									}

									if counter < ids.len() {
										ids[counter] = None;
									}
								}
								else {
									if node.next.is_some() && node.next.unwrap() == removeNodeId.unwrap() {
										node.next = None;
									}
								}
							}
						}

						// DISPLAY CONNECTIONS
						for node in nodes.values() {
							if node.next.is_some() {
								let next_id = node.next.unwrap();
								let mut fromPos = node.outPos;
								if node.visible == false {
									fromPos = node.pos + self.offset;
								}

								let target = nodes.get(&next_id).unwrap();
								let mut toPos = target.inputPos;
								if target.visible == false {
									toPos = target.pos + self.offset;
								}
								segments.push(egui::Shape::line_segment([fromPos, toPos], MyApp::defaultStroke()));
							}

							if node.title == "Choice" {
								let ids = match node.data.get("destinations").unwrap() {
									Value::destination(d) => { d }
									_ => { & vec![] }
								};
								let mut counter = 0;

								for id in ids {
									if id.is_some() {
										if node.visible == true {
											segments.push(egui::Shape::line_segment([node.choicePos[counter], nodes.get(&id.unwrap()).unwrap().inputPos], MyApp::defaultStroke()));
										}
										else {
											segments.push(egui::Shape::line_segment([node.pos + self.offset, nodes.get(&id.unwrap()).unwrap().inputPos], MyApp::defaultStroke()));
										}
										
									}
									counter += 1;
								}
							}
						}

						// DRAW TEMPORARY CONNECTION
						if ui.input(|input| {
							input.pointer.primary_down()
						}) {
							let mut from = None;
							for node in nodes.values() {
								if node.selectedAt.is_some() {
									from = Some(node);
									break;
								}
							}
							if from.is_some() {
								let to = ctx.pointer_latest_pos();
								if to.is_some() {
									segments.push(egui::Shape::line_segment([from.unwrap().selectedAt.unwrap(), to.unwrap()], MyApp::defaultStroke()));
								}
							}
							// segments.push(egui::Shape::line_segment([egui::Pos2::ZERO, ctx.pointer_latest_pos().unwrap()], MyApp::defaultStroke()));
						}

						if ui.input(|input| {
							input.pointer.primary_released()
						}) {
							let mut from = None;
							let mut to = None;

							for node in nodes.values_mut() {
								if node.selectedAt.is_some() {
									from = Some(node);
								}
								else if node.releasedAt.is_some() {
									to = Some(node);
								}

								if from.is_some() && to.is_some() { break }
							}

							if from.is_some() && to.is_some() {
								if from.as_ref().unwrap().title == "Choice" {
									let index = from.as_ref().unwrap().fromChoice;
									let a = match from.as_mut().unwrap().data.get_mut("destinations").unwrap() {
										Value::destination(d) => {
											d
										}
										_ => { &mut vec![] }
									};
									if index.is_some() {
										a[index.unwrap()] = Some(to.as_ref().unwrap().id);
									}
								}
								else {
									from.as_mut().unwrap().next = Some(to.as_mut().unwrap().id);
								}
								from.unwrap().selectedAt = None;
								to.unwrap().releasedAt = None;
							}
							else if from.is_some() {
								from.unwrap().selectedAt = None;
							}
						}

						innerRect = ui.min_rect();

						self.nodes[self.currentScene] = nodes;
					}).response;
					
			response.context_menu(|ui| {
				self.ContextMenu(ctx, ui);
			});

			if response.dragged() {
				self.offset += response.drag_motion();
			}
				
			let painter = ui.painter();

			for segment in segments {
				painter.add(segment);
			}
			
			self.sceneRect = rect;
		});

		//});

		// let commands = [
		// 	// egui::ViewportCommand::OuterPosition(Pos2::new(
		// 	// pos.x + leftPanel.rect.width(),
		// 	// pos.y + topPanel.rect.height()
		// 	// )),
		// 	egui::ViewportCommand::Decorations(false),
		// ];
		// for cmd in commands {
		// 	ctx.send_viewport_cmd_to(viewportId, cmd);
		// }

		
		if self.force_movement.is_some() {
			let mut force_move = vec![self.force_movement.unwrap().0];
			let mut next_pass = force_move.clone();
			while(next_pass.is_empty() == false) {
				next_pass.clear();
				for id in force_move.iter() {
					if self.nodes[self.currentScene][&id].next.is_some() {
						let target = self.nodes[self.currentScene][&id].next.unwrap();
						if force_move.contains(&target) == false
						&& next_pass.contains(&target) == false {
							next_pass.push(target);
						}
					}
					if self.nodes[self.currentScene][&id].title == "Choice" {
						let node = &self.nodes[self.currentScene][&id];
						let ids = match node.data.get("destinations").unwrap() {
							Value::destination(d) => { d }
							_ => { &vec![] }
						};
						for id in ids.iter() {
							if id.is_some()
							&& force_move.contains(&id.unwrap()) == false
							&& next_pass.contains(&id.unwrap()) == false {
								next_pass.push(id.unwrap());
							}
						}
					}
				}
				for i in next_pass.iter() {
					force_move.push(*i);
				}
			}
			let drag = self.force_movement.unwrap().1;
			for id in force_move {
				let nodes = self.nodes.get_mut(self.currentScene).unwrap();
				nodes.get_mut(&id).unwrap().pos += drag;
			}

		}

		if self.chain_delete.is_some() {
			let mut force_move = vec![self.chain_delete.unwrap()];
			let mut next_pass = force_move.clone();
			while(next_pass.is_empty() == false) {
				next_pass.clear();
				for id in force_move.iter() {
					if self.nodes[self.currentScene][&id].next.is_some() {
						let target = self.nodes[self.currentScene][&id].next.unwrap();
						if force_move.contains(&target) == false
						&& next_pass.contains(&target) == false {
							next_pass.push(target);
						}
					}
					if self.nodes[self.currentScene][&id].title == "Choice" {
						let node = &self.nodes[self.currentScene][&id];
						let ids = match node.data.get("destinations").unwrap() {
							Value::destination(d) => { d }
							_ => { &vec![] }
						};
						for id in ids.iter() {
							if id.is_some()
							&& force_move.contains(&id.unwrap()) == false
							&& next_pass.contains(&id.unwrap()) == false {
								next_pass.push(id.unwrap());
							}
						}
					}
				}
				for i in next_pass.iter() {
					force_move.push(*i);
				}
			}
			let drag = self.chain_delete.unwrap();
			for id in force_move {
				let nodes = self.nodes.get_mut(self.currentScene).unwrap();
				let node = nodes.get_mut(&id).unwrap();
				if node.title != "Root" {
					node.queueDelete = true;
				}
			}
		}	

		ctx.input(|i| {
			if i.modifiers.command && i.modifiers.shift {
				if i.key_released(egui::Key::S) {
					self.SaveAs();
				}
			}
			else if i.modifiers.command {
				if i.key_released(egui::Key::S) {
					self.Save();
				}
				else if i.key_released(egui::Key::O) {
					self.Open();
				}
				else if i.key_pressed(egui::Key::E) {
					self.ExportWeb(None);
				}
				else if i.key_pressed(egui::Key::N) {
					self.NewFile();
				}
			}
			else {
				if i.key_released(egui::Key::F5) {
					self.ExportWeb(Some(current_dir().unwrap().join("public")));
					
					open::that("http://localhost:7979/index.html");
				}
			}

			if i.key_released(egui::Key::I) {
				//println!("{:?}", current_dir().unwrap());
			}
		});
	}
}