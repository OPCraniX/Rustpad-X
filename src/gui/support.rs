fn show_about(hwnd: Hwnd) {
    message_box(
        hwnd,
        "Rustpad-X v1.0\n\nLow-level native Win32 text editor written in Rust.",
        "About Rustpad-X",
        MB_OK | MB_ICONINFORMATION,
    );
}

fn toggle_line_numbers(app: &mut AppData) {
    app.line_numbers_visible = !app.line_numbers_visible;
    update_line_number_menu_item(app);

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(app.hwnd, &mut rect);
    }
    layout_editor(app, rect.right - rect.left, rect.bottom - rect.top);
    invalidate_gutter(app);
    invalidate_status(app);
}

fn toggle_word_wrap(app: &mut AppData) {
    app.word_wrap_enabled = !app.word_wrap_enabled;
    update_word_wrap_menu_item(app);
    apply_word_wrap_to_edit(app, app.edit);
    apply_word_wrap_to_edit(app, app.compare_edit);
    relayout_current_client(app);
    ensure_gutter_sync(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
    save_session_state(app);
}

fn apply_word_wrap_to_edit(app: &AppData, edit: Hwnd) {
    if edit.is_null() {
        return;
    }

    let style = unsafe { GetWindowLongPtrW(edit, GWL_STYLE) };
    let new_style = if app.word_wrap_enabled {
        style & !WS_HSCROLL & !ES_AUTOHSCROLL
    } else {
        style | WS_HSCROLL | ES_AUTOHSCROLL
    };

    unsafe {
        if new_style != style {
            SetWindowLongPtrW(edit, GWL_STYLE, new_style);
            SetWindowPos(
                edit,
                null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }

        if app.use_rich_edit {
            let wrap_width = if app.word_wrap_enabled { 0 } else { 1 };
            SendMessageW(edit, EM_SETTARGETDEVICE, 0, wrap_width);
        }

        InvalidateRect(edit, null(), 1);
    }
}

fn set_theme(app: &mut AppData, theme: Theme) {
    if app.theme == theme {
        return;
    }

    let new_brush = match create_theme_editor_brush(theme) {
        Ok(brush) => brush,
        Err(error) => {
            message_box(
                app.hwnd,
                &format!("Could not change the color theme:\n\n{error}"),
                "Theme",
                MB_OK | MB_ICONERROR,
            );
            return;
        }
    };

    let old_brush = app.editor_background_brush;
    app.theme = theme;
    app.editor_background_brush = new_brush;

    if !old_brush.is_null() {
        unsafe {
            DeleteObject(old_brush as Hgdiobj);
        }
    }

    update_theme_menu_items(app);
    refresh_theme(app);
}

fn refresh_theme(app: &AppData) {
    apply_native_window_theme(app);
    apply_rich_edit_theme(app);
    unsafe {
        InvalidateRect(app.hwnd, null(), 1);
        InvalidateRect(app.menu_bar, null(), 0);
        InvalidateRect(app.path_bar, null(), 1);
        InvalidateRect(app.tab_bar, null(), 1);
        InvalidateRect(app.edit, null(), 1);
        InvalidateRect(app.compare_edit, null(), 1);
        InvalidateRect(app.compare_splitter, null(), 1);
    }
    invalidate_gutter(app);
    invalidate_status(app);
}

fn update_line_number_menu_item(app: &AppData) {
    let check_state = if app.line_numbers_visible {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };

    unsafe {
        CheckMenuItem(
            app.view_menu,
            ID_VIEW_LINE_NUMBERS as Uint,
            MF_BYCOMMAND | check_state,
        );
        DrawMenuBar(app.hwnd);
        InvalidateRect(app.menu_bar, null(), 0);
    }
}

fn update_word_wrap_menu_item(app: &AppData) {
    let check_state = if app.word_wrap_enabled {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };

    unsafe {
        CheckMenuItem(
            app.view_menu,
            ID_VIEW_WORD_WRAP as Uint,
            MF_BYCOMMAND | check_state,
        );
        DrawMenuBar(app.hwnd);
        InvalidateRect(app.menu_bar, null(), 0);
    }
}

fn update_compare_page_sync_menu_item(app: &AppData) {
    let check_state = if app.compare_page_sync {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };

    unsafe {
        CheckMenuItem(
            app.view_menu,
            ID_VIEW_SYNC_COMPARE_PAGING as Uint,
            MF_BYCOMMAND | check_state,
        );
        DrawMenuBar(app.hwnd);
        InvalidateRect(app.menu_bar, null(), 0);
    }
}

fn update_theme_menu_items(app: &AppData) {
    let light_state = if app.theme == Theme::Light {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };
    let dark_state = if app.theme == Theme::Dark {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };

    unsafe {
        CheckMenuItem(
            app.view_menu,
            ID_VIEW_LIGHT_MODE as Uint,
            MF_BYCOMMAND | light_state,
        );
        CheckMenuItem(
            app.view_menu,
            ID_VIEW_DARK_MODE as Uint,
            MF_BYCOMMAND | dark_state,
        );
        DrawMenuBar(app.hwnd);
        InvalidateRect(app.menu_bar, null(), 0);
    }
}

fn update_recent_files_menu(app: &AppData) {
    unsafe {
        while GetMenuItemCount(app.recent_menu) > 0 {
            DeleteMenu(app.recent_menu, 0, MF_BYPOSITION);
        }
    }

    if app.recent_files.is_empty() {
        let _ = append_menu(
            app.recent_menu,
            MF_STRING | MF_GRAYED,
            0,
            "(No Recent Files)",
        );
    } else {
        for (index, path) in app.recent_files.iter().enumerate() {
            let label = format!("&{} {}", index + 1, menu_label_escape(&display_path(path)));
            let _ = append_menu(
                app.recent_menu,
                MF_STRING,
                (ID_FILE_RECENT_BASE + index as u16) as usize,
                &label,
            );
        }
    }

    unsafe {
        DrawMenuBar(app.hwnd);
        InvalidateRect(app.menu_bar, null(), 0);
    }
}

fn update_window_title(app: &AppData) {
    let title = window_title_for_document(active_document(app), app.running_as_administrator);
    let wide = to_wide(&title);

    unsafe {
        SetWindowTextW(app.hwnd, wide.as_ptr());
    }
}

fn window_title_for_document(document: &Document, running_as_administrator: bool) -> String {
    let document_name = match &document.path {
        Some(path) => display_path(path),
        None => document.display_name(),
    };
    let title = format!("{APP_TITLE} - {document_name}");

    if running_as_administrator {
        format!("Administrator: {title}")
    } else {
        title
    }
}

fn is_process_running_as_administrator() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

fn create_main_menu() -> io::Result<MainMenus> {
    let menu = unsafe { CreateMenu() };
    let file_menu = unsafe { CreatePopupMenu() };
    let recent_menu = unsafe { CreatePopupMenu() };
    let edit_menu = unsafe { CreatePopupMenu() };
    let view_menu = unsafe { CreatePopupMenu() };
    let help_menu = unsafe { CreatePopupMenu() };

    if menu.is_null()
        || file_menu.is_null()
        || recent_menu.is_null()
        || edit_menu.is_null()
        || view_menu.is_null()
        || help_menu.is_null()
    {
        return Err(io::Error::last_os_error());
    }

    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_NEW as usize,
        "Open New &Tab\tCtrl+T",
    )?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_OPEN as usize,
        "&Open...\tCtrl+O",
    )?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_OPEN_PROJECT as usize,
        "Open &Project...\tCtrl+W",
    )?;
    append_menu(file_menu, MF_POPUP, recent_menu as usize, "Recent &Files")?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_CLOSE_TAB as usize,
        "&Close Tab\tCtrl+Shift+T",
    )?;
    append_menu(file_menu, MF_SEPARATOR, 0, "")?;
    append_menu(file_menu, MF_STRING, ID_FILE_SAVE as usize, "&Save\tCtrl+S")?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_SAVE_ALL as usize,
        "Save A&ll\tCtrl+Shift+S",
    )?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_SAVE_AS as usize,
        "Save &As...",
    )?;
    append_menu(file_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_PRINT as usize,
        "&Print...\tCtrl+P",
    )?;
    append_menu(file_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        file_menu,
        MF_STRING,
        ID_FILE_EXIT as usize,
        "E&xit\tCtrl+Shift+X",
    )?;

    append_menu(edit_menu, MF_STRING, ID_EDIT_UNDO as usize, "&Undo\tCtrl+Z")?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_REDO as usize,
        "&Redo\tCtrl+Shift+Z",
    )?;
    append_menu(edit_menu, MF_SEPARATOR, 0, "")?;
    append_menu(edit_menu, MF_STRING, ID_EDIT_CUT as usize, "Cu&t\tCtrl+X")?;
    append_menu(edit_menu, MF_STRING, ID_EDIT_COPY as usize, "&Copy\tCtrl+C")?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_PASTE as usize,
        "&Paste\tCtrl+V",
    )?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_SELECT_ALL as usize,
        "Select &All\tCtrl+A",
    )?;
    append_menu(edit_menu, MF_SEPARATOR, 0, "")?;
    append_menu(edit_menu, MF_STRING, ID_EDIT_FIND as usize, "&Find\tCtrl+F")?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_FIND_NEXT as usize,
        "Find &Next\tF3",
    )?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_FIND_PREVIOUS as usize,
        "Find Pre&vious\tShift+F3",
    )?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_REPLACE as usize,
        "&Replace\tCtrl+R",
    )?;
    append_menu(edit_menu, MF_SEPARATOR, 0, "")?;
    append_menu(edit_menu, MF_STRING, ID_EDIT_DATE as usize, "&Date\tCtrl+D")?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_TIME_DATE as usize,
        "Time/Da&te\tCtrl+Shift+D",
    )?;
    append_menu(
        edit_menu,
        MF_STRING,
        ID_EDIT_FONT as usize,
        "Fon&t\tCtrl+Shift+F",
    )?;

    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_FULLSCREEN as usize,
        "Full &Screen\tF11",
    )?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_NEXT_TAB as usize,
        "&Switch Tab\tCtrl+Tab",
    )?;
    append_menu(view_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_GOTO_LINE as usize,
        "&Go To Line\tCtrl+G",
    )?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_TOP_OF_DOCUMENT as usize,
        "Top of &Document\tCtrl+Pg Up",
    )?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_BOTTOM_OF_DOCUMENT as usize,
        "&Bottom of Document\tCtrl+Pg Dn",
    )?;
    append_menu(view_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_COMPARE_TABS as usize,
        "&Compare Tabs\tCtrl+Q",
    )?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_SYNC_COMPARE_PAGING as usize,
        "Sync PgUp/PgDn in Compare (Persistent)\tCtrl+Alt+Y",
    )?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_CLOSE_COMPARE_TABS as usize,
        "Close Compare Tabs\tCtrl+Shift+X",
    )?;
    append_menu(view_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        view_menu,
        MF_STRING | MF_CHECKED,
        ID_VIEW_WORD_WRAP as usize,
        "&Word Wrap",
    )?;
    append_menu(
        view_menu,
        MF_STRING | MF_CHECKED,
        ID_VIEW_LINE_NUMBERS as usize,
        "&Line Numbers",
    )?;
    append_menu(view_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_LIGHT_MODE as usize,
        "&Light Mode",
    )?;
    append_menu(
        view_menu,
        MF_STRING | MF_CHECKED,
        ID_VIEW_DARK_MODE as usize,
        "&Dark Mode",
    )?;
    append_menu(view_menu, MF_SEPARATOR, 0, "")?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_ZOOM_IN as usize,
        "Zoom In\tCtrl++",
    )?;
    append_menu(
        view_menu,
        MF_STRING,
        ID_VIEW_ZOOM_OUT as usize,
        "Zoom Out\tCtrl+-",
    )?;

    append_menu(help_menu, MF_STRING, ID_HELP_ABOUT as usize, "&About\tF1")?;

    append_menu(menu, MF_POPUP, file_menu as usize, "&File")?;
    append_menu(menu, MF_POPUP, edit_menu as usize, "&Edit")?;
    append_menu(menu, MF_POPUP, view_menu as usize, "&View")?;
    append_menu(menu, MF_POPUP, help_menu as usize, "&Help")?;

    append_menu(recent_menu, MF_STRING | MF_GRAYED, 0, "(No Recent Files)")?;

    Ok(MainMenus {
        menu,
        file_menu,
        edit_menu,
        view_menu,
        help_menu,
        recent_menu,
    })
}

