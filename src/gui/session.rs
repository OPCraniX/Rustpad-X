fn load_session_state() -> SessionState {
    let path = session_file_path();
    if std::fs::metadata(&path)
        .map(|metadata| metadata.len() > SESSION_FILE_MAX_BYTES)
        .unwrap_or(false)
    {
        return empty_session_state();
    }

    let Ok(text) = std::fs::read_to_string(path) else {
        return empty_session_state();
    };

    let mut state = empty_session_state();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("active=") {
            state.active_tab = value.parse::<usize>().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("compare_page_sync=") {
            state.compare_page_sync = matches!(value.trim(), "1" | "true" | "yes" | "on");
        } else if let Some(value) = line.strip_prefix("recent=") {
            if !value.trim().is_empty() && state.recent_files.len() < RECENT_FILE_LIMIT {
                state.recent_files.push(PathBuf::from(value));
            }
        } else if let Some(value) = line.strip_prefix("doc=") {
            if state.tabs.len() >= SESSION_MAX_TABS {
                continue;
            }
            if let Some(tab) = parse_session_tab(value) {
                state.tabs.push(tab);
            }
        } else if let Some(value) = line.strip_prefix("tab=") {
            if !value.trim().is_empty() && state.tabs.len() < SESSION_MAX_TABS {
                state.tabs.push(SessionTab::File {
                    path: PathBuf::from(value),
                    text: None,
                    line_ending: None,
                });
            }
        }
    }

    state
}

fn empty_session_state() -> SessionState {
    SessionState {
        recent_files: Vec::new(),
        tabs: Vec::new(),
        active_tab: 0,
        compare_page_sync: false,
    }
}

fn restored_session_documents(
    session: &SessionState,
    use_rich_edit: bool,
) -> (Vec<Document>, usize) {
    let mut documents = Vec::new();
    let mut loaded_paths = Vec::new();
    let mut active_tab = 0;

    for (session_index, tab) in session.tabs.iter().enumerate() {
        let document = match tab {
            SessionTab::File {
                path,
                text,
                line_ending,
            } => {
                let path = normalized_path(path);
                if loaded_paths
                    .iter()
                    .any(|loaded: &PathBuf| paths_match(loaded, &path))
                {
                    continue;
                }

                let document = match text {
                    Some(text) => Document::from_file(
                        path.clone(),
                        to_edit_line_endings(text),
                        line_ending.unwrap_or(LineEnding::Lf),
                    ),
                    None => {
                        let Ok(text) = read_text_file_lossy(&path) else {
                            continue;
                        };
                        Document::from_open_file(path.clone(), text, use_rich_edit)
                    }
                };

                loaded_paths.push(path.clone());
                document
            }
            SessionTab::Untitled {
                name,
                text,
                line_ending,
            } => Document::untitled_with_text(
                name.clone(),
                to_edit_line_endings(text),
                *line_ending,
            ),
        };

        documents.push(document);
        if session_index == session.active_tab {
            active_tab = documents.len() - 1;
        }
    }

    if documents.is_empty() {
        return (vec![Document::untitled(1)], 0);
    }

    (documents, active_tab)
}

fn startup_documents(paths: &[PathBuf], use_rich_edit: bool) -> (Vec<Document>, usize) {
    let mut documents = Vec::new();
    let mut loaded_paths = Vec::new();

    for path in paths {
        let path = normalized_path(path);
        if loaded_paths
            .iter()
            .any(|loaded: &PathBuf| paths_match(loaded, &path))
        {
            continue;
        }

        let Ok(text) = read_text_file_lossy(&path) else {
            continue;
        };

        loaded_paths.push(path.clone());
        documents.push(Document::from_open_file(path, text, use_rich_edit));
    }

    if documents.is_empty() {
        documents.push(Document::untitled(1));
    }

    let active_tab = documents.len().saturating_sub(1);
    (documents, active_tab)
}

