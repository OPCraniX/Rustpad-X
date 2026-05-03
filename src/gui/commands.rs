fn handle_command(app: &mut AppData, command_id: u16) {
    if let Some(index) = recent_file_command_index(command_id) {
        open_recent_file(app, index);
        return;
    }

    match command_id {
        ID_FILE_NEW => new_file(app),
        ID_FILE_OPEN => open_file(app),
        ID_FILE_OPEN_PROJECT => open_project(app),
        ID_FILE_CLOSE_TAB => close_active_tab(app),
        ID_FILE_SAVE => save_file(app),
        ID_FILE_SAVE_ALL => save_all(app),
        ID_FILE_SAVE_AS => save_file_as(app),
        ID_FILE_PRINT => print_active_file(app),
        ID_FILE_EXIT => close_compare_or_exit(app),
        ID_EDIT_UNDO => send_mutating_edit_message(app, WM_UNDO),
        ID_EDIT_REDO => send_mutating_edit_message(app, EM_REDO),
        ID_EDIT_CUT => send_mutating_edit_message(app, WM_CUT),
        ID_EDIT_COPY => send_edit_message(app, WM_COPY),
        ID_EDIT_PASTE => paste_into_editor(app),
        ID_EDIT_FIND => open_find_dialog(app),
        ID_EDIT_FIND_NEXT => find_next_from_menu(app),
        ID_EDIT_FIND_PREVIOUS => find_previous_from_menu(app),
        ID_EDIT_REPLACE => open_replace_dialog(app),
        ID_EDIT_DATE => insert_date(app),
        ID_EDIT_TIME_DATE => insert_time_and_date(app),
        ID_EDIT_FONT => choose_editor_font(app),
        ID_EDIT_SELECT_ALL => unsafe {
            SendMessageW(app.edit, EM_SETSEL, 0, -1);
        },
        ID_VIEW_LINE_NUMBERS => toggle_line_numbers(app),
        ID_VIEW_WORD_WRAP => toggle_word_wrap(app),
        ID_VIEW_FULLSCREEN => toggle_fullscreen(app),
        ID_VIEW_ZOOM_IN => adjust_zoom(app, ZOOM_STEP_PERCENT),
        ID_VIEW_ZOOM_OUT => adjust_zoom(app, -ZOOM_STEP_PERCENT),
        ID_VIEW_LIGHT_MODE => set_theme(app, Theme::Light),
        ID_VIEW_DARK_MODE => set_theme(app, Theme::Dark),
        ID_VIEW_NEXT_TAB => cycle_tab(app, 1),
        ID_VIEW_PREVIOUS_TAB => cycle_tab(app, -1),
        ID_VIEW_GOTO_LINE => show_goto_line(app),
        ID_VIEW_TOP_OF_DOCUMENT => move_to_document_edge(app, false),
        ID_VIEW_BOTTOM_OF_DOCUMENT => move_to_document_edge(app, true),
        ID_VIEW_COMPARE_TABS => show_compare_tab_picker(app),
        ID_VIEW_CLOSE_COMPARE_TABS => close_compare_tabs(app),
        ID_VIEW_SYNC_COMPARE_PAGING => toggle_compare_page_sync(app),
        ID_HELP_ABOUT => show_about(app.hwnd),
        _ => {}
    }
}