fn append_menu(menu: Hmenu, flags: Uint, id: usize, label: &str) -> io::Result<()> {
    let label = to_wide(label);
    if unsafe { AppendMenuW(menu, flags, id, label.as_ptr()) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn menu_label_escape(label: &str) -> String {
    label.replace('&', "&&")
}

fn display_path(path: &Path) -> String {
    let value = path.display().to_string();
    if let Some(stripped) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", stripped)
    } else if let Some(stripped) = value.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        value
    }
}

fn create_accelerators() -> Haccel {
    let mut accelerators = [
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'T' as Word,
            cmd: ID_FILE_NEW,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'O' as Word,
            cmd: ID_FILE_OPEN,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'P' as Word,
            cmd: ID_FILE_PRINT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'W' as Word,
            cmd: ID_FILE_OPEN_PROJECT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'S' as Word,
            cmd: ID_FILE_SAVE,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: b'S' as Word,
            cmd: ID_FILE_SAVE_ALL,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: b'T' as Word,
            cmd: ID_FILE_CLOSE_TAB,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'Q' as Word,
            cmd: ID_VIEW_COMPARE_TABS,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FALT,
            key: b'Y' as Word,
            cmd: ID_VIEW_SYNC_COMPARE_PAGING,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'Z' as Word,
            cmd: ID_EDIT_UNDO,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: b'Z' as Word,
            cmd: ID_EDIT_REDO,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'X' as Word,
            cmd: ID_EDIT_CUT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'C' as Word,
            cmd: ID_EDIT_COPY,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'V' as Word,
            cmd: ID_EDIT_PASTE,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'A' as Word,
            cmd: ID_EDIT_SELECT_ALL,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'F' as Word,
            cmd: ID_EDIT_FIND,
        },
        Accel {
            fVirt: FVIRTKEY,
            key: VK_F3,
            cmd: ID_EDIT_FIND_NEXT,
        },
        Accel {
            fVirt: FVIRTKEY | FSHIFT,
            key: VK_F3,
            cmd: ID_EDIT_FIND_PREVIOUS,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'R' as Word,
            cmd: ID_EDIT_REPLACE,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'D' as Word,
            cmd: ID_EDIT_DATE,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: b'D' as Word,
            cmd: ID_EDIT_TIME_DATE,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: b'F' as Word,
            cmd: ID_EDIT_FONT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_TAB,
            cmd: ID_VIEW_NEXT_TAB,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: VK_TAB,
            cmd: ID_VIEW_PREVIOUS_TAB,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: b'G' as Word,
            cmd: ID_VIEW_GOTO_LINE,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_PRIOR,
            cmd: ID_VIEW_TOP_OF_DOCUMENT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_NEXT,
            cmd: ID_VIEW_BOTTOM_OF_DOCUMENT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL | FSHIFT,
            key: b'X' as Word,
            cmd: ID_FILE_EXIT,
        },
        Accel {
            fVirt: FVIRTKEY,
            key: VK_F11,
            cmd: ID_VIEW_FULLSCREEN,
        },
        Accel {
            fVirt: FVIRTKEY,
            key: VK_F1,
            cmd: ID_HELP_ABOUT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_OEM_PLUS,
            cmd: ID_VIEW_ZOOM_IN,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_ADD,
            cmd: ID_VIEW_ZOOM_IN,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_OEM_MINUS,
            cmd: ID_VIEW_ZOOM_OUT,
        },
        Accel {
            fVirt: FVIRTKEY | FCONTROL,
            key: VK_SUBTRACT,
            cmd: ID_VIEW_ZOOM_OUT,
        },
    ];

    unsafe { CreateAcceleratorTableW(accelerators.as_mut_ptr(), accelerators.len() as Int) }
}

