use std::{
    collections::HashMap,
    iter::FromIterator,
    process::Child,
    sync::{Arc, Mutex},
};

use sciter::Value;

use hbb_common::{
    allow_err,
    config::{LocalConfig, PeerConfig},
    log,
};

#[cfg(not(any(feature = "flutter", feature = "cli")))]
use crate::ui_session_interface::Session;
use crate::{common::get_app_name, ipc, ui_interface::*};

mod cm;
#[cfg(feature = "inline")]
pub mod inline;
pub mod remote;

pub type Children = Arc<Mutex<(bool, HashMap<(String, String), Child>)>>;
#[allow(dead_code)]
type Status = (i32, bool, i64, String);

lazy_static::lazy_static! {
    // stupid workaround for https://sciter.com/forums/topic/crash-on-latest-tis-mac-sdk-sometimes/
    static ref STUPID_VALUES: Mutex<Vec<Arc<Vec<Value>>>> = Default::default();
}

#[cfg(not(any(feature = "flutter", feature = "cli")))]
lazy_static::lazy_static! {
    pub static ref CUR_SESSION: Arc<Mutex<Option<Session<remote::SciterHandler>>>> = Default::default();
    static ref CHILDREN : Children = Default::default();
}

struct UIHostHandler;

pub fn start(args: &mut [String]) {
    #[cfg(target_os = "macos")]
    crate::platform::delegate::show_dock();
    #[cfg(all(target_os = "linux", feature = "inline"))]
    {
        #[cfg(feature = "appimage")]
        let prefix = std::env::var("APPDIR").unwrap_or("".to_string());
        #[cfg(not(feature = "appimage"))]
        let prefix = "".to_string();
        #[cfg(feature = "flatpak")]
        let dir = "/app";
        #[cfg(not(feature = "flatpak"))]
        let dir = "/usr";
        sciter::set_library(&(prefix + dir + "/lib/rustdesk/libsciter-gtk.so")).ok();
    }
    // https://github.com/c-smile/sciter-sdk/blob/master/include/sciter-x-types.h
    // https://github.com/rustdesk/rustdesk/issues/132#issuecomment-886069737
    #[cfg(windows)]
    allow_err!(sciter::set_options(sciter::RuntimeOptions::GfxLayer(
        sciter::GFX_LAYER::WARP
    )));
    #[cfg(all(windows, not(feature = "inline")))]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }
    use sciter::SCRIPT_RUNTIME_FEATURES::*;
    allow_err!(sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        ALLOW_FILE_IO as u8 | ALLOW_SOCKET_IO as u8 | ALLOW_EVAL as u8 | ALLOW_SYSINFO as u8
    )));
    let mut frame = sciter::WindowBuilder::main_window().create();
    #[cfg(windows)]
    allow_err!(sciter::set_options(sciter::RuntimeOptions::UxTheming(true)));
    frame.set_title(&crate::get_app_name());
    #[cfg(target_os = "macos")]
    crate::platform::delegate::make_menubar(frame.get_host(), args.is_empty());
    let page;
    if args.len() > 1 && args[0] == "--play" {
        args[0] = "--connect".to_owned();
        let path: std::path::PathBuf = (&args[1]).into();
        let id = path
            .file_stem()
            .map(|p| p.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();
        args[1] = id;
    }
    if args.is_empty() {
        let children: Children = Default::default();
        std::thread::spawn(move || check_zombie(children));
        crate::common::check_software_update();
        frame.event_handler(UI {});
        frame.sciter_handler(UIHostHandler {});
        page = "index.html";
        // Start pulse audio local server.
        #[cfg(target_os = "linux")]
        std::thread::spawn(crate::ipc::start_pa);
    } else if args[0] == "--install" {
        frame.event_handler(UI {});
        frame.sciter_handler(UIHostHandler {});
        page = "install.html";
    } else if args[0] == "--cm" {
        frame.register_behavior("connection-manager", move || {
            Box::new(cm::SciterConnectionManager::new())
        });
        page = "cm.html";
    } else if (args[0] == "--connect"
        || args[0] == "--file-transfer"
        || args[0] == "--port-forward"
        || args[0] == "--rdp")
        && args.len() > 1
    {
        #[cfg(windows)]
        {
            let hw = frame.get_host().get_hwnd();
            crate::platform::windows::enable_lowlevel_keyboard(hw as _);
        }
        let mut iter = args.iter();
        let cmd = iter.next().unwrap().clone();
        let id = iter.next().unwrap().clone();
        let pass = iter.next().unwrap_or(&"".to_owned()).clone();
        let args: Vec<String> = iter.map(|x| x.clone()).collect();
        frame.set_title(&id);
        frame.register_behavior("native-remote", move || {
            let handler =
                remote::SciterSession::new(cmd.clone(), id.clone(), pass.clone(), args.clone());
            #[cfg(not(any(feature = "flutter", feature = "cli")))]
            {
                *CUR_SESSION.lock().unwrap() = Some(handler.inner());
            }
            Box::new(handler)
        });
        page = "remote.html";
    } else {
        log::error!("Wrong command: {:?}", args);
        return;
    }
    #[cfg(feature = "inline")]
    {
        let html = if page == "index.html" {
            inline::get_index()
        } else if page == "cm.html" {
            inline::get_cm()
        } else if page == "install.html" {
            inline::get_install()
        } else {
            inline::get_remote()
        };
        frame.load_html(html.as_bytes(), Some(page));
    }
    #[cfg(not(feature = "inline"))]
    frame.load_file(&format!(
        "file://{}/src/ui/{}",
        std::env::current_dir()
            .map(|c| c.display().to_string())
            .unwrap_or("".to_owned()),
        page
    ));
    frame.run_app();
}

