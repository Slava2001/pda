use crate::{
    app::{App, AppError, apps::{AppCtlAppKind, build_app}}, display::Display, key_event::{Key, KeyEvent, KeyEventType},
};
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::{Rgb565, RgbColor},
    text::Text,
};
use std::{
    collections::{HashMap, hash_map::Iter},
    println,
    sync::Arc,
    time::Duration,
    vec,
};
use tokio::{
    select,
    task::{AbortHandle, Id, JoinError, JoinSet},
    time::interval,
};

struct AppInfo {
    run_abort_handle: AbortHandle,
    app: Arc<dyn App>,
    kind: AppCtlAppKind,
}

pub struct Manager {
    app_ctl: AppCtl,
    main_menu: MainMenu,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            app_ctl: AppCtl::new(),
            main_menu: MainMenu::new(),
        }
    }

    pub async fn run(mut self) -> ! {
        let mut display = Display::new().expect("Failed to create display");
        let mut key_events = KeyEvent::new().expect("Failed to create kb");
        let mut frame_timer = interval(Duration::from_secs_f32(1.0 / 30.0));

        self.app_ctl.spawn(AppCtlAppKind::Clock);
        self.app_ctl.spawn(AppCtlAppKind::Clock);
        self.app_ctl.spawn(AppCtlAppKind::Clock);
        self.app_ctl.spawn(AppCtlAppKind::Clock);
        self.app_ctl.spawn(AppCtlAppKind::Clock);
        self.app_ctl.spawn(AppCtlAppKind::Radio);

        loop {
            self.main_menu.update(&mut self.app_ctl);
            select! {
                _ = frame_timer.tick() => {
                    display.clear(Rgb565::BLACK).ok();
                    if self.app_ctl.get_active().is_some() {
                        self.app_ctl.render(&mut display).await;
                    } else {
                        self.main_menu.render(&mut self.app_ctl, &mut display);
                    }
                    display.flush();
                }
                key = key_events.next() => {
                    if let KeyEventType::Press(Key::VolumeUp) = key {
                        self.app_ctl.set_active(None);
                    }
                    if self.app_ctl.get_active().is_some() {
                        self.app_ctl.key_event(key).await;
                    } else {
                        self.main_menu.key_event(&mut self.app_ctl, key);
                    }
                },
                Some(event) = self.app_ctl.next_join_event() => {
                    self.app_ctl.handle_next_join_event(event);
                }
            }
        }
    }
}

pub struct AppCtl {
    apps_run: JoinSet<Result<(), AppError>>,
    apps: HashMap<Id, AppInfo>,
    active_app: Option<Id>,
}

type JoinEvent = Result<(Id, Result<(), AppError>), JoinError>;

impl AppCtl {
    fn new() -> Self {
        Self {
            apps_run: JoinSet::new(),
            apps: HashMap::new(),
            active_app: None,
        }
    }

    fn get_apps_list(&self) -> Iter<'_, Id, AppInfo> {
        self.apps.iter()
    }

    async fn next_join_event(&mut self) -> Option<JoinEvent> {
        self.apps_run.join_next_with_id().await
    }

    fn handle_next_join_event(&mut self, event: JoinEvent) {
        println!("Handel join set event");
        let id = match event {
            Ok((id, _result)) => {
                println!("App {id} exited");
                id
            },
            Err(join_err) => {
                println!("App {} panicked or canceled", join_err.id());
                join_err.id()
            }
        };
        self.kill(id);
    }

    fn get_active(&self) -> Option<Id> {
        self.active_app
    }

    fn set_active(&mut self, app_id: Option<Id>) {
        self.active_app = app_id
    }

    fn spawn(&mut self, kind: AppCtlAppKind) -> Id {
        let app = build_app(kind);
        let app_c = app.clone();
        let handle = self.apps_run.spawn(async move { app_c.run().await });
        let id = handle.id();
        self.apps.insert(
            id,
            AppInfo {
                run_abort_handle: handle,
                app,
                kind,
            },
        );
        println!("Spawned app");
        id
    }

    fn kill(&mut self, id: Id) {
        let Some(app) = self.apps.remove(&id) else {
            return;
        };
        app.run_abort_handle.abort();
        if self.active_app == Some(id) {
            self.active_app = None;
        }
    }

    async fn key_event(&mut self, key: KeyEventType) {
        if let Some(id) = self.active_app {
            let _ = self.apps.get(&id).unwrap().app.key_event(key).await;
        }
    }

    async fn render(&self, display: &mut Display) {
        if let Some(id) = self.active_app {
            let _ = self.apps.get(&id).unwrap().app.render(display).await;
        }
    }
}

struct MainMenuEntry {
    id: Id,
    kind: AppCtlAppKind,
}

struct MainMenu {
    cursor_pos: usize,
    menu_entry: Vec<MainMenuEntry>,
}

impl MainMenu {
    fn new() -> Self {
        Self {
            cursor_pos: 0,
            menu_entry: vec![],
        }
    }

    fn update(&mut self, app_ctl: &mut AppCtl) {
        self.menu_entry = app_ctl
            .get_apps_list()
            .map(|(id, app)| MainMenuEntry {
                id: *id,
                kind: app.kind,
            })
            .collect();
        if self.cursor_pos > self.menu_entry.len() {
            self.cursor_pos = self.menu_entry.len();
        }
    }

    fn draw_line(text: &str, line: usize, display: &mut Display) {
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        Text::new(text, Point::new(10, (line as i32 + 1) * 25 ), style)
            .draw(display)
            .ok();
    }

    fn render(&self, _app_ctl: &mut AppCtl, display: &mut Display) {
        for (i, e) in self.menu_entry.iter().enumerate() {
            let text = format!("  {:?}:{}", e.kind, e.id);
            Self::draw_line(&text, i, display);
        }
        Self::draw_line("->", self.cursor_pos, display);
    }

    fn key_event(&mut self, app_ctl: &mut AppCtl, event: KeyEventType) {
        match event {
            KeyEventType::Press(key) | KeyEventType::Repeat(key) => match key {
                Key::Up => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos = self.cursor_pos - 1;
                    }
                }
                Key::Down => {
                    if self.cursor_pos + 1 < self.menu_entry.len() {
                        self.cursor_pos = self.cursor_pos + 1;
                    }
                }
                Key::Enter => {
                    if !self.menu_entry.is_empty() {
                        app_ctl.set_active(Some(self.menu_entry[self.cursor_pos].id));
                    }
                }
                _ => {

                }
            },
            _ => {}
        };
    }
}