fn prompt_for_open_path(hwnd: Hwnd) -> Option<PathBuf> {
    let mut file_buffer = vec![0u16; 32_768];
    let filter = file_filter();
    let title = to_wide("Open");

    let mut dialog = open_file_name(hwnd, &filter, &title, &mut file_buffer);
    dialog.Flags |= OFN_FILEMUSTEXIST;

    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        return None;
    }

    wide_buffer_to_path(&file_buffer)
}

fn prompt_for_project_source_path(hwnd: Hwnd) -> Option<PathBuf> {
    let mut file_buffer = vec![0u16; 32_768];
    let filter = source_file_filter();
    let title = to_wide("Open Project");

    let mut dialog = open_file_name(hwnd, &filter, &title, &mut file_buffer);
    dialog.Flags |= OFN_FILEMUSTEXIST;

    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        return None;
    }

    wide_buffer_to_path(&file_buffer)
}

fn prompt_for_save_path(hwnd: Hwnd, current_path: Option<&Path>) -> Option<PathBuf> {
    let mut file_buffer = vec![0u16; 32_768];
    if let Some(path) = current_path {
        fill_wide_buffer(path, &mut file_buffer);
    }

    let filter = file_filter();
    let title = to_wide("Save As");
    let default_ext = to_wide("txt");

    let mut dialog = open_file_name(hwnd, &filter, &title, &mut file_buffer);
    dialog.lpstrDefExt = default_ext.as_ptr();
    dialog.Flags |= OFN_OVERWRITEPROMPT;

    if unsafe { GetSaveFileNameW(&mut dialog) } == 0 {
        return None;
    }

    wide_buffer_to_path(&file_buffer)
}