struct UI {}

impl UI {
    fn recent_sessions_updated(&self) -> bool {
        recent_sessions_updated()
    }

    fn get_id(&self) -> String {
        ipc::get_id()
    }

    fn temporary_password(&mut self) -> String {
        temporary_password()
    }

    fn update_temporary_password(&self) {
        update_temporary_password()
    }

    fn permanent_password(&self) -> String {
        permanent_password()
    }

    fn set_permanent_password(&self, password: String) {
        set_permanent_password(password);
    }

    fn get_remote_id(&mut self) -> String {
        LocalConfig::get_remote_id()
    }

    fn set_remote_id(&mut self, id: String) {
        LocalConfig::set_remote_id(&id);
    }

    fn goto_install(&mut self) {
        goto_install();
    }

    fn install_me(&mut self, _options: String, _path: String) {
        install_me(_options, _path, false, false);
    }

    fn update_me(&self, _path: String) {
        update_me(_path);
    }

    fn run_without_install(&self) {
        run_without_install();
    }

    fn show_run_without_install(&self) -> bool {
        show_run_without_install()
    }

    fn get_license(&self) -> String {
        get_license()
    }

    fn get_option(&self, key: String) -> String {
        get_option(key)
    }

    fn get_local_option(&self, key: String) -> String {
        get_local_option(key)
    }

    fn set_local_option(&self, key: String, value: String) {
        set_local_option(key, value);
    }

    fn peer_has_password(&self, id: String) -> bool {
        peer_has_password(id)
    }

    fn forget_password(&self, id: String) {
        forget_password(id)
    }

    fn get_peer_option(&self, id: String, name: String) -> String {
        get_peer_option(id, name)
    }

    fn set_peer_option(&self, id: String, name: String, value: String) {
        set_peer_option(id, name, value)
    }

    fn using_public_server(&self) -> bool {
        using_public_server()
    }

    fn get_options(&self) -> Value {
        let hashmap: HashMap<String, String> = serde_json::from_str(&get_options()).unwrap();
        let mut m = Value::map();
        for (k, v) in hashmap {
            m.set_item(k, v);
        }
        m
    }

    fn test_if_valid_server(&self, host: String) -> String {
        test_if_valid_server(host)
    }

    fn get_sound_inputs(&self) -> Value {
        Value::from_iter(get_sound_inputs())
    }

    fn set_options(&self, v: Value) {
        let mut m = HashMap::new();
        for (k, v) in v.items() {
            if let Some(k) = k.as_string() {
                if let Some(v) = v.as_string() {
                    if !v.is_empty() {
                        m.insert(k, v);
                    }
                }
            }
        }
        set_options(m);
    }

    fn set_option(&self, key: String, value: String) {
        set_option(key, value);
    }

    fn install_path(&mut self) -> String {
        install_path()
    }

    fn get_socks(&self) -> Value {
        Value::from_iter(get_socks())
    }

    fn set_socks(&self, proxy: String, username: String, password: String) {
        set_socks(proxy, username, password)
    }

    fn is_installed(&self) -> bool {
        is_installed()
    }

    fn is_root(&self) -> bool {
        is_root()
    }

    fn is_release(&self) -> bool {
        #[cfg(not(debug_assertions))]
        return true;
        #[cfg(debug_assertions)]
        return false;
    }

    fn is_rdp_service_open(&self) -> bool {
        is_rdp_service_open()
    }

    fn is_share_rdp(&self) -> bool {
        is_share_rdp()
    }

    fn set_share_rdp(&self, _enable: bool) {
        set_share_rdp(_enable);
    }

    fn is_installed_lower_version(&self) -> bool {
        is_installed_lower_version()
    }

    fn closing(&mut self, x: i32, y: i32, w: i32, h: i32) {
        crate::server::input_service::fix_key_down_timeout_at_exit();
        LocalConfig::set_size(x, y, w, h);
    }

