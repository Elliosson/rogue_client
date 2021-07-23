use rltk::{Console, GameState, Rltk, RGB};
#[macro_use]
extern crate specs_derive;
mod components;
pub use components::*;
mod general_network;
#[cfg(target_arch = "wasm32")]
mod wasm_network;

#[cfg(not(target_arch = "wasm32"))]
mod desktop_network;

mod runstate;

use runstate::{player_input, Runstate};

use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

extern crate specs;
use specs::prelude::*;
mod gui;

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
pub fn consol_print(mes: String) {
    console_log!("{}", mes);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn consol_print(mes: String) {
    println!("{}", mes);
}

const CENTER:i32 = 30;

struct State {
    pub rectangle: Rect,
    pub data: Arc<Mutex<Data>>,
    pub to_send: Arc<Mutex<Vec<String>>>,
    pub player_info: PlayerInfo,
    pub runstate: Runstate,
    pub ecs: World,
    pub pseudo: String,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let data_guard = self.data.lock().unwrap();

        match serde_json::from_str(&data_guard.info_string) {
            Ok(info) => self.player_info = info,
            Err(_) => {
                consol_print("unable to deserialize json".to_string());
            }
        }

        ctx.cls();

        draw_map(ctx, data_guard.map.clone(), &self.player_info.my_info.pos);

        draw_player(ctx);

        gui::draw_ui(ctx, &self.player_info);

        self.runstate = player_input(
            data_guard.my_uid.clone(),
            self.to_send.clone(),
            ctx,
            &self.runstate,
            &mut self.rectangle,
            &self.player_info,
            &mut self.pseudo,
        );

        for pos in data_guard.characters.iter() {
            ctx.set(
                pos.x,
                pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('X'),
            );
        }
    }
}

pub struct Rect {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
}

pub fn draw_rect(ctx: &mut Rltk, rect: &Rect) {
    for x in rect.x..rect.width + rect.x {
        for y in rect.y..rect.height + rect.y {
            ctx.set(
                x,
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('#'),
            );
        }
    }
}

pub struct Data {
    characters: Vec<Point>,
    my_uid: String,
    map: Vec<(Point, Renderable)>,
    info_string: String,
}

fn draw_map(ctx: &mut Rltk, mut map: Vec<(Point, Renderable)>, my_pos: &Position) {
    let center_x = CENTER;
    let center_y = CENTER;
    map.sort_by(|a, b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render) in map.iter() {
        let x = pos.x - my_pos.x + center_x;
        let y = pos.y - my_pos.y + center_y;
        if gui::inside_windows(x, y) {
            ctx.set(x, y, render.fg, render.bg, render.glyph);
        }
    }
}

fn draw_player(ctx: &mut Rltk){
    ctx.set(CENTER, CENTER, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), '@' as u8);
}

fn main() {
    let data = Data {
        characters: vec![],
        my_uid: "".to_string(),
        map: vec![],
        info_string: "".to_string(),
    };
    let protect_data: Arc<Mutex<Data>> = Arc::new(Mutex::new(data));
    let to_send: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let context = Rltk::init_simple8x8(180 as u32, 90 as u32, "Ecosystem simulator", "resources");
    let gs = State {
        rectangle: Rect {
            height: 6,
            width: 2,
            x: 5,
            y: 5,
        },
        data: protect_data.clone(),
        to_send: to_send.clone(),
        ecs: World::new(),
        player_info: PlayerInfo {
            inventaire: Vec::new(),
            close_interations: Vec::new(),
            my_info: MyInfo {
                pos: Position { x: 0, y: 0 },
                hp: 0,
                max_hp: 0,
                player_log: vec![],
            },
            possible_builds: Vec::new(),
            equipement: Vec::new(),
            combat_stats: Default::default(),
        },
        runstate: Runstate::Register,
        pseudo: "".to_string(),
    };

    lauch_network(protect_data.clone(), to_send.clone());
    rltk::main_loop(context, gs);
}

#[cfg(target_arch = "wasm32")]
fn lauch_network(protect_data: Arc<Mutex<Data>>, to_send: Arc<Mutex<Vec<String>>>) {
    wasm_network::start_websocket(protect_data, to_send).expect("Unable to start websocket");
}

#[cfg(not(target_arch = "wasm32"))]
fn lauch_network(protect_data: Arc<Mutex<Data>>, to_send: Arc<Mutex<Vec<String>>>) {
    desktop_network::start_websocket(protect_data, to_send);
}