fn open_file_name<'a>(
    hwnd: Hwnd,
    filter: &'a [u16],
    title: &'a [u16],
    file_buffer: &'a mut [u16],
) -> OpenFileNameW {
    OpenFileNameW {
        lStructSize: size_of::<OpenFileNameW>() as Dword,
        hwndOwner: hwnd,
        hInstance: null_mut(),
        lpstrFilter: filter.as_ptr(),
        lpstrCustomFilter: null_mut(),
        nMaxCustFilter: 0,
        nFilterIndex: 1,
        lpstrFile: file_buffer.as_mut_ptr(),
        nMaxFile: file_buffer.len() as Dword,
        lpstrFileTitle: null_mut(),
        nMaxFileTitle: 0,
        lpstrInitialDir: null(),
        lpstrTitle: title.as_ptr(),
        Flags: OFN_EXPLORER | OFN_HIDEREADONLY | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST,
        nFileOffset: 0,
        nFileExtension: 0,
        lpstrDefExt: null(),
        lCustData: 0,
        lpfnHook: null_mut(),
        lpTemplateName: null(),
        pvReserved: null_mut(),
        dwReserved: 0,
        FlagsEx: 0,
    }
}

fn file_filter() -> Vec<u16> {
    to_wide(
        "Text / Code Documents\0*.txt;*.text;*.log;*.md;*.markdown;*.rst;*.csv;*.tsv;*.json;*.jsonc;*.xml;*.html;*.htm;*.css;*.scss;*.sass;*.less;*.yaml;*.yml;*.toml;*.ini;*.inf;*.cfg;*.conf;*.config;*.properties;*.env;*.gitignore;*.gitattributes;*.editorconfig;*.rs;*.py;*.pyw;*.vbs;*.vb;*.bas;*.js;*.jsx;*.ts;*.tsx;*.java;*.kt;*.kts;*.c;*.h;*.cpp;*.cxx;*.cc;*.hpp;*.cs;*.go;*.php;*.rb;*.lua;*.swift;*.sql;*.sh;*.bash;*.zsh;*.ps1;*.bat;*.cmd;*.pl;*.pm;*.r;*.m;*.mm;*.scala;*.dart;*.ex;*.exs;*.erl;*.hrl;*.fs;*.fsx;*.fsi;*.clj;*.cljs;*.hs;*.lhs;*.ml;*.mli;*.nim;*.zig;*.vue;*.svelte;*.astro;*.dockerfile;Dockerfile;*.make;Makefile;*.cmake;CMakeLists.txt;*.gradle;*.mod;*.sum;*.lock;*.rtf;*.doc;*.docx\0INI / Config Files\0*.ini;*.inf;*.cfg;*.conf;*.config;*.properties;*.env;*.toml;*.yaml;*.yml\0Source Files\0*.rs;*.py;*.pyw;*.vbs;*.vb;*.bas;*.js;*.jsx;*.ts;*.tsx;*.java;*.kt;*.kts;*.c;*.h;*.cpp;*.cxx;*.cc;*.hpp;*.cs;*.go;*.php;*.rb;*.lua;*.swift;*.sql;*.sh;*.bash;*.zsh;*.ps1;*.bat;*.cmd;*.pl;*.pm;*.r;*.scala;*.dart\0Rich/Office Files (raw text only)\0*.rtf;*.doc;*.docx\0All Files (*.*)\0*.*\0",
    )
}