    fn get_size(&mut self) -> Value {
        let s = LocalConfig::get_size();
        let mut v = Vec::new();
        v.push(s.0);
        v.push(s.1);
        v.push(s.2);
        v.push(s.3);
        Value::from_iter(v)
    }

    fn get_mouse_time(&self) -> f64 {
        get_mouse_time()
    }

    fn check_mouse_time(&self) {
        check_mouse_time()
    }

    fn get_connect_status(&mut self) -> Value {
        let mut v = Value::array(0);
        let x = get_connect_status();
        v.push(x.0);
        v.push(x.1);
        v.push(x.3);
        v
    }

    #[inline]
    fn get_peer_value(id: String, p: PeerConfig) -> Value {
        let values = vec![
            id,
            p.info.username.clone(),
            p.info.hostname.clone(),
            p.info.platform.clone(),
            p.options.get("alias").unwrap_or(&"".to_owned()).to_owned(),
        ];
        Value::from_iter(values)
    }

    fn get_peer(&self, id: String) -> Value {
        let c = get_peer(id.clone());
        Self::get_peer_value(id, c)
    }

    fn get_fav(&self) -> Value {
        Value::from_iter(get_fav())
    }

    fn store_fav(&self, fav: Value) {
        let mut tmp = vec![];
        fav.values().for_each(|v| {
            if let Some(v) = v.as_string() {
                if !v.is_empty() {
                    tmp.push(v);
                }
            }
        });
        store_fav(tmp);
    }

    fn get_recent_sessions(&mut self) -> Value {
        // to-do: limit number of recent sessions, and remove old peer file
        let peers: Vec<Value> = PeerConfig::peers()
            .drain(..)
            .map(|p| Self::get_peer_value(p.0, p.2))
            .collect();
        Value::from_iter(peers)
    }

    fn get_icon(&mut self) -> String {
        get_icon()
    }

    fn remove_peer(&mut self, id: String) {
        PeerConfig::remove(&id);
    }

    fn remove_discovered(&mut self, id: String) {
        remove_discovered(id);
    }

    fn send_wol(&mut self, id: String) {
        crate::lan::send_wol(id)
    }

    fn new_remote(&mut self, id: String, remote_type: String, force_relay: bool) {
        new_remote(id, remote_type, force_relay)
    }

    fn is_process_trusted(&mut self, _prompt: bool) -> bool {
        is_process_trusted(_prompt)
    }

    fn is_can_screen_recording(&mut self, _prompt: bool) -> bool {
        is_can_screen_recording(_prompt)
    }

    fn is_installed_daemon(&mut self, _prompt: bool) -> bool {
        is_installed_daemon(_prompt)
    }

    fn get_error(&mut self) -> String {
        get_error()
    }

    fn is_login_wayland(&mut self) -> bool {
        is_login_wayland()
    }

    fn current_is_wayland(&mut self) -> bool {
        current_is_wayland()
    }

    fn get_software_update_url(&self) -> String {
        crate::SOFTWARE_UPDATE_URL.lock().unwrap().clone()
    }

    fn get_new_version(&self) -> String {
        get_new_version()
    }

    fn get_version(&self) -> String {
        get_version()
    }

    fn get_app_name(&self) -> String {
        get_app_name()
    }

    fn get_software_ext(&self) -> String {
        #[cfg(windows)]
        let p = "exe";
        #[cfg(target_os = "macos")]
        let p = "dmg";
        #[cfg(target_os = "linux")]
        let p = "deb";
        p.to_owned()
    }

    fn get_software_store_path(&self) -> String {
        let mut p = std::env::temp_dir();
        let name = crate::SOFTWARE_UPDATE_URL
            .lock()
            .unwrap()
            .split("/")
            .last()
            .map(|x| x.to_owned())
            .unwrap_or(crate::get_app_name());
        p.push(name);
        format!("{}.{}", p.to_string_lossy(), self.get_software_ext())
    }

    fn create_shortcut(&self, _id: String) {
        #[cfg(windows)]
        create_shortcut(_id)
    }

    fn discover(&self) {
        std::thread::spawn(move || {
            allow_err!(crate::lan::discover());
        });
    }

    fn get_lan_peers(&self) -> String {
        // let peers = get_lan_peers()
        //     .into_iter()
        //     .map(|mut peer| {
        //         (
        //             peer.remove("id").unwrap_or_default(),
        //             peer.remove("username").unwrap_or_default(),
        //             peer.remove("hostname").unwrap_or_default(),
        //             peer.remove("platform").unwrap_or_default(),
        //         )
        //     })
        //     .collect::<Vec<(String, String, String, String)>>();
        serde_json::to_string(&get_lan_peers()).unwrap_or_default()
    }