fn new_file(app: &mut AppData) {
    sync_active_document_text(app);
    let untitled_index = next_available_untitled_index(&app.documents);
    app.documents.push(Document::untitled(untitled_index));
    app.next_untitled_index = next_available_untitled_index(&app.documents);
    app.active_tab = app.documents.len() - 1;

    reset_editor_visual_state(app);
    set_active_edit_text(app, "");
    update_window_title(app);
    invalidate_document_chrome(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
    save_session_state(app);
}

fn open_file(app: &mut AppData) {
    let Some(path) = prompt_for_open_path(app.hwnd) else {
        return;
    };

    open_path(app, &path, "Open Failed");
}

fn open_recent_file(app: &mut AppData, index: usize) {
    let Some(path) = app.recent_files.get(index).cloned() else {
        return;
    };

    if !path.exists() {
        message_box(
            app.hwnd,
            "The recent file could not be found.",
            "Open Recent File",
            MB_OK | MB_ICONERROR,
        );
        remove_recent_file(app, &path);
        return;
    }

    open_path(app, &path, "Open Recent File Failed");
}

fn open_dropped_files(app: &mut AppData, drop_handle: Hdrop) {
    if drop_handle.is_null() {
        return;
    }

    let count = unsafe { DragQueryFileW(drop_handle, u32::MAX, null_mut(), 0) };
    for index in 0..count {
        if let Some(path) = dropped_file_path(drop_handle, index) {
            open_path(app, &path, "Open Dropped File Failed");
        }
    }

    unsafe {
        DragFinish(drop_handle);
    }
}

fn dropped_file_path(drop_handle: Hdrop, index: Uint) -> Option<PathBuf> {
    let len = unsafe { DragQueryFileW(drop_handle, index, null_mut(), 0) };
    if len == 0 {
        return None;
    }

    let mut buffer = vec![0u16; len as usize + 1];
    let copied = unsafe { DragQueryFileW(drop_handle, index, buffer.as_mut_ptr(), len + 1) };
    if copied == 0 {
        return None;
    }

    Some(PathBuf::from(OsString::from_wide(
        &buffer[..copied as usize],
    )))
}

fn open_path(app: &mut AppData, path: &Path, error_title: &str) -> bool {
    let path = normalized_path(path);
    sync_active_document_text(app);

    if let Some(index) = existing_document_index(app, &path) {
        switch_to_tab(app, index);
        add_recent_file(app, &path);
        return true;
    }

    match read_text_file_lossy(&path) {
        Ok(text) => {
            let document = Document::from_open_file(path.clone(), text, app.use_rich_edit);

            if active_document(app).is_empty_untitled() {
                replace_active_document(app, document);
            } else {
                if let Some(index) = app
                    .documents
                    .iter()
                    .position(Document::is_empty_initial_untitled)
                {
                    remove_tab_without_loading(app, index);
                }

                app.documents.push(document);
                app.active_tab = app.documents.len() - 1;

                let edit_text = active_document(app).text.clone();
                reset_editor_visual_state(app);
                set_active_edit_text(app, &edit_text);
                update_window_title(app);
                invalidate_document_chrome(app);
                invalidate_gutter(app);
                refresh_status_if_changed(app);
            }
            add_recent_file(app, &path);
            return true;
        }
        Err(error) => {
            message_box(
                app.hwnd,
                &format!("Could not open file:\n\n{error}"),
                error_title,
                MB_OK | MB_ICONERROR,
            );
        }
    }
    false
}

fn open_project(app: &mut AppData) {
    let Some(path) = prompt_for_project_source_path(app.hwnd) else {
        return;
    };

    let selected_path = normalized_path(&path);
    let paths = project_files_for_source(&selected_path);

    if paths.is_empty() {
        message_box(
            app.hwnd,
            "No matching source files were found near the selected file.",
            "Open Project",
            MB_OK | MB_ICONINFORMATION,
        );
        return;
    }

    open_project_paths(app, paths, &selected_path);
}

fn open_project_paths(app: &mut AppData, paths: Vec<PathBuf>, active_path: &Path) {
    sync_active_document_text(app);

    let mut empty_slot = if active_document(app).is_empty_untitled() {
        Some(app.active_tab)
    } else {
        if let Some(index) = app
            .documents
            .iter()
            .position(Document::is_empty_initial_untitled)
        {
            remove_tab_without_loading(app, index);
        }
        None
    };

    let mut active_index = existing_document_index(app, active_path);
    let mut opened_any = false;

    for path in paths {
        if let Some(index) = existing_document_index(app, &path) {
            if paths_match(&path, active_path) {
                active_index = Some(index);
            }
            continue;
        }

        let text = match read_text_file_lossy(&path) {
            Ok(text) => text,
            Err(error) => {
                if paths_match(&path, active_path) {
                    message_box(
                        app.hwnd,
                        &format!("Could not open project source file:\n\n{error}"),
                        "Open Project Failed",
                        MB_OK | MB_ICONERROR,
                    );
                }
                continue;
            }
        };

        let document = Document::from_open_file(path.clone(), text, app.use_rich_edit);
        let index = if let Some(slot) = empty_slot.take() {
            app.documents[slot] = document;
            slot
        } else {
            app.documents.push(document);
            app.documents.len() - 1
        };

        opened_any = true;
        if paths_match(&path, active_path) {
            active_index = Some(index);
        }
    }

    if !opened_any && active_index.is_none() {
        return;
    }

    app.active_tab = active_index.unwrap_or_else(|| app.documents.len().saturating_sub(1));
    let edit_text = active_document(app).text.clone();
    reset_editor_visual_state(app);
    set_active_edit_text(app, &edit_text);
    unsafe {
        SetFocus(app.edit);
    }
    update_window_title(app);
    invalidate_document_chrome(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
    add_recent_file(app, active_path);
}

fn existing_document_index(app: &AppData, path: &Path) -> Option<usize> {
    app.documents.iter().position(|document| {
        document
            .path
            .as_deref()
            .is_some_and(|document_path| paths_match(document_path, path))
    })
}

fn add_recent_file(app: &mut AppData, path: &Path) {
    let path = normalized_path(path);
    app.recent_files
        .retain(|recent| !paths_match(recent, &path));
    app.recent_files.insert(0, path);
    if app.recent_files.len() > RECENT_FILE_LIMIT {
        app.recent_files.truncate(RECENT_FILE_LIMIT);
    }
    update_recent_files_menu(app);
    save_session_state(app);
}

fn remove_recent_file(app: &mut AppData, path: &Path) {
    app.recent_files.retain(|recent| !paths_match(recent, path));
    update_recent_files_menu(app);
    save_session_state(app);
}

fn recent_file_command_index(command_id: u16) -> Option<usize> {
    let end = ID_FILE_RECENT_BASE + RECENT_FILE_LIMIT as u16;
    if (ID_FILE_RECENT_BASE..end).contains(&command_id) {
        Some((command_id - ID_FILE_RECENT_BASE) as usize)
    } else {
        None
    }
}

fn save_file(app: &mut AppData) {
    if active_document(app).path.is_none() {
        save_file_as(app);
        return;
    }

    if let Some(path) = active_document(app).path.clone() {
        if write_editor_text(app, &path) {
            add_recent_file(app, &path);
        }
    }
}

fn save_all(app: &mut AppData) {
    sync_active_document_text(app);

    for index in 0..app.documents.len() {
        let path = if let Some(path) = app.documents[index].path.clone() {
            path
        } else {
            let Some(path) = prompt_for_save_path(app.hwnd, None) else {
                return;
            };
            app.documents[index].path = Some(path.clone());
            path
        };

        let text = from_edit_line_endings_with(
            &app.documents[index].text,
            app.documents[index].line_ending,
        );
        if !write_text_to_path(app.hwnd, &path, &text) {
            return;
        }
        add_recent_file(app, &path);
    }

    update_window_title(app);
    invalidate_document_chrome(app);
}

fn save_file_as(app: &mut AppData) {
    let Some(path) = prompt_for_save_path(app.hwnd, active_document(app).path.as_deref())
    else {
        return;
    };

    if write_editor_text(app, &path) {
        active_document_mut(app).path = Some(path);
        sync_active_document_text(app);
        update_window_title(app);
        invalidate_document_chrome(app);
        if let Some(path) = active_document(app).path.clone() {
            add_recent_file(app, &path);
        }
    }
}

fn print_active_file(app: &mut AppData) {
    if active_document(app).path.is_none() {
        save_file_as(app);
    }

    let Some(path) = active_document(app).path.clone() else {
        return;
    };

    if !write_editor_text(app, &path) {
        return;
    }

    let operation = to_wide("print");
    let file = path
        .as_os_str()
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>();
    let result = unsafe {
        ShellExecuteW(
            app.hwnd,
            operation.as_ptr(),
            file.as_ptr(),
            null(),
            null(),
            SW_HIDE,
        )
    };

    if result <= 32 {
        message_box(
            app.hwnd,
            "Windows could not print this file.",
            "Print Failed",
            MB_OK | MB_ICONERROR,
        );
    }
}

fn write_editor_text(app: &AppData, path: &Path) -> bool {
    let text =
        from_edit_line_endings_with(&get_edit_text(app.edit), active_document(app).line_ending);
    write_text_to_path(app.hwnd, path, &text)
}

fn write_text_to_path(hwnd: Hwnd, path: &Path, text: &str) -> bool {
    match std::fs::write(path, text) {
        Ok(()) => true,
        Err(error) => {
            message_box(
                hwnd,
                &format!("Could not save file:\n\n{error}"),
                "Save Failed",
                MB_OK | MB_ICONERROR,
            );
            false
        }
    }
}

fn edit_text_len(edit: Hwnd) -> usize {
    if edit.is_null() {
        return 0;
    }

    unsafe { SendMessageW(edit, WM_GETTEXTLENGTH, 0, 0).max(0) as usize }
}

fn edit_text_is_large(edit: Hwnd) -> bool {
    edit_text_len(edit) > LARGE_TEXT_FEATURE_LIMIT
}

fn get_edit_text(edit: Hwnd) -> String {
    let text_len = edit_text_len(edit);
    let mut buffer = vec![0u16; text_len + 1];
    let copied = unsafe {
        SendMessageW(
            edit,
            WM_GETTEXT,
            buffer.len() as Wparam,
            buffer.as_mut_ptr() as Lparam,
        )
    } as usize;

    String::from_utf16_lossy(&buffer[..copied])
}

fn window_text(hwnd: Hwnd) -> String {
    get_edit_text(hwnd)
}

fn set_active_edit_text(app: &mut AppData, text: &str) {
    let edit = app.edit;
    set_edit_text_without_notifications(app, edit, text);
    refresh_editor_state_after_text_replace(app);
    ensure_gutter_sync(app);
}

fn set_compare_edit_text(app: &mut AppData, text: &str) {
    let edit = app.compare_edit;
    set_edit_text_without_notifications(app, edit, text);
}

fn set_edit_text_without_notifications(app: &mut AppData, edit: Hwnd, text: &str) {
    app.programmatic_text_update = true;
    unsafe {
        SendMessageW(edit, WM_SETREDRAW, 0, 0);
    }
    set_edit_text(edit, text);
    apply_word_wrap_to_edit(app, edit);
    unsafe {
        SendMessageW(edit, EM_EMPTYUNDOBUFFER, 0, 0);
        SendMessageW(edit, WM_SETREDRAW, 1, 0);
        InvalidateRect(edit, null(), 1);
    }
    app.programmatic_text_update = false;
}

fn refresh_editor_state_after_text_replace(app: &mut AppData) {
    refresh_fold_ranges(app);
    apply_fold_formats(app);
}

fn set_edit_text(edit: Hwnd, text: &str) {
    let wide = to_wide(text);
    unsafe {
        SendMessageW(edit, WM_SETTEXT, 0, wide.as_ptr() as Lparam);
        SendMessageW(edit, EM_SETSEL, 0, 0);
    }
}

fn send_edit_message(app: &AppData, message: Uint) {
    unsafe {
        SendMessageW(app.edit, message, 0, 0);
    }
}

fn send_mutating_edit_message(app: &mut AppData, message: Uint) {
    unsafe {
        SendMessageW(app.edit, message, 0, 0);
    }
    sync_active_document_text(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

fn paste_into_editor(app: &mut AppData) {
    let Ok(text) = clipboard_text(app.hwnd) else {
        return;
    };

    if text.is_empty() {
        return;
    }

    let (selection_start, _) = edit_selection(app.edit);
    let edit_text = to_editor_insert_line_endings(&text);
    let wide = to_wide(&edit_text);
    let inserted_len = (wide.len() - 1) as i32;
    let caret = selection_start + inserted_len;

    unsafe {
        SendMessageW(app.edit, WM_SETREDRAW, 0, 0);
        SendMessageW(app.edit, EM_REPLACESEL, 1, wide.as_ptr() as Lparam);
        SendMessageW(app.edit, EM_SETSEL, caret as Wparam, caret as Lparam);
        SetFocus(app.edit);
        SendMessageW(app.edit, WM_SETREDRAW, 1, 0);
        InvalidateRect(app.edit, null(), 1);
    }

    sync_active_document_text(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

fn insert_time_and_date(app: &mut AppData) {
    insert_text_at_caret(app, &time_and_date_text());
}

fn insert_date(app: &mut AppData) {
    insert_text_at_caret(app, &date_text());
}

fn insert_text_at_caret(app: &mut AppData, text: &str) {
    if text.is_empty() {
        return;
    }

    let (selection_start, _) = edit_selection(app.edit);
    let wide = to_wide(text);
    let caret = selection_start + (wide.len() - 1) as i32;

    unsafe {
        SendMessageW(app.edit, WM_SETREDRAW, 0, 0);
        SendMessageW(app.edit, EM_REPLACESEL, 1, wide.as_ptr() as Lparam);
        SendMessageW(app.edit, EM_SETSEL, caret as Wparam, caret as Lparam);
        SetFocus(app.edit);
        SendMessageW(app.edit, WM_SETREDRAW, 1, 0);
        InvalidateRect(app.edit, null(), 1);
    }

    sync_active_document_text(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

