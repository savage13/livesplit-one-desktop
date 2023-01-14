#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod stream_markers;

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::Config;
use bytemuck::{Pod, Zeroable};
use livesplit_core::{
    auto_splitting, layout::Component, layout::LayoutState, rendering::software::Renderer, Timer,
};
use mimalloc::MiMalloc;
use minifb::{Key, KeyRepeat};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct WTimer(Arc<RwLock<Timer>>);

impl WTimer {
    pub fn save_state(&self) {
        let timer_state = self.read().timer_state();
        std::fs::write("timer_state.json", timer_state.to_json()).unwrap();
    }
    pub fn load_state(&mut self) {
        if let Some(timer_state) = livesplit_core::TimerState::from_file("timer_state.json") {
            println!("loading timer state in timer_state.json");
            self.write().replace_state(&timer_state);
        }
    }
    pub fn reset(&mut self) {
        self.write().reset(true);
        self.save_state();
    }
    pub fn skip_split(&mut self) {
        self.write().skip_split();
        self.save_state();
    }
    pub fn undo_split(&mut self) {
        self.write().undo_split();
        self.save_state();
    }
    pub fn pause(&mut self) {
        self.write().toggle_pause();
        self.save_state();
    }
    pub fn split_or_start(&mut self) {
        self.write().split_or_start();
        self.save_state();
    }
    pub fn switch_to_next_comparison(&mut self) {
        self.write().switch_to_next_comparison();
    }
    pub fn switch_to_previous_comparison(&mut self) {
        self.write().switch_to_previous_comparison();
    }
    pub fn turn_off_comparison(&mut self) {
        self.write().set_current_comparison("None").unwrap();
    }
    fn write(&self) -> RwLockWriteGuard<'_, Timer> {
        self.0.write().unwrap()
    }
    fn read(&self) -> RwLockReadGuard<'_, Timer> {
        self.0.read().unwrap()
    }
}

fn main() {
    let config = Config::parse("config.yaml").unwrap_or_default();
    config.setup_logging();

    let run = config.parse_run_or_default();

    let timer = Timer::new(run).unwrap().into_shared();
    config.configure_timer(&mut timer.write().unwrap());

    let mut markers = config.build_marker_client();

    let auto_splitter = auto_splitting::Runtime::new(timer.clone());
    config.maybe_load_auto_splitter(&auto_splitter);

    let _hotkey_system = config.create_hotkey_system(timer.clone());

    let mut layout = config.parse_layout_or_default();
    for comp in &mut layout.components {
        match comp {
            Component::Splits(s) => {
                let settings = s.settings_mut();
                //settings.fill_with_blank_space = false;
                dbg!(&settings.fill_with_blank_space);
            }
            _ => {}
        }
    }
    /*
    if false {
        layout.push(livesplit_core::component::graph::Component::new());
        let mut layout_file = std::fs::File::create("layout.json").unwrap();
        let settings = layout.settings();
        settings.write_json(layout_file);
    }
     */
    //let settings = layout.general_settings();

    let mut wtimer = WTimer(timer);

    let mut window = config.build_window().unwrap();

    let mut renderer = Renderer::new();
    let mut layout_state = LayoutState::default();
    let mut buf = Vec::new();

    wtimer.load_state(); //load_state(&mut timer.write().unwrap());

    while window.is_open() {
        if let Some((_, val)) = window.get_scroll_wheel() {
            if val > 0.0 {
                layout.scroll_up();
            } else if val < 0.0 {
                layout.scroll_down();
            }
        }

        if window.is_key_pressed(Key::S, KeyRepeat::No)
            && (window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl))
        {
            config.save_splits(&wtimer.0.read().unwrap());
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            wtimer.split_or_start();
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            wtimer.pause();
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            wtimer.reset();
        }
        if window.is_key_pressed(Key::U, KeyRepeat::No) {
            wtimer.undo_split();
        }
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            wtimer.skip_split();
        }
        if window.is_key_pressed(Key::Left, KeyRepeat::No) {
            wtimer.switch_to_previous_comparison();
        }
        if window.is_key_pressed(Key::Right, KeyRepeat::No) {
            wtimer.switch_to_next_comparison();
        }
        if window.is_key_pressed(Key::Down, KeyRepeat::No) {
            wtimer.turn_off_comparison();
        }

        let (width, height) = window.get_size();
        if width != 0 && height != 0 {
            {
                let timer = wtimer.0.read().unwrap();
                markers.tick(&timer);
                layout.update_state(&mut layout_state, &timer.snapshot());
            }
            renderer.render(&layout_state, [width as _, height as _]);

            buf.resize(width * height, 0);

            transpose(
                bytemuck::cast_slice_mut(&mut buf),
                bytemuck::cast_slice(renderer.image_data()),
            );
        }
        window.update_with_buffer(&buf, width, height).unwrap();
    }
}

pub fn transpose(dst: &mut [[u8; 4]], src: &[[u8; 4]]) {
    #[derive(Copy, Clone, Pod, Zeroable)]
    #[repr(transparent)]
    pub struct Chunk([[u8; 4]; 8]);

    let (dst_before, dst, dst_after) = bytemuck::pod_align_to_mut::<_, Chunk>(dst);
    let (src_before, src, src_after) = bytemuck::pod_align_to::<_, Chunk>(src);

    for (dst, &[r, g, b, a]) in dst_before.iter_mut().zip(src_before) {
        *dst = [b, g, r, a];
    }
    for (dst, src) in dst.iter_mut().zip(src) {
        for (dst, &[r, g, b, a]) in dst.0.iter_mut().zip(&src.0) {
            *dst = [b, g, r, a];
        }
    }
    for (dst, &[r, g, b, a]) in dst_after.iter_mut().zip(src_after) {
        *dst = [b, g, r, a];
    }
}