fn source_file_filter() -> Vec<u16> {
    to_wide(
        "Source / Config Files\0*.rs;*.java;*.kt;*.kts;*.py;*.pyw;*.vbs;*.vb;*.bas;*.js;*.jsx;*.ts;*.tsx;*.c;*.h;*.cpp;*.cxx;*.cc;*.hpp;*.cs;*.go;*.php;*.rb;*.lua;*.swift;*.sql;*.sh;*.bash;*.zsh;*.ps1;*.bat;*.cmd;*.pl;*.pm;*.r;*.scala;*.dart;*.json;*.jsonc;*.xml;*.html;*.htm;*.css;*.scss;*.yaml;*.yml;*.toml;*.ini;*.cfg;*.conf;*.properties;*.env;*.txt;*.md\0All Files (*.*)\0*.*\0",
    )
}

fn project_files_for_source(source: &Path) -> Vec<PathBuf> {
    let root = project_root_for_source(source);
    let search_root = project_source_root_for_source(source).unwrap_or(root);
    let source_extension = file_extension(source);
    let mut paths = Vec::new();

    push_unique_path(&mut paths, source);
    collect_project_files(&search_root, &mut paths, source_extension.as_deref());

    paths.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());

    if let Some(index) = paths.iter().position(|path| paths_match(path, source)) {
        let selected = paths.remove(index);
        paths.insert(0, selected);
    }

    if paths.len() > MAX_PROJECT_FILES {
        paths.truncate(MAX_PROJECT_FILES);
    }

    paths
}

fn project_source_root_for_source(source: &Path) -> Option<PathBuf> {
    let parent = source.parent()?;
    if let Some(root) = ancestor_matching_suffix(parent, &["src", "main", "java"]) {
        return Some(root);
    }
    if let Some(root) = ancestor_matching_suffix(parent, &["src", "test", "java"]) {
        return Some(root);
    }
    if let Some(root) = ancestor_matching_suffix(parent, &["src", "main", "kotlin"]) {
        return Some(root);
    }
    if let Some(root) = ancestor_matching_suffix(parent, &["src", "test", "kotlin"]) {
        return Some(root);
    }
    if let Some(root) = ancestor_named(parent, "src") {
        return Some(root);
    }

    None
}

fn ancestor_matching_suffix(path: &Path, suffix: &[&str]) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| path_ends_with_components(ancestor, suffix))
        .map(Path::to_path_buf)
}

fn ancestor_named(path: &Path, name: &str) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| {
            ancestor
                .file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|component| component.eq_ignore_ascii_case(name))
        })
        .map(Path::to_path_buf)
}

fn path_ends_with_components(path: &Path, suffix: &[&str]) -> bool {
    let components = path
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();

    components.len() >= suffix.len()
        && components[components.len() - suffix.len()..]
            .iter()
            .zip(suffix.iter())
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn collect_project_files(dir: &Path, paths: &mut Vec<PathBuf>, source_extension: Option<&str>) {
    if paths.len() >= MAX_PROJECT_FILES {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.path().to_string_lossy().to_ascii_lowercase());

    for entry in entries {
        if paths.len() >= MAX_PROJECT_FILES {
            return;
        }

        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            if !is_skipped_project_dir(&path) {
                collect_project_files(&path, paths, source_extension);
            }
        } else if file_type.is_file() && is_matching_source_file(&path, source_extension) {
            push_unique_path(paths, &path);
        }
    }
}

fn project_root_for_source(source: &Path) -> PathBuf {
    let start = source
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut current = start.clone();

    loop {
        if is_project_root(&current) {
            return current;
        }

        if !current.pop() {
            return start;
        }
    }
}

fn is_project_root(dir: &Path) -> bool {
    [
        ".git",
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
        "settings.gradle",
        "composer.json",
        "Gemfile",
    ]
    .iter()
    .any(|marker| dir.join(marker).exists())
        || std::fs::read_dir(dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .any(|entry| {
                let path = entry.path();
                has_extension(&path, &["sln", "csproj", "vcxproj", "fsproj"])
            })
}

fn is_skipped_project_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };

    [
        ".git",
        ".hg",
        ".svn",
        "target",
        "node_modules",
        ".venv",
        "venv",
        "build",
        "dist",
        "bin",
        "obj",
        "__pycache__",
    ]
    .iter()
    .any(|skip| name.eq_ignore_ascii_case(skip))
}

fn is_matching_source_file(path: &Path, source_extension: Option<&str>) -> bool {
    if let Some(source_extension) = source_extension {
        return is_project_related_file(path, source_extension);
    }

    let Some(extension) = file_extension(path) else {
        return is_extensionless_text_file(path);
    };

    is_text_or_code_extension(&extension)
}

fn is_project_related_file(path: &Path, source_extension: &str) -> bool {
    let Some(extension) = file_extension(path) else {
        return is_extensionless_project_file(path, source_extension);
    };

    if extension.eq_ignore_ascii_case(source_extension) {
        return true;
    }

    let extension = extension.to_ascii_lowercase();
    let source_extension = source_extension.to_ascii_lowercase();

    related_extensions_for_project(&source_extension)
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        || is_named_project_file(path, &source_extension)
}