    fn get_uuid(&self) -> String {
        get_uuid()
    }

    fn open_url(&self, url: String) {
        #[cfg(windows)]
        let p = "explorer";
        #[cfg(target_os = "macos")]
        let p = "open";
        #[cfg(target_os = "linux")]
        let p = if std::path::Path::new("/usr/bin/firefox").exists() {
            "firefox"
        } else {
            "xdg-open"
        };
        allow_err!(std::process::Command::new(p).arg(url).spawn());
    }

    fn change_id(&self, id: String) {
        let old_id = self.get_id();
        change_id_shared(id, old_id);
    }

    fn post_request(&self, url: String, body: String, header: String) {
        post_request(url, body, header)
    }

    fn is_ok_change_id(&self) -> bool {
        machine_uid::get().is_ok()
    }

    fn get_async_job_status(&self) -> String {
        get_async_job_status()
    }

    fn t(&self, name: String) -> String {
        crate::client::translate(name)
    }

    fn is_xfce(&self) -> bool {
        crate::platform::is_xfce()
    }

    fn get_api_server(&self) -> String {
        get_api_server()
    }

    fn has_hwcodec(&self) -> bool {
        has_hwcodec()
    }

    fn get_langs(&self) -> String {
        get_langs()
    }

    fn default_video_save_directory(&self) -> String {
        default_video_save_directory()
    }

    fn handle_relay_id(&self, id: String) -> String {
        handle_relay_id(id)
    }
}

impl sciter::EventHandler for UI {
    sciter::dispatch_script_call! {
        fn t(String);
        fn get_api_server();
        fn is_xfce();
        fn using_public_server();
        fn get_id();
        fn temporary_password();
        fn update_temporary_password();
        fn permanent_password();
        fn set_permanent_password(String);
        fn get_remote_id();
        fn set_remote_id(String);
        fn closing(i32, i32, i32, i32);
        fn get_size();
        fn new_remote(String, String, bool);
        fn send_wol(String);
        fn remove_peer(String);
        fn remove_discovered(String);
        fn get_connect_status();
        fn get_mouse_time();
        fn check_mouse_time();
        fn get_recent_sessions();
        fn get_peer(String);
        fn get_fav();
        fn store_fav(Value);
        fn recent_sessions_updated();
        fn get_icon();
        fn install_me(String, String);
        fn is_installed();
        fn is_root();
        fn is_release();
        fn set_socks(String, String, String);
        fn get_socks();
        fn is_rdp_service_open();
        fn is_share_rdp();
        fn set_share_rdp(bool);
        fn is_installed_lower_version();
        fn install_path();
        fn goto_install();
        fn is_process_trusted(bool);
        fn is_can_screen_recording(bool);
        fn is_installed_daemon(bool);
        fn get_error();
        fn is_login_wayland();
        fn current_is_wayland();
        fn get_options();
        fn get_option(String);
        fn get_local_option(String);
        fn set_local_option(String, String);
        fn get_peer_option(String, String);
        fn peer_has_password(String);
        fn forget_password(String);
        fn set_peer_option(String, String, String);
        fn get_license();
        fn test_if_valid_server(String);
        fn get_sound_inputs();
        fn set_options(Value);
        fn set_option(String, String);
        fn get_software_update_url();
        fn get_new_version();
        fn get_version();
        fn update_me(String);
        fn show_run_without_install();
        fn run_without_install();
        fn get_app_name();
        fn get_software_store_path();
        fn get_software_ext();
        fn open_url(String);
        fn change_id(String);
        fn get_async_job_status();
        fn post_request(String, String, String);
        fn is_ok_change_id();
        fn create_shortcut(String);
        fn discover();
        fn get_lan_peers();
        fn get_uuid();
        fn has_hwcodec();
        fn get_langs();
        fn default_video_save_directory();
        fn handle_relay_id(String);
    }
}

impl sciter::host::HostHandler for UIHostHandler {
    fn on_graphics_critical_failure(&mut self) {
        log::error!("Critical rendering error: e.g. DirectX gfx driver error. Most probably bad gfx drivers.");
    }
}

