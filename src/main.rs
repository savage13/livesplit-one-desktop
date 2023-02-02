#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
pub mod keys;
mod stream_markers;

use config::Config;
use keys::Key;

use bytemuck::{Pod, Zeroable};
use clap::Parser;
use lazy_static::lazy_static;
use livesplit_core::layout;
use livesplit_core::layout::LayoutSettings;
use livesplit_core::{auto_splitting, rendering::software::Renderer};
use livesplit_core::{layout::Layout, layout::LayoutState};
use livesplit_core::{SharedTimer, Timer};
use rfd::FileDialog;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::ModifiersState;
use winit::event::MouseScrollDelta;
use winit::event::VirtualKeyCode;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::new());
}

struct WTimer {
    timer: SharedTimer,
    markers: stream_markers::Client,
    layout: Layout,
    keys: HashMap<Key, Action>,
}

#[derive(Debug, Copy, Clone)]
enum Action {
    Split,
    Reset,
    Undo,
    Skip,
    Pause,
    UndoAllPauses,
    PreviousComparison,
    NextComparison,
    ToggleTimingMethod,
    HideComparison,
    OpenSplits,
    SaveSplits,
    OpenLayout,
    Quit,
    LayoutUp,
    LayoutDown,
}

pub fn save_state(timer_state: &livesplit_core::TimerState) {
    let path = config().state_file();
    std::fs::write(&path, timer_state.to_json()).unwrap();
}

impl WTimer {
    pub fn load_state(&mut self) {
        let path = config().state_file();
        if let Some(path_str) = path.to_str() {
            if let Some(timer_state) = livesplit_core::TimerState::from_file(path_str) {
                self.write().replace_state(&timer_state);
            }
        }
    }

    pub fn reset(&mut self) {
        self.write().reset(true);
    }
    pub fn skip_split(&mut self) {
        self.write().skip_split();
    }
    pub fn undo_split(&mut self) {
        self.write().undo_split();
    }
    pub fn pause(&mut self) {
        self.write().toggle_pause();
    }
    pub fn split_or_start(&mut self) {
        self.write().split_or_start();
    }
    pub fn switch_to_next_comparison(&mut self) {
        self.write().switch_to_next_comparison();
        config_mut().set_comparison(self.read().current_comparison());
    }
    pub fn switch_to_previous_comparison(&mut self) {
        self.write().switch_to_previous_comparison();
        config_mut().set_comparison(self.read().current_comparison());
    }
    pub fn turn_off_comparisons(&mut self) {
        self.write().set_current_comparison("None").unwrap();
        config_mut().set_comparison(self.read().current_comparison());
    }
    pub fn toggle_timing_method(&mut self) {
        //self.write().set_current_comparison("None").unwrap();
    }
    fn write(&self) -> RwLockWriteGuard<'_, Timer> {
        self.timer.write().unwrap()
    }

    fn read(&self) -> RwLockReadGuard<'_, Timer> {
        self.timer.read().unwrap()
    }
    fn new_from_splits(splits_file: PathBuf) -> Self {
        let mut splits_file = splits_file;
        config_mut().set_splits_path(&splits_file);
        splits_file.set_extension("lsz");
        config_mut().set_state_file(&splits_file);
        Self::new()
    }
    fn new() -> Self {
        let markers = config().build_marker_client();
        let layout = config().parse_layout_or_default();
        let run = config().parse_run_or_default();
        let timer = Timer::new(run).unwrap().into_shared();
        let auto_splitter = auto_splitting::Runtime::new(timer.clone());
        let keys = Self::hotkey_setup(&timer);
        config().maybe_load_auto_splitter(&auto_splitter);
        config().configure_timer(&mut timer.write().unwrap());
        timer.write().unwrap().set_on_timer_change(save_state);
        Self {
            timer,
            markers,
            layout,
            keys,
        }
    }
    fn hotkey_setup(timer: &SharedTimer) -> HashMap<Key, Action> {
        let use_global_hotkeys = config().use_global_hotkeys();
        let mut keys: HashMap<Key, Action> = HashMap::new();

        // Default Value for key commands
        let list = [
            ("open", "Ctrl+O", Action::OpenSplits),
            ("save", "Ctrl+S", Action::SaveSplits),
            ("quit", "Ctrl+Q", Action::Quit),
            ("hide", "Ctrl+T", Action::HideComparison),
            ("layout", "Ctrl+L", Action::OpenLayout),
            ("layout_up", "ArrowUp", Action::LayoutUp),
            ("layout_down", "ArrowDown", Action::LayoutDown),
        ];
        for (name, dcmd, action) in list {
            let key = config()
                .keys()
                .get(name)
                .unwrap_or(&dcmd.parse().unwrap())
                .into();
            keys.insert(key, action);
        }
        if use_global_hotkeys {
            let _hotkey_system = config().create_hotkey_system(timer.clone());
        } else {
            keys.insert(config().split_key().into(), Action::Split);
            keys.insert(config().reset_key().into(), Action::Reset);
            keys.insert(config().undo_key().into(), Action::Undo);
            keys.insert(config().skip_key().into(), Action::Skip);
            keys.insert(config().pause_key().into(), Action::Pause);
            keys.insert(config().undo_all_key().into(), Action::UndoAllPauses);
            keys.insert(config().prev_key().into(), Action::PreviousComparison);
            keys.insert(config().next_key().into(), Action::NextComparison);
            keys.insert(
                config().toggle_timing_method_key().into(),
                Action::ToggleTimingMethod,
            );
        }
        keys
    }
    fn handle_keypress(&mut self, key: Key) {
        let action = if let Some(action) = self.keys.get(&key) {
            *action
        } else {
            return;
        };
        self.action(&action);
    }
    fn open_layout(&mut self) -> Result<(), ()> {
        let path = pick_layout_file().ok_or(())?;
        let file = std::fs::read_to_string(&path).or(Err(()))?;
        if let Ok(settings) = LayoutSettings::from_json(Cursor::new(&file)) {
            self.layout = Layout::from_settings(settings);
        } else if let Ok(layout) = layout::parser::parse(&file) {
            self.layout = layout;
        } else {
            println!("Error parsing layout");
            return Ok(());
        }
        config_mut().set_layout_path(&path);
        config().save_config();

        Ok(())
    }
    fn action(&mut self, action: &Action) {
        match action {
            Action::Split => self.split_or_start(),
            Action::Pause => self.pause(),
            Action::Reset => self.reset(),
            Action::Undo => self.undo_split(),
            Action::Skip => self.skip_split(),
            Action::NextComparison => self.switch_to_next_comparison(),
            Action::PreviousComparison => self.switch_to_previous_comparison(),
            Action::HideComparison => self.turn_off_comparisons(),
            Action::LayoutUp => self.layout.scroll_up(),
            Action::LayoutDown => self.layout.scroll_down(),
            Action::SaveSplits => config().save_splits(&self.read()),
            Action::OpenSplits => {
                if let Some(file) = pick_splits_file() {
                    *self = WTimer::new_from_splits(file);
                    config().save_config();
                }
            }
            Action::OpenLayout => self.open_layout().unwrap_or(()),
            Action::ToggleTimingMethod => self.toggle_timing_method(),
            Action::Quit => {} //self.skip_split(),
            _ => unimplemented!("{:?}", action),
        }
    }
}
fn pick_splits_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("splits", &["lss", "rs"])
        .set_directory(".")
        .pick_file()
}
fn pick_layout_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("layout", &["json", "ls1l"])
        .set_directory(".")
        .pick_file()
}