fn related_extensions_for_project(source_extension: &str) -> &'static [&'static str] {
    match source_extension {
        "java" => &["java", "gradle", "properties", "xml"],
        "kt" | "kts" => &["kt", "kts", "gradle", "properties", "xml"],
        "rs" => &["rs", "toml", "lock"],
        "py" | "pyw" => &["py", "pyw", "toml", "ini", "cfg", "txt", "md"],
        "js" | "jsx" | "ts" | "tsx" => &[
            "js", "jsx", "ts", "tsx", "json", "jsonc", "css", "scss", "html", "htm", "vue",
            "svelte", "astro", "md",
        ],
        "c" | "h" | "cpp" | "cxx" | "cc" | "hpp" => {
            &["c", "h", "cpp", "cxx", "cc", "hpp", "cmake", "txt"]
        }
        "cs" => &["cs", "csproj", "sln", "json", "config", "xml"],
        "go" => &["go", "mod", "sum"],
        "php" => &["php", "json", "lock", "env", "ini"],
        "rb" => &["rb", "gemspec", "lock", "yml", "yaml"],
        "lua" => &["lua", "rockspec"],
        "swift" => &["swift"],
        "sql" => &["sql"],
        "sh" | "bash" | "zsh" => &["sh", "bash", "zsh", "env"],
        "ps1" => &["ps1", "psm1", "psd1"],
        "bat" | "cmd" => &["bat", "cmd"],
        "vbs" | "vb" | "bas" => &["vbs", "vb", "bas"],
        "html" | "htm" => &[
            "html", "htm", "css", "scss", "sass", "less", "js", "ts", "json",
        ],
        "css" | "scss" | "sass" | "less" => {
            &["css", "scss", "sass", "less", "html", "htm", "js", "ts"]
        }
        "json" | "jsonc" => &["json", "jsonc"],
        "xml" => &["xml", "xsd", "xsl", "xslt"],
        "yaml" | "yml" => &["yaml", "yml"],
        "toml" => &["toml", "lock"],
        "ini" | "cfg" | "conf" | "config" | "properties" | "env" => {
            &["ini", "cfg", "conf", "config", "properties", "env"]
        }
        "md" | "markdown" | "rst" | "txt" | "text" | "log" => {
            &["md", "markdown", "rst", "txt", "text", "log"]
        }
        _ => &[],
    }
}

fn is_named_project_file(path: &Path, source_extension: &str) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };

    let name = name.to_ascii_lowercase();
    match source_extension {
        "java" | "kt" | "kts" => matches!(
            name.as_str(),
            "pom.xml"
                | "build.gradle"
                | "build.gradle.kts"
                | "settings.gradle"
                | "settings.gradle.kts"
                | "gradle.properties"
        ),
        "rs" => matches!(name.as_str(), "cargo.toml" | "cargo.lock"),
        "py" | "pyw" => matches!(
            name.as_str(),
            "pyproject.toml"
                | "requirements.txt"
                | "setup.py"
                | "setup.cfg"
                | "tox.ini"
                | "pytest.ini"
                | ".env"
        ),
        "js" | "jsx" | "ts" | "tsx" => matches!(
            name.as_str(),
            "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "tsconfig.json"
                | "vite.config.js"
                | "vite.config.ts"
                | "webpack.config.js"
        ),
        "c" | "h" | "cpp" | "cxx" | "cc" | "hpp" => {
            matches!(name.as_str(), "cmakelists.txt" | "makefile")
        }
        "go" => matches!(name.as_str(), "go.mod" | "go.sum"),
        "php" => matches!(name.as_str(), "composer.json" | "composer.lock" | ".env"),
        "rb" => matches!(name.as_str(), "gemfile" | "gemfile.lock"),
        _ => false,
    }
}

fn is_extensionless_project_file(path: &Path, source_extension: &str) -> bool {
    if !is_extensionless_text_file(path) {
        return false;
    }

    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };

    let name = name.to_ascii_lowercase();
    match source_extension.to_ascii_lowercase().as_str() {
        "js" | "jsx" | "ts" | "tsx" => matches!(name.as_str(), ".env"),
        "py" | "pyw" => matches!(name.as_str(), ".env"),
        "php" => matches!(name.as_str(), ".env"),
        "c" | "h" | "cpp" | "cxx" | "cc" | "hpp" => matches!(name.as_str(), "makefile"),
        _ => false,
    }
}

fn is_text_or_code_extension(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "txt"
            | "text"
            | "log"
            | "md"
            | "markdown"
            | "rst"
            | "csv"
            | "tsv"
            | "json"
            | "jsonc"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "scss"
            | "sass"
            | "less"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "inf"
            | "cfg"
            | "conf"
            | "config"
            | "properties"
            | "env"
            | "editorconfig"
            | "rs"
            | "py"
            | "pyw"
            | "vbs"
            | "vb"
            | "bas"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "java"
            | "kt"
            | "kts"
            | "c"
            | "h"
            | "cpp"
            | "cxx"
            | "cc"
            | "hpp"
            | "cs"
            | "go"
            | "php"
            | "rb"
            | "lua"
            | "swift"
            | "sql"
            | "sh"
            | "bash"
            | "zsh"
            | "ps1"
            | "bat"
            | "cmd"
            | "pl"
            | "pm"
            | "r"
            | "m"
            | "mm"
            | "scala"
            | "dart"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "fs"
            | "fsx"
            | "fsi"
            | "clj"
            | "cljs"
            | "hs"
            | "lhs"
            | "ml"
            | "mli"
            | "nim"
            | "zig"
            | "vue"
            | "svelte"
            | "astro"
            | "dockerfile"
            | "make"
            | "cmake"
            | "gradle"
            | "mod"
            | "sum"
            | "lock"
            | "rtf"
            | "doc"
            | "docx"
    )
}