pub fn check_zombie(children: Children) {
    let mut deads = Vec::new();
    loop {
        let mut lock = children.lock().unwrap();
        let mut n = 0;
        for (id, c) in lock.1.iter_mut() {
            if let Ok(Some(_)) = c.try_wait() {
                deads.push(id.clone());
                n += 1;
            }
        }
        for ref id in deads.drain(..) {
            lock.1.remove(id);
        }
        if n > 0 {
            lock.0 = true;
        }
        drop(lock);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[cfg(not(target_os = "linux"))]
fn get_sound_inputs() -> Vec<String> {
    let mut out = Vec::new();
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    if let Ok(devices) = host.devices() {
        for device in devices {
            if device.default_input_config().is_err() {
                continue;
            }
            if let Ok(name) = device.name() {
                out.push(name);
            }
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn get_sound_inputs() -> Vec<String> {
    crate::platform::linux::get_pa_sources()
        .drain(..)
        .map(|x| x.1)
        .collect()
}

// sacrifice some memory
pub fn value_crash_workaround(values: &[Value]) -> Arc<Vec<Value>> {
    let persist = Arc::new(values.to_vec());
    STUPID_VALUES.lock().unwrap().push(persist.clone());
    persist
}

#[inline]
pub fn new_remote(id: String, remote_type: String, force_relay: bool) {
    let mut lock = CHILDREN.lock().unwrap();
    let mut args = vec![format!("--{}", remote_type), id.clone()];
    if force_relay {
        args.push("".to_string()); // password
        args.push("--relay".to_string());
    }
    let key = (id.clone(), remote_type.clone());
    if let Some(c) = lock.1.get_mut(&key) {
        if let Ok(Some(_)) = c.try_wait() {
            lock.1.remove(&key);
        } else {
            if remote_type == "rdp" {
                allow_err!(c.kill());
                std::thread::sleep(std::time::Duration::from_millis(30));
                c.try_wait().ok();
                lock.1.remove(&key);
            } else {
                return;
            }
        }
    }
    match crate::run_me(args) {
        Ok(child) => {
            lock.1.insert(key, child);
        }
        Err(err) => {
            log::error!("Failed to spawn remote: {}", err);
        }
    }
}

#[inline]
pub fn recent_sessions_updated() -> bool {
    let mut children = CHILDREN.lock().unwrap();
    if children.0 {
        children.0 = false;
        true
    } else {
        false
    }
}

pub fn get_icon() -> String {
    // 128x128
    #[cfg(target_os = "macos")]
    // 128x128 on 160x160 canvas, then shrink to 128, mac looks better with padding
    {
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAACxMAAAsTAQCanBgAAAluaVRYdFhNTDpjb20uYWRvYmUueG1wAAAAAAA8P3hwYWNrZXQgYmVnaW49Iu+7vyIgaWQ9Ilc1TTBNcENlaGlIenJlU3pOVGN6a2M5ZCI/PiA8eDp4bXBtZXRhIHhtbG5zOng9ImFkb2JlOm5zOm1ldGEvIiB4OnhtcHRrPSJBZG9iZSBYTVAgQ29yZSA5LjAtYzAwMCA3OS4xNzFjMjdmYWIsIDIwMjIvMDgvMTYtMjI6MzU6NDEgICAgICAgICI+IDxyZGY6UkRGIHhtbG5zOnJkZj0iaHR0cDovL3d3dy53My5vcmcvMTk5OS8wMi8yMi1yZGYtc3ludGF4LW5zIyI+IDxyZGY6RGVzY3JpcHRpb24gcmRmOmFib3V0PSIiIHhtbG5zOnhtcD0iaHR0cDovL25zLmFkb2JlLmNvbS94YXAvMS4wLyIgeG1sbnM6ZGM9Imh0dHA6Ly9wdXJsLm9yZy9kYy9lbGVtZW50cy8xLjEvIiB4bWxuczp4bXBNTT0iaHR0cDovL25zLmFkb2JlLmNvbS94YXAvMS4wL21tLyIgeG1sbnM6c3RFdnQ9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC9zVHlwZS9SZXNvdXJjZUV2ZW50IyIgeG1sbnM6c3RSZWY9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC9zVHlwZS9SZXNvdXJjZVJlZiMiIHhtbG5zOnBob3Rvc2hvcD0iaHR0cDovL25zLmFkb2JlLmNvbS9waG90b3Nob3AvMS4wLyIgeG1wOkNyZWF0b3JUb29sPSJBZG9iZSBQaG90b3Nob3AgMjQuMSAoTWFjaW50b3NoKSIgeG1wOkNyZWF0ZURhdGU9IjIwMjMtMDItMjFUMjM6MDY6MzYrMDE6MDAiIHhtcDpNZXRhZGF0YURhdGU9IjIwMjMtMDItMjFUMjM6MDc6MTErMDE6MDAiIHhtcDpNb2RpZnlEYXRlPSIyMDIzLTAyLTIxVDIzOjA3OjExKzAxOjAwIiBkYzpmb3JtYXQ9ImltYWdlL3BuZyIgeG1wTU06SW5zdGFuY2VJRD0ieG1wLmlpZDpmNDE4MDRlZi1mNjk3LTRjMDctODg3NS01MjY4ZmJiOWI4MzEiIHhtcE1NOkRvY3VtZW50SUQ9ImFkb2JlOmRvY2lkOnBob3Rvc2hvcDoyN2QxM2NmMy01Yjk3LTBlNDUtODVmNy0zYmY0NzRkNDRmMTMiIHhtcE1NOk9yaWdpbmFsRG9jdW1lbnRJRD0ieG1wLmRpZDoyYTA3M2Y5My1mZmRkLTQ4YjYtOWE0MC1iMzU0MjYzOGU5MjUiIHBob3Rvc2hvcDpDb2xvck1vZGU9IjMiPiA8eG1wTU06SGlzdG9yeT4gPHJkZjpTZXE+IDxyZGY6bGkgc3RFdnQ6YWN0aW9uPSJjcmVhdGVkIiBzdEV2dDppbnN0YW5jZUlEPSJ4bXAuaWlkOjJhMDczZjkzLWZmZGQtNDhiNi05YTQwLWIzNTQyNjM4ZTkyNSIgc3RFdnQ6d2hlbj0iMjAyMy0wMi0yMVQyMzowNjozNiswMTowMCIgc3RFdnQ6c29mdHdhcmVBZ2VudD0iQWRvYmUgUGhvdG9zaG9wIDI0LjEgKE1hY2ludG9zaCkiLz4gPHJkZjpsaSBzdEV2dDphY3Rpb249InNhdmVkIiBzdEV2dDppbnN0YW5jZUlEPSJ4bXAuaWlkOjhhZDdjODAwLTBiN2YtNDkxMC1iZjAwLTVmNjI0ZDE5MzVhMyIgc3RFdnQ6d2hlbj0iMjAyMy0wMi0yMVQyMzowNzoxMSswMTowMCIgc3RFdnQ6c29mdHdhcmVBZ2VudD0iQWRvYmUgUGhvdG9zaG9wIDI0LjEgKE1hY2ludG9zaCkiIHN0RXZ0OmNoYW5nZWQ9Ii8iLz4gPHJkZjpsaSBzdEV2dDphY3Rpb249ImNvbnZlcnRlZCIgc3RFdnQ6cGFyYW1ldGVycz0iZnJvbSBhcHBsaWNhdGlvbi92bmQuYWRvYmUucGhvdG9zaG9wIHRvIGltYWdlL3BuZyIvPiA8cmRmOmxpIHN0RXZ0OmFjdGlvbj0iZGVyaXZlZCIgc3RFdnQ6cGFyYW1ldGVycz0iY29udmVydGVkIGZyb20gYXBwbGljYXRpb24vdm5kLmFkb2JlLnBob3Rvc2hvcCB0byBpbWFnZS9wbmciLz4gPHJkZjpsaSBzdEV2dDphY3Rpb249InNhdmVkIiBzdEV2dDppbnN0YW5jZUlEPSJ4bXAuaWlkOmY0MTgwNGVmLWY2OTctNGMwNy04ODc1LTUyNjhmYmI5YjgzMSIgc3RFdnQ6d2hlbj0iMjAyMy0wMi0yMVQyMzowNzoxMSswMTowMCIgc3RFdnQ6c29mdHdhcmVBZ2VudD0iQWRvYmUgUGhvdG9zaG9wIDI0LjEgKE1hY2ludG9zaCkiIHN0RXZ0OmNoYW5nZWQ9Ii8iLz4gPC9yZGY6U2VxPiA8L3htcE1NOkhpc3Rvcnk+IDx4bXBNTTpEZXJpdmVkRnJvbSBzdFJlZjppbnN0YW5jZUlEPSJ4bXAuaWlkOjhhZDdjODAwLTBiN2YtNDkxMC1iZjAwLTVmNjI0ZDE5MzVhMyIgc3RSZWY6ZG9jdW1lbnRJRD0ieG1wLmRpZDoyYTA3M2Y5My1mZmRkLTQ4YjYtOWE0MC1iMzU0MjYzOGU5MjUiIHN0UmVmOm9yaWdpbmFsRG9jdW1lbnRJRD0ieG1wLmRpZDoyYTA3M2Y5My1mZmRkLTQ4YjYtOWE0MC1iMzU0MjYzOGU5MjUiLz4gPHBob3Rvc2hvcDpEb2N1bWVudEFuY2VzdG9ycz4gPHJkZjpCYWc+IDxyZGY6bGk+YWRvYmU6ZG9jaWQ6cGhvdG9zaG9wOjA5ZDUwMDE5LWY2MDYtZDc0Ny1iNzBkLWMxZjkwMDI2NDBiYzwvcmRmOmxpPiA8cmRmOmxpPnhtcC5kaWQ6ZWJjZDFmOGQtZTY5ZS00NzlkLThiMzUtODI5MDcyNjM5ZTk4PC9yZGY6bGk+IDwvcmRmOkJhZz4gPC9waG90b3Nob3A6RG9jdW1lbnRBbmNlc3RvcnM+IDwvcmRmOkRlc2NyaXB0aW9uPiA8L3JkZjpSREY+IDwveDp4bXBtZXRhPiA8P3hwYWNrZXQgZW5kPSJyIj8+OBBWNAAAIAtJREFUeF7tfQmAFNW19tc93dOzL+CwyCKbAvkTfSaazYVoVFAwQUVBFAFRQeVpXGIS9y2+PPPi+5OYGDdcgoIo8rubBDXqc18wMfqzDZvAzLDPPt3Tyzvfqaqe6p5eh24UqK/79L1169bdzqlzz7116zYcOHDgwIEDBw4cOHDgwIEDBw4cOHDgwIEDBw4cOHDgwMG+CNcNP77rGH84eHskHAEpHA4jHBI3EkIkIjHoF5KTEsbzQBgSV471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".into()
    }
    #[cfg(not(target_os = "macos"))] // 128x128 no padding
    {
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAYAAADDPmHLAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAACxMAAAsTAQCanBgAAAluaVRYdFhNTDpjb20uYWRvYmUueG1wAAAAAAA8P3hwYWNrZXQgYmVnaW49Iu+7vyIgaWQ9Ilc1TTBNcENlaGlIenJlU3pOVGN6a2M5ZCI/PiA8eDp4bXBtZXRhIHhtbG5zOng9ImFkb2JlOm5zOm1ldGEvIiB4OnhtcHRrPSJBZG9iZSBYTVAgQ29yZSA5LjAtYzAwMCA3OS4xNzFjMjdmYWIsIDIwMjIvMDgvMTYtMjI6MzU6NDEgICAgICAgICI+IDxyZGY6UkRGIHhtbG5zOnJkZj0iaHR0cDovL3d3dy53My5vcmcvMTk5OS8wMi8yMi1yZGYtc3ludGF4LW5zIyI+IDxyZGY6RGVzY3JpcHRpb24gcmRmOmFib3V0PSIiIHhtbG5zOnhtcD0iaHR0cDovL25zLmFkb2JlLmNvbS94YXAvMS4wLyIgeG1sbnM6ZGM9Imh0dHA6Ly9wdXJsLm9yZy9kYy9lbGVtZW50cy8xLjEvIiB4bWxuczp4bXBNTT0iaHR0cDovL25zLmFkb2JlLmNvbS94YXAvMS4wL21tLyIgeG1sbnM6c3RFdnQ9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC9zVHlwZS9SZXNvdXJjZUV2ZW50IyIgeG1sbnM6c3RSZWY9Imh0dHA6Ly9ucy5hZG9iZS5jb20veGFwLzEuMC9zVHlwZS9SZXNvdXJjZVJlZiMiIHhtbG5zOnBob3Rvc2hvcD0iaHR0cDovL25zLmFkb2JlLmNvbS9waG90b3Nob3AvMS4wLyIgeG1wOkNyZWF0b3JUb29sPSJBZG9iZSBQaG90b3Nob3AgMjQuMSAoTWFjaW50b3NoKSIgeG1wOkNyZWF0ZURhdGU9IjIwMjMtMDItMjFUMjM6MDY6MzYrMDE6MDAiIHhtcDpNZXRhZGF0YURhdGU9IjIwMjMtMDItMjFUMjM6MDc6MTErMDE6MDAiIHhtcDpNb2RpZnlEYXRlPSIyMDIzLTAyLTIxVDIzOjA3OjExKzAxOjAwIiBkYzpmb3JtYXQ9ImltYWdlL3BuZyIgeG1wTU06SW5zdGFuY2VJRD0ieG1wLmlpZDpmNDE4MDRlZi1mNjk3LTRjMDctODg3NS01MjY4ZmJiOWI4MzEiIHhtcE1NOkRvY3VtZW50SUQ9ImFkb2JlOmRvY2lkOnBob3Rvc2hvcDoyN2QxM2NmMy01Yjk3LTBlNDUtODVmNy0zYmY0NzRkNDRmMTMiIHhtcE1NOk9yaWdpbmFsRG9jdW1lbnRJRD0ieG1wLmRpZDoyYTA3M2Y5My1mZmRkLTQ4YjYtOWE0MC1iMzU0MjYzOGU5MjUiIHBob3Rvc2hvcDpDb2xvck1vZGU9IjMiPiA8eG1wTU06SGlzdG9yeT4gPHJkZjpTZXE+IDxyZGY6bGkgc3RFdnQ6YWN0aW9uPSJjcmVhdGVkIiBzdEV2dDppbnN0YW5jZUlEPSJ4bXAuaWlkOjJhMDczZjkzLWZmZGQtNDhiNi05YTQwLWIzNTQyNjM4ZTkyNSIgc3RFdnQ6d2hlbj0iMjAyMy0wMi0yMVQyMzowNjozNiswMTowMCIgc3RFdnQ6c29mdHdhcmVBZ2VudD0iQWRvYmUgUGhvdG9zaG9wIDI0LjEgKE1hY2ludG9zaCkiLz4gPHJkZjpsaSBzdEV2dDphY3Rpb249InNhdmVkIiBzdEV2dDppbnN0YW5jZUlEPSJ4bXAuaWlkOjhhZDdjODAwLTBiN2YtNDkxMC1iZjAwLTVmNjI0ZDE5MzVhMyIgc3RFdnQ6d2hlbj0iMjAyMy0wMi0yMVQyMzowNzoxMSswMTowMCIgc3RFdnQ6c29mdHdhcmVBZ2VudD0iQWRvYmUgUGhvdG9zaG9wIDI0LjEgKE1hY2ludG9zaCkiIHN0RXZ0OmNoYW5nZWQ9Ii8iLz4gPHJkZjpsaSBzdEV2dDphY3Rpb249ImNvbnZlcnRlZCIgc3RFdnQ6cGFyYW1ldGVycz0iZnJvbSBhcHBsaWNhdGlvbi92bmQuYWRvYmUucGhvdG9zaG9wIHRvIGltYWdlL3BuZyIvPiA8cmRmOmxpIHN0RXZ0OmFjdGlvbj0iZGVyaXZlZCIgc3RFdnQ6cGFyYW1ldGVycz0iY29udmVydGVkIGZyb20gYXBwbGljYXRpb24vdm5kLmFkb2JlLnBob3Rvc2hvcCB0byBpbWFnZS9wbmciLz4gPHJkZjpsaSBzdEV2dDphY3Rpb249InNhdmVkIiBzdEV2dDppbnN0YW5jZUlEPSJ4bXAuaWlkOmY0MTgwNGVmLWY2OTctNGMwNy04ODc1LTUyNjhmYmI5YjgzMSIgc3RFdnQ6d2hlbj0iMjAyMy0wMi0yMVQyMzowNzoxMSswMTowMCIgc3RFdnQ6c29mdHdhcmVBZ2VudD0iQWRvYmUgUGhvdG9zaG9wIDI0LjEgKE1hY2ludG9zaCkiIHN0RXZ0OmNoYW5nZWQ9Ii8iLz4gPC9yZGY6U2VxPiA8L3htcE1NOkhpc3Rvcnk+IDx4bXBNTTpEZXJpdmVkRnJvbSBzdFJlZjppbnN0YW5jZUlEPSJ4bXAuaWlkOjhhZDdjODAwLTBiN2YtNDkxMC1iZjAwLTVmNjI0ZDE5MzVhMyIgc3RSZWY6ZG9jdW1lbnRJRD0ieG1wLmRpZDoyYTA3M2Y5My1mZmRkLTQ4YjYtOWE0MC1iMzU0MjYzOGU5MjUiIHN0UmVmOm9yaWdpbmFsRG9jdW1lbnRJRD0ieG1wLmRpZDoyYTA3M2Y5My1mZmRkLTQ4YjYtOWE0MC1iMzU0MjYzOGU5MjUiLz4gPHBob3Rvc2hvcDpEb2N1bWVudEFuY2VzdG9ycz4gPHJkZjpCYWc+IDxyZGY6bGk+YWRvYmU6ZG9jaWQ6cGhvdG9zaG9wOjA5ZDUwMDE5LWY2MDYtZDc0Ny1iNzBkLWMxZjkwMDI2NDBiYzwvcmRmOmxpPiA8cmRmOmxpPnhtcC5kaWQ6ZWJjZDFmOGQtZTY5ZS00NzlkLThiMzUtODI5MDcyNjM5ZTk4PC9yZGY6bGk+IDwvcmRmOkJhZz4gPC9waG90b3Nob3A6RG9jdW1lbnRBbmNlc3RvcnM+IDwvcmRmOkRlc2NyaXB0aW9uPiA8L3JkZjpSREY+IDwveDp4bXBtZXRhPiA8P3hwYWNrZXQgZW5kPSJyIj8+OBBWNAAAIAtJREFUeF7tfQmAFNW19tc93dOzL+CwyCKbAvkTfSaazYVoVFAwQUVBFAFRQeVpXGIS9y2+PPPi+5OYGDdcgoIo8rubBDXqc18wMfqzDZvAzLDPPt3Tyzvfqaqe6p5eh24UqK/79L1169bdzqlzz7116zYcOHDgwIEDBw4cOHDgwIEDBw4cOHDgwIEDBw4cOHDgwMG+CNcNP77rGH84eHskHAEpHA4jHBI3EkIkIjHoF5KTEsbzQBgSV471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".into()
    }
}