fn config() -> RwLockReadGuard<'static, Config> {
    CONFIG.read().unwrap()
}
fn config_mut() -> RwLockWriteGuard<'static, Config> {
    CONFIG.write().unwrap()
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Hotkeys
    #[arg(short, long)]
    global_hotkeys: bool,
    /// Splits file
    #[arg(short, long)]
    splits: Option<PathBuf>,
    /// State file
    #[arg(short, long)]
    layout: Option<String>,
    /// State file
    #[arg(short, long)]
    timing_file: Option<PathBuf>,
}

fn scroll_up(delta: &MouseScrollDelta) -> bool {
    match delta {
        MouseScrollDelta::LineDelta(y, ..) => y < &0.0,
        MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition { y, .. }) => y < &0.0,
    }
}

fn key_code(input: &KeyboardInput, pressed: bool) -> Option<VirtualKeyCode> {
    if input.state == ElementState::Pressed && pressed {
        input.virtual_keycode
    } else {
        None
    }
}

fn main() {
    *config_mut() = Config::parse().unwrap_or_default();
    config().setup_logging();
    let args = Args::parse();
    if args.global_hotkeys {
        config_mut().set_use_global_hotkeys(args.global_hotkeys);
    }
    if let Some(ref val) = args.splits {
        config_mut().set_splits_path(&val.to_path_buf());
    }
    if let Some(val) = args.timing_file {
        config_mut().set_state_file(&val);
    }

    let file = config().splits_path();
    let mut wtimer = if let Some(file) = file {
        WTimer::new_from_splits(file)
    } else {
        WTimer::new()
    };

    let event_loop = EventLoop::new();
    let size = config().window_size();

    let size = winit::dpi::LogicalSize::new(size[0] as f64, size[1] as f64);
    let window = winit::window::WindowBuilder::new()
        .with_title("LiveSplit One")
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new();
    let mut layout_state = LayoutState::default();

    wtimer.load_state();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();
    let mut buf = Vec::new();

    let mut modifiers: ModifiersState = ModifiersState::empty();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent { event: ref ev, .. } => match ev {
                WindowEvent::CloseRequested { .. } => control_flow.set_exit(),
                WindowEvent::Resized(_size) => {}
                WindowEvent::MouseWheel { delta, .. } => {
                    if scroll_up(delta) {
                        wtimer.layout.scroll_up();
                    } else {
                        wtimer.layout.scroll_down();
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = key_code(input, true) {
                        let key = Key::from(key, modifiers);
                        wtimer.handle_keypress(key)
                    }
                }
                WindowEvent::ModifiersChanged(mods) => {
                    modifiers = *mods;
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width as usize, size.height as usize)
                };
                {
                    let timer = wtimer.timer.read().unwrap();
                    wtimer.markers.tick(&timer);
                    wtimer
                        .layout
                        .update_state(&mut layout_state, &timer.snapshot());
                }
                renderer.render(&layout_state, [width as u32, height as u32]);

                if buf.len() != width * height {
                    buf.resize(width * height, 0);
                }
                transpose(
                    bytemuck::cast_slice_mut(&mut buf),
                    bytemuck::cast_slice(renderer.image_data()),
                );
                surface.set_buffer(&buf, width as u16, height as u16);
            }
            _ => {}
        }
        if let Event::RedrawRequested(_) = event {}
        window.request_redraw();
    });
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