fn is_extensionless_text_file(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "dockerfile"
                    | "makefile"
                    | "cmakelists.txt"
                    | ".gitignore"
                    | ".gitattributes"
                    | ".editorconfig"
                    | ".env"
            )
        })
}

fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|extension| extension.to_ascii_lowercase())
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    let Some(extension) = path.extension().and_then(OsStr::to_str) else {
        return false;
    };

    extensions
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: &Path) {
    let normalized = normalized_path(path);
    if !paths
        .iter()
        .any(|existing| paths_match(existing, &normalized))
    {
        paths.push(normalized);
    }
}

fn normalized_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn paths_match(left: &Path, right: &Path) -> bool {
    normalized_path(left)
        .to_string_lossy()
        .eq_ignore_ascii_case(&normalized_path(right).to_string_lossy())
}

fn fill_wide_buffer(path: &Path, buffer: &mut [u16]) {
    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let count = path_wide.len().min(buffer.len());
    buffer[..count].copy_from_slice(&path_wide[..count]);
}

fn wide_buffer_to_path(buffer: &[u16]) -> Option<PathBuf> {
    let end = buffer.iter().position(|value| *value == 0)?;
    if end == 0 {
        return None;
    }

    Some(PathBuf::from(OsString::from_wide(&buffer[..end])))
}

fn to_edit_line_endings(text: &str) -> String {
    let line_break = "\r\n";
    normalize_line_endings(text, line_break)
}

fn to_editor_insert_line_endings(text: &str) -> String {
    to_edit_line_endings(text)
}

fn from_edit_line_endings(text: &str) -> String {
    normalize_line_endings(text, "\n")
}

fn from_edit_line_endings_with(text: &str, line_ending: LineEnding) -> String {
    normalize_line_endings(text, line_ending.save_sequence())
}

fn normalize_line_endings(text: &str, line_break: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut segment_start = 0;
    let mut index = 0;

    while index < bytes.len() {
        let newline_end = match bytes[index] {
            b'\r' => {
                let next = index + 1;
                if next < bytes.len() && bytes[next] == b'\n' {
                    next + 1
                } else if next < bytes.len() && bytes[next] == b'\r' {
                    let mut run_end = next;
                    while run_end < bytes.len() && bytes[run_end] == b'\r' {
                        run_end += 1;
                    }
                    if run_end < bytes.len() && bytes[run_end] == b'\n' {
                        run_end + 1
                    } else {
                        next
                    }
                } else {
                    next
                }
            }
            b'\n' => index + 1,
            _ => {
                index += 1;
                continue;
            }
        };

        normalized.push_str(&text[segment_start..index]);
        normalized.push_str(line_break);
        index = newline_end;
        segment_start = index;
    }

    normalized.push_str(&text[segment_start..]);
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ending_normalization_collapses_cr_cr_lf() {
        let text = "alpha\r\r\nbeta\r\r\n\r\r\ngamma";

        assert_eq!(normalize_line_endings(text, "\n"), "alpha\nbeta\n\ngamma");
        assert_eq!(normalize_line_endings(text, "\r"), "alpha\rbeta\r\rgamma");
    }

    #[test]
    fn logical_line_count_counts_text_lines_only() {
        assert_eq!(logical_line_count(""), 1);
        assert_eq!(logical_line_count("one"), 1);
        assert_eq!(logical_line_count("one\r"), 2);
        assert_eq!(logical_line_count("one\r\ntwo\nthree"), 3);
    }

    #[test]
    fn detects_document_line_endings() {
        assert_eq!(detect_line_ending(""), LineEnding::Lf);
        assert_eq!(detect_line_ending("one\ntwo"), LineEnding::Lf);
        assert_eq!(detect_line_ending("one\r\ntwo"), LineEnding::Crlf);
        assert_eq!(detect_line_ending("one\rtwo"), LineEnding::Cr);
        assert_eq!(detect_line_ending("one\r\ntwo\nthree"), LineEnding::Mixed);
    }

    #[test]
    fn saves_with_requested_line_ending() {
        assert_eq!(
            from_edit_line_endings_with("one\rtwo", LineEnding::Crlf),
            "one\r\ntwo"
        );
        assert_eq!(
            from_edit_line_endings_with("one\rtwo", LineEnding::Lf),
            "one\ntwo"
        );
    }

    #[test]
    fn default_session_enables_word_wrap() {
        assert!(empty_session_state().word_wrap_enabled);
    }

    #[test]
    fn project_open_anchors_on_selected_source_tree() {
        let source =
            Path::new(r"C:\repo\BlockSigningPlugin\src\main\java\org\example\Plugin.java");

        assert_eq!(
            project_source_root_for_source(source),
            Some(PathBuf::from(r"C:\repo\BlockSigningPlugin\src\main\java"))
        );
    }

    #[test]
    fn project_open_uses_nearest_src_for_general_sources() {
        let source = Path::new(r"C:\repo\tool\src\commands\build.rs");

        assert_eq!(
            project_source_root_for_source(source),
            Some(PathBuf::from(r"C:\repo\tool\src"))
        );
    }

    #[test]
    fn window_title_uses_untitled_name_and_admin_prefix() {
        let document = Document::untitled(1);

        assert_eq!(
            window_title_for_document(&document, false),
            "Rustpad-X - Untitled 1"
        );
        assert_eq!(
            window_title_for_document(&document, true),
            "Administrator: Rustpad-X - Untitled 1"
        );
    }

    #[test]
    fn window_title_uses_open_file_path() {
        let document = Document::from_file(
            PathBuf::from(r"C:\repo\sample.rs"),
            String::new(),
            LineEnding::Lf,
        );

        assert_eq!(
            window_title_for_document(&document, true),
            r"Administrator: Rustpad-X - C:\repo\sample.rs"
        );
    }
}