fn restored_recent_files(session: &SessionState) -> Vec<PathBuf> {
    let mut recent_files = Vec::new();
    for path in &session.recent_files {
        if !path.exists() {
            continue;
        }

        let path = normalized_path(path);
        if recent_files
            .iter()
            .any(|recent: &PathBuf| paths_match(recent, &path))
        {
            continue;
        }

        recent_files.push(path);
        if recent_files.len() >= RECENT_FILE_LIMIT {
            break;
        }
    }
    recent_files
}

fn read_text_file_lossy(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(decode_text_bytes_lossy(&bytes))
}

fn decode_text_bytes_lossy(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let words = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16_lossy(&words);
    }

    if bytes.starts_with(&[0xFE, 0xFF]) {
        let words = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16_lossy(&words);
    }

    String::from_utf8_lossy(bytes).into_owned()
}

fn save_session_state(app: &mut AppData) {
    if !app.persist_session {
        return;
    }

    if app.edit.is_null() {
        return;
    }

    sync_active_document_text_for_session(app);

    let active_tab = app.active_tab.min(app.documents.len().saturating_sub(1));

    let mut text = String::from("# Rustpad-X session v1\n");
    text.push_str(&format!("active={active_tab}\n"));
    if let Some(window_line) = current_window_placement_line(app) {
        text.push_str(&window_line);
    }
    text.push_str(&format!(
        "compare_page_sync={}\n",
        if app.compare_page_sync { 1 } else { 0 }
    ));
    for path in app.recent_files.iter().take(RECENT_FILE_LIMIT) {
        text.push_str("recent=");
        text.push_str(&path.display().to_string());
        text.push('\n');
    }
    for document in &app.documents {
        if let Some(path) = &document.path {
            if document.text.len() > SESSION_INLINE_TEXT_LIMIT {
                text.push_str("doc=file-ref\t");
                text.push_str(&hex_encode(&path.display().to_string()));
                text.push('\t');
                text.push_str(document.line_ending.session_token());
                text.push('\n');
                continue;
            }

            text.push_str("doc=");
            text.push_str("file\t");
            text.push_str(&hex_encode(&path.display().to_string()));
            text.push('\t');
            text.push_str(&hex_encode(&document.text));
            text.push('\t');
            text.push_str(document.line_ending.session_token());
        } else {
            text.push_str("doc=");
            text.push_str("untitled\t");
            text.push_str(&hex_encode(&document.untitled_name));
            text.push('\t');
            text.push_str(&hex_encode(&document.text));
            text.push('\t');
            text.push_str(document.line_ending.session_token());
        }
        text.push('\n');
    }

    let path = session_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, text);
}

fn session_file_path() -> PathBuf {
    let base = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    base.join(APP_TITLE).join(SESSION_FILE_NAME)
}

fn parse_session_tab(value: &str) -> Option<SessionTab> {
    let parts = value.split('\t').collect::<Vec<_>>();
    let kind = *parts.first()?;
    let first = *parts.get(1)?;
    let second = parts.get(2).copied().unwrap_or("");

    match kind {
        "file" => Some(SessionTab::File {
            path: PathBuf::from(hex_decode(first)?),
            text: if hex_decoded_len(second)? > SESSION_INLINE_TEXT_LIMIT {
                None
            } else {
                Some(hex_decode(second)?)
            },
            line_ending: parts
                .get(3)
                .and_then(|value| LineEnding::from_session_token(value)),
        }),
        "file-ref" => Some(SessionTab::File {
            path: PathBuf::from(hex_decode(first)?),
            text: None,
            line_ending: parts
                .get(2)
                .and_then(|value| LineEnding::from_session_token(value)),
        }),
        "untitled" => Some(SessionTab::Untitled {
            name: hex_decode(first)?,
            text: hex_decode(second)?,
            line_ending: parts
                .get(3)
                .and_then(|value| LineEnding::from_session_token(value))
                .unwrap_or(LineEnding::Lf),
        }),
        _ => None,
    }
}

fn next_available_untitled_index(documents: &[Document]) -> usize {
    let mut index = 1;
    loop {
        let name = format!("Untitled {index}");
        if !documents
            .iter()
            .any(|document| document.path.is_none() && document.untitled_name == name)
        {
            return index;
        }
        index += 1;
    }
}

