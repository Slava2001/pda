use crate::{
    app::{
        App, AppError, apps::{AppCtlAppKind, build_app},
    }, display::Display, key_event::{Key, KeyEvent, KeyEventType}, menu::{
        Menu, draw_line,
        entries::{FocusController, MenuEntry},
    },
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
};
use std::{
    collections::{HashMap, hash_map::Iter},
    println,
    sync::Arc,
    time::Duration,
    unreachable,
};
use tokio::{
    select,
    sync::Mutex,
    task::{AbortHandle, Id, JoinError, JoinSet},
    time::interval,
};

struct AppInfo {
    run_abort_handle: AbortHandle,
    app: Arc<dyn App>,
    kind: AppCtlAppKind,
}

pub struct Manager {
    app_ctl: Arc<Mutex<AppCtl>>,
    main_menu: Menu,
}

impl Manager {
    pub fn new() -> Self {
        let app_ctl = Arc::new(Mutex::new(AppCtl::new()));
        Self {
            main_menu: Menu::new()
                .add(RunAppMenuEntry::new(app_ctl.clone()))
                .add(TaskListMenuEntry::new(app_ctl.clone())),
            app_ctl,
        }
    }

    pub async fn run(mut self) -> ! {
        let mut display = Display::new().expect("Failed to create display");
        let mut key_events = KeyEvent::new().expect("Failed to create kb");
        let mut frame_timer = interval(Duration::from_secs_f32(1.0 / 30.0));

        let id = self
            .app_ctl
            .try_lock()
            .unwrap()
            .spawn(AppCtlAppKind::Radio);
        self.app_ctl.try_lock().unwrap().set_active(Some(id));

        loop {
            select! {
                _ = frame_timer.tick() => {
                    display.clear(Rgb565::BLACK).ok();
                    if self.app_ctl.lock().await.get_active().is_some() {
                        self.app_ctl.lock().await.render(&mut display).await;
                    } else {
                        self.main_menu.render(&mut display);
                    }
                    display.flush();
                },
                key = key_events.next() => {
                    if let KeyEventType::Press(Key::VolumeUp) = key {
                        self.app_ctl.lock().await.set_active(None);
                    }
                    if self.app_ctl.lock().await.get_active().is_some() {
                        self.app_ctl.lock().await.key_event(key).await;
                    } else {
                        self.main_menu.update(key);
                    }
                },
                Some(event) = async {
                    let mut mg = self.app_ctl.lock().await;
                    mg.next_join_event().await
                } => {
                    self.app_ctl.lock().await.handle_next_join_event(event);
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
            }
            Err(join_err) => {
                println!("App {} panicked or canceled: {}", join_err.id(), join_err);
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

struct RunAppMenuEntry {
    app_ctl: Arc<Mutex<AppCtl>>,
    cursor: usize,
    apps_kinds: &'static [AppCtlAppKind],
}
impl RunAppMenuEntry {
    fn new(app_ctl: Arc<Mutex<AppCtl>>) -> Self {
        Self {
            app_ctl,
            cursor: 0,
            apps_kinds: &[AppCtlAppKind::Clock, AppCtlAppKind::Radio],
        }
    }
}
impl MenuEntry for RunAppMenuEntry {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        if parent.is_focused() {
            match key_event {
                KeyEventType::Press(key) | KeyEventType::Repeat(key) => match key {
                    Key::Up if self.cursor > 0 => {
                        self.cursor = self.cursor - 1;
                    }
                    Key::Down if self.cursor < self.apps_kinds.len() => {
                        self.cursor = self.cursor + 1;
                    }
                    Key::Enter if self.cursor == 0 => {
                        parent.release_focus();
                    }
                    Key::Enter => {
                        let id = self
                            .app_ctl
                            .try_lock()
                            .unwrap()
                            .spawn(self.apps_kinds[self.cursor - 1]);
                        self.app_ctl.try_lock().unwrap().set_active(Some(id));
                        parent.release_focus();
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            if let KeyEventType::Press(Key::Enter) = key_event {
                self.cursor = 0;
                parent.grab_focus();
            }
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line("Run", display, x, y);
    }

    fn render(&self, display: &mut Display, x: i32, y: i32) {
        for i in 0..=self.apps_kinds.len() {
            if i == self.cursor {
                draw_line("->", display, x, y + 20 * (i as i32 + 1));
            }
            let str = if i == 0 {
                "Back"
            } else {
                &self.apps_kinds[i - 1].as_ref()
            };
            draw_line(str, display, x + 20, y + 20 * (i as i32 + 1));
        }
    }
}

struct TaskListMenuEntry {
    app_ctl: Arc<Mutex<AppCtl>>,
    enters: Vec<(String, Id)>,
    cursor: usize,
}
impl TaskListMenuEntry {
    fn new(app_ctl: Arc<Mutex<AppCtl>>) -> Self {
        Self {
            app_ctl,
            cursor: 0,
            enters: Vec::new(),
        }
    }
}

impl MenuEntry for TaskListMenuEntry {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        if parent.is_focused() {
            match key_event {
                KeyEventType::Press(key) | KeyEventType::Repeat(key) => match key {
                    Key::Up if self.cursor > 0 => {
                        self.cursor = self.cursor - 1;
                    }
                    Key::Down if self.cursor < self.enters.len() => {
                        self.cursor = self.cursor + 1;
                    }
                    Key::Enter if self.cursor == 0 => {
                        parent.release_focus();
                    }
                    Key::Enter => {
                        parent.release_focus();
                        self.app_ctl
                            .try_lock()
                            .unwrap()
                            .set_active(Some(self.enters[self.cursor - 1].1));
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            if let KeyEventType::Press(Key::Enter) = key_event {
                self.enters = self
                    .app_ctl
                    .try_lock()
                    .unwrap()
                    .get_apps_list()
                    .map(|(id, info)| (info.kind.as_ref().to_string(), *id))
                    .collect();
                parent.grab_focus();
                self.cursor = 0;
            }
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line("Tasks List", display, x, y);
    }

    fn render(&self, display: &mut Display, x: i32, y: i32) {
        for i in 0..=self.enters.len() {
            if i == self.cursor {
                draw_line("->", display, x, y + 20 * (i as i32 + 1));
            }
            let str = if i == 0 {
                "Back".to_string()
            } else {
                format!("{}: {}", self.enters[i - 1].0, self.enters[i - 1].1)
            };
            draw_line(&str, display, x + 20, y + 20 * (i as i32 + 1));
        }
    }
}
