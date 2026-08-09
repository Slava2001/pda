use crate::{
    app::{
        App, AppError,
        apps::{AppCtlAppKind, build_app},
    },
    display::Display,
    key_event::{Key, KeyEvent, KeyEventType},
    menu::{
        Menu, draw_line,
        entries::{FocusController, MenuEntry, list::List},
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

    fn get_info(&self, id: Id) -> Option<&AppInfo> {
        self.apps.get(&id)
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

struct ListEntry<T: 'static + Send + Sync>(T, String);

impl<T: 'static + Send + Sync, S: Into<String>> From<(T, S)> for ListEntry<T> {
    fn from(value: (T, S)) -> Self {
        Self(value.0, value.1.into())
    }
}

impl<T: 'static + Send + Sync> std::fmt::Display for ListEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

struct RunAppMenuEntry {
    app_ctl: Arc<Mutex<AppCtl>>,
    app_list: &'static [AppCtlAppKind],
    list: List<&'static str, ListEntry<Option<AppCtlAppKind>>>,
}

impl RunAppMenuEntry {
    fn new(app_ctl: Arc<Mutex<AppCtl>>) -> Self {
        let app_list: &'static [AppCtlAppKind] = &[AppCtlAppKind::Clock, AppCtlAppKind::Radio];
        Self {
            list: List::new(
                "Run",
                [(None, "Back")]
                    .into_iter()
                    .chain(app_list.iter().map(|k| (Some(*k), k.as_ref())))
                    .map(ListEntry::from)
                    .collect(),
            ),
            app_list,
            app_ctl,
        }
    }
}
impl MenuEntry for RunAppMenuEntry {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        self.list.update(parent, key_event);
        if let Some(ListEntry(Some(app_kind), _)) = self.list.value() {
            self.app_ctl.try_lock().unwrap().spawn(*app_kind);
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        self.list.render_line(display, x, y);
    }

    fn render(&self, display: &mut Display, x: i32, y: i32) {
        self.list.render(display, x, y);
    }
}

struct TaskListMenuEntry {
    app_ctl: Arc<Mutex<AppCtl>>,
    tasks_list: List<String, ListEntry<Option<Id>>>,
    action_list: List<&'static str, ListEntry<usize>>,
    current_task: Option<Id>,
}

impl TaskListMenuEntry {
    fn new(app_ctl: Arc<Mutex<AppCtl>>) -> Self {
        Self {
            app_ctl,
            tasks_list: List::new("".to_string(), Vec::new()),
            action_list: List::new("", [(0, "Back"), (0, "Back"), (0, "Back")].into_iter().map(ListEntry::from).collect()),
            current_task: None,
        }
    }
}

impl MenuEntry for TaskListMenuEntry {
    fn update(&mut self, parent: &mut dyn FocusController, key_event: KeyEventType) {
        if let Some(_current_task) = self.current_task {
            self.action_list.update(parent, key_event);
            if let Some(ListEntry(_action_id, _)) = self.action_list.value() {
                self.current_task = None;
            }
        } else {
            if !parent.is_focused() {
                if let KeyEventType::Press(Key::Enter) = key_event {
                    self.tasks_list.set_entries(
                        [(None, "Back".to_string())]
                            .into_iter()
                            .chain(self.app_ctl.try_lock().unwrap().get_apps_list().map(
                                |(id, info)| {
                                    (
                                        Some(*id),
                                        format!("{}: {}", info.kind.as_ref().to_string(), id),
                                    )
                                },
                            ))
                            .map(ListEntry::from)
                            .collect(),
                    );
                }
            }
            self.tasks_list.update(parent, key_event);
            if let Some(ListEntry(Some(app_id), _)) = self.tasks_list.value() {
                parent.grab_focus();
                self.current_task = Some(*app_id);
            }
        }
    }

    fn render_line(&self, display: &mut Display, x: i32, y: i32) {
        draw_line("Task list", display, x, y);
    }

    fn render(&self, display: &mut Display, x: i32, y: i32) {
        if self.current_task.is_none() {
            self.tasks_list.render(display, x, y);
        } else {
            self.action_list.render(display, x, y);
        }
    }
}