fn toggle_fullscreen(app: &mut AppData) {
    if let Some(snapshot) = app.fullscreen.take() {
        unsafe {
            SetWindowLongPtrW(app.hwnd, GWL_STYLE, snapshot.style);
            SetWindowPos(
                app.hwnd,
                null_mut(),
                snapshot.rect.left,
                snapshot.rect.top,
                snapshot.rect.right - snapshot.rect.left,
                snapshot.rect.bottom - snapshot.rect.top,
                SWP_NOOWNERZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        }
        relayout_current_client(app);
        return;
    }

    let Some(monitor_rect) = monitor_rect_for_window(app.hwnd) else {
        return;
    };

    let mut rect = empty_rect();
    if unsafe { GetWindowRect(app.hwnd, &mut rect) } == 0 {
        return;
    }

    let style = unsafe { GetWindowLongPtrW(app.hwnd, GWL_STYLE) };
    app.fullscreen = Some(FullscreenSnapshot { rect, style });

    unsafe {
        SetWindowLongPtrW(app.hwnd, GWL_STYLE, style & !WS_OVERLAPPEDWINDOW);
        SetWindowPos(
            app.hwnd,
            null_mut(),
            monitor_rect.left,
            monitor_rect.top,
            monitor_rect.right - monitor_rect.left,
            monitor_rect.bottom - monitor_rect.top,
            SWP_NOOWNERZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        );
    }
    relayout_current_client(app);
}

fn relayout_current_client(app: &AppData) {
    let mut rect = empty_rect();
    unsafe {
        if GetClientRect(app.hwnd, &mut rect) != 0 {
            layout_editor(app, rect.right - rect.left, rect.bottom - rect.top);
            InvalidateRect(app.hwnd, null(), 1);
            UpdateWindow(app.hwnd);
        }
    }
}

fn centered_window_position(width: i32, height: i32) -> (i32, i32) {
    let point = Point { x: 0, y: 0 };
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTOPRIMARY) };

    if monitor.is_null() {
        return (0, 0);
    }

    let mut info = MonitorInfo {
        cbSize: size_of::<MonitorInfo>() as Dword,
        rcMonitor: empty_rect(),
        rcWork: empty_rect(),
        dwFlags: 0,
    };

    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return (0, 0);
    }

    let work = info.rcWork;
    let x = work.left + ((work.right - work.left - width).max(0) / 2);
    let y = work.top + ((work.bottom - work.top - height).max(0) / 2);

    (x, y)
}

fn monitor_rect_for_window(hwnd: Hwnd) -> Option<Rect> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };

    if monitor.is_null() {
        return None;
    }

    let mut info = MonitorInfo {
        cbSize: size_of::<MonitorInfo>() as Dword,
        rcMonitor: empty_rect(),
        rcWork: empty_rect(),
        dwFlags: 0,
    };

    let ok = unsafe { GetMonitorInfoW(monitor, &mut info) };
    if ok == 0 { None } else { Some(info.rcMonitor) }
}

fn with_app_data<R>(hwnd: Hwnd, callback: impl FnOnce(&mut AppData) -> R) -> Option<R> {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppDataCell };
    if ptr.is_null() {
        return None;
    }

    unsafe { (&*ptr).with_mut(callback) }
}

fn app_data_ptr(hwnd: Hwnd) -> *mut AppData {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppDataCell };
    if ptr.is_null() {
        null_mut()
    } else {
        unsafe { (*ptr).data.get() }
    }
}

fn translate_find_replace_dialog_message(hwnd: Hwnd, message: &mut Msg) -> bool {
    let (find_dialog, replace_dialog) =
        with_app_data(hwnd, |app| (app.find_dialog, app.replace_dialog))
            .unwrap_or((null_mut(), null_mut()));

    if !find_dialog.is_null() && unsafe { IsDialogMessageW(find_dialog, message) } != 0 {
        return true;
    }

    if !replace_dialog.is_null() && unsafe { IsDialogMessageW(replace_dialog, message) } != 0 {
        return true;
    }

    false
}

fn message_box(hwnd: Hwnd, message: &str, title: &str, flags: Uint) {
    let message = to_wide(message);
    let title = to_wide(title);
    unsafe {
        MessageBoxW(hwnd, message.as_ptr(), title.as_ptr(), flags);
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

fn low_word(value: usize) -> u16 {
    (value & 0xffff) as u16
}

fn high_word(value: usize) -> u16 {
    ((value >> 16) & 0xffff) as u16
}

fn signed_low_word(value: usize) -> i32 {
    ((value & 0xffff) as u16 as i16) as i32
}

fn signed_high_word(value: usize) -> i32 {
    (((value >> 16) & 0xffff) as u16 as i16) as i32
}

fn point_lparam(x: i32, y: i32) -> Lparam {
    ((x as u16 as usize) | ((y as u16 as usize) << 16)) as Lparam
}

fn empty_rect() -> Rect {
    Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    }
}