fn hex_encode(value: &str) -> String {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";

    let bytes = value.as_bytes();
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 0x0F) as usize] as char);
    }
    encoded
}

fn hex_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }

    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_value(pair[0])?;
        let low = hex_value(pair[1])?;
        decoded.push((high << 4) | low);
    }

    String::from_utf8(decoded).ok()
}

fn hex_decoded_len(value: &str) -> Option<usize> {
    if value.len() % 2 == 0 {
        Some(value.len() / 2)
    } else {
        None
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

struct MainMenus {
    menu: Hmenu,
    file_menu: Hmenu,
    edit_menu: Hmenu,
    view_menu: Hmenu,
    help_menu: Hmenu,
    recent_menu: Hmenu,
}

#[derive(Clone, Copy)]
struct MenuBarItem {
    index: usize,
    label: &'static str,
    menu: Hmenu,
}

struct AppDataCell {
    borrow_depth: Cell<u32>,
    data: UnsafeCell<AppData>,
}

impl AppDataCell {
    fn new(data: AppData) -> Self {
        Self {
            borrow_depth: Cell::new(0),
            data: UnsafeCell::new(data),
        }
    }

    fn with_mut<R>(&self, callback: impl FnOnce(&mut AppData) -> R) -> Option<R> {
        if self.borrow_depth.get() != 0 {
            return None;
        }

        self.borrow_depth.set(1);
        let _guard = BorrowGuard {
            borrow_depth: &self.borrow_depth,
        };

        Some(callback(unsafe { &mut *self.data.get() }))
    }
}

struct BorrowGuard<'a> {
    borrow_depth: &'a Cell<u32>,
}

impl Drop for BorrowGuard<'_> {
    fn drop(&mut self) {
        self.borrow_depth.set(0);
    }
}

fn startup_request() -> StartupRequest {
    let mut paths = Vec::new();
    let mut persist_session = true;

    for argument in env::args_os().skip(1) {
        if argument == OsString::from(NEW_WINDOW_ARG) {
            persist_session = false;
        } else {
            paths.push(PathBuf::from(argument));
        }
    }

    if !paths.is_empty() {
        persist_session = false;
    }

    StartupRequest {
        paths,
        persist_session,
    }
}

fn load_rich_edit_library() -> bool {
    let library = to_wide(RICH_EDIT_LIBRARY);
    !unsafe { LoadLibraryW(library.as_ptr()) }.is_null()
}

fn load_app_icon(instance: Hinstance, width: Int, height: Int) -> Hicon {
    unsafe {
        LoadImageW(
            instance,
            make_int_resource(APP_ICON_RESOURCE_ID),
            IMAGE_ICON,
            width,
            height,
            LR_DEFAULTCOLOR | LR_SHARED,
        ) as Hicon
    }
}

fn make_int_resource(id: usize) -> *const u16 {
    id as *const u16
}

fn saved_window_placement() -> Option<(i32, i32, i32, i32)> {
    let text = std::fs::read_to_string(session_file_path()).ok()?;
    for line in text.lines() {
        let Some(value) = line.strip_prefix("window=") else {
            continue;
        };
        let parts = value
            .split(',')
            .filter_map(|part| part.trim().parse::<i32>().ok())
            .collect::<Vec<_>>();
        if parts.len() == 4 {
            let width = parts[2].max(400);
            let height = parts[3].max(300);
            return Some((parts[0], parts[1], width, height));
        }
    }
    None
}

fn current_window_placement_line(app: &AppData) -> Option<String> {
    if app.hwnd.is_null() || app.fullscreen.is_some() {
        return None;
    }

    let mut rect = empty_rect();
    if unsafe { GetWindowRect(app.hwnd, &mut rect) } == 0 {
        return None;
    }

    let width = (rect.right - rect.left).max(400);
    let height = (rect.bottom - rect.top).max(300);
    Some(format!(
        "window={},{},{},{}\n",
        rect.left, rect.top, width, height
    ))
}
