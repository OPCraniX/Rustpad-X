fn create_editor_controls(app: &mut AppData) -> io::Result<()> {
    let menu_bar_class = to_wide(MENU_BAR_CLASS_NAME);
    app.menu_bar = unsafe {
        CreateWindowExW(
            0,
            menu_bar_class.as_ptr(),
            null(),
            (WS_CHILD | WS_VISIBLE) as Dword,
            0,
            0,
            0,
            MENU_BAR_HEIGHT,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.menu_bar.is_null() {
        return Err(io::Error::last_os_error());
    }

    let path_bar_class = to_wide(PATH_BAR_CLASS_NAME);
    app.path_bar = unsafe {
        CreateWindowExW(
            0,
            path_bar_class.as_ptr(),
            null(),
            (WS_CHILD | WS_VISIBLE) as Dword,
            0,
            MENU_BAR_HEIGHT,
            0,
            PATH_BAR_HEIGHT,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.path_bar.is_null() {
        return Err(io::Error::last_os_error());
    }

    let tab_bar_class = to_wide(TAB_BAR_CLASS_NAME);
    app.tab_bar = unsafe {
        CreateWindowExW(
            0,
            tab_bar_class.as_ptr(),
            null(),
            (WS_CHILD | WS_VISIBLE) as Dword,
            0,
            MENU_BAR_HEIGHT + PATH_BAR_HEIGHT,
            0,
            TAB_BAR_HEIGHT,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.tab_bar.is_null() {
        return Err(io::Error::last_os_error());
    }

    let gutter_class = to_wide(GUTTER_CLASS_NAME);
    app.gutter = unsafe {
        CreateWindowExW(
            0,
            gutter_class.as_ptr(),
            null(),
            (WS_CHILD | WS_VISIBLE) as Dword,
            0,
            0,
            GUTTER_WIDTH,
            0,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.gutter.is_null() {
        return Err(io::Error::last_os_error());
    }

    app.compare_gutter = unsafe {
        CreateWindowExW(
            0,
            gutter_class.as_ptr(),
            null(),
            WS_CHILD as Dword,
            0,
            0,
            GUTTER_WIDTH,
            0,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.compare_gutter.is_null() {
        return Err(io::Error::last_os_error());
    }

    let status_class = to_wide(STATUS_CLASS_NAME);
    app.status = unsafe {
        CreateWindowExW(
            0,
            status_class.as_ptr(),
            null(),
            (WS_CHILD | WS_VISIBLE) as Dword,
            0,
            0,
            0,
            STATUS_BAR_HEIGHT,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.status.is_null() {
        return Err(io::Error::last_os_error());
    }

    let edit_class = if app.use_rich_edit {
        to_wide(RICH_EDIT_CLASS_NAME)
    } else {
        to_wide("EDIT")
    };
    let mut edit_style = WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_VSCROLL
        | ES_LEFT
        | ES_MULTILINE
        | ES_AUTOVSCROLL
        | ES_NOHIDESEL
        | ES_WANTRETURN;
    if !app.word_wrap_enabled {
        edit_style |= WS_HSCROLL | ES_AUTOHSCROLL;
    }

    app.edit = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            null(),
            edit_style as Dword,
            0,
            0,
            0,
            0,
            app.hwnd,
            EDIT_CONTROL_ID as Hmenu,
            null_mut(),
            null_mut(),
        )
    };

    if app.edit.is_null() {
        return Err(io::Error::last_os_error());
    }

    app.compare_edit = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            null(),
            ((edit_style & !WS_VISIBLE) | ES_READONLY) as Dword,
            0,
            0,
            0,
            0,
            app.hwnd,
            COMPARE_EDIT_CONTROL_ID as Hmenu,
            null_mut(),
            null_mut(),
        )
    };

    if app.compare_edit.is_null() {
        return Err(io::Error::last_os_error());
    }

    app.line_probe = unsafe {
        CreateWindowExW(
            0,
            edit_class.as_ptr(),
            null(),
            (WS_CHILD | ES_LEFT | ES_READONLY) as Dword,
            0,
            0,
            0,
            1,
            app.hwnd,
            LINE_PROBE_CONTROL_ID as Hmenu,
            null_mut(),
            null_mut(),
        )
    };

    if app.line_probe.is_null() {
        return Err(io::Error::last_os_error());
    }

    let splitter_class = to_wide(COMPARE_SPLITTER_CLASS_NAME);
    app.compare_splitter = unsafe {
        CreateWindowExW(
            0,
            splitter_class.as_ptr(),
            null(),
            WS_CHILD as Dword,
            0,
            0,
            COMPARE_SPLITTER_WIDTH,
            0,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.compare_splitter.is_null() {
        return Err(io::Error::last_os_error());
    }

    let copy_indicator_class = to_wide(COPY_INDICATOR_CLASS_NAME);
    app.copy_indicator = unsafe {
        CreateWindowExW(
            0,
            copy_indicator_class.as_ptr(),
            null(),
            WS_CHILD as Dword,
            0,
            0,
            COPY_INDICATOR_WIDTH,
            COPY_INDICATOR_HEIGHT,
            app.hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    if app.copy_indicator.is_null() {
        return Err(io::Error::last_os_error());
    }

    app.font = create_editor_font(app)?;
    if app.font.is_null() {
        return Err(io::Error::last_os_error());
    }

    unsafe {
        let initial_text = active_document(app).text.clone();
        let edit = app.edit;
        set_edit_text_without_notifications(app, edit, &initial_text);
        SendMessageW(app.edit, WM_SETFONT, app.font as Wparam, 1);
        SendMessageW(app.compare_edit, WM_SETFONT, app.font as Wparam, 1);
        SendMessageW(app.line_probe, WM_SETFONT, app.font as Wparam, 1);
        apply_editor_char_formats(app);
        apply_word_wrap_to_edit(app, app.edit);
        apply_word_wrap_to_edit(app, app.compare_edit);
        ShowWindow(app.line_probe, SW_HIDE);
        ShowWindow(app.copy_indicator, SW_HIDE);
        apply_rich_edit_theme(app);
        let previous = SetWindowLongPtrW(
            app.edit,
            GWLP_WNDPROC,
            edit_proc as *const () as usize as isize,
        );
        SetWindowLongPtrW(app.edit, GWLP_USERDATA, previous);
        let previous_compare = SetWindowLongPtrW(
            app.compare_edit,
            GWLP_WNDPROC,
            edit_proc as *const () as usize as isize,
        );
        SetWindowLongPtrW(app.compare_edit, GWLP_USERDATA, previous_compare);
        SetFocus(app.edit);
    }
    invalidate_gutter(app);
    Ok(())
}

fn layout_editor(app: &AppData, width: i32, height: i32) {
    let menu_height = MENU_BAR_HEIGHT;
    let top_height = menu_height + PATH_BAR_HEIGHT + TAB_BAR_HEIGHT;
    let status_height = STATUS_BAR_HEIGHT;
    let gutter_width = if app.line_numbers_visible {
        GUTTER_WIDTH
    } else {
        0
    };
    let editor_height = (height - status_height - top_height).max(0);
    let status_top = top_height + editor_height;
    let compare_open = app.compare_tab.is_some();
    let (left_width, right_left, right_width) = if compare_open {
        compare_layout_columns(app, width.max(0), gutter_width)
    } else {
        (width.max(0), width.max(0), 0)
    };

    unsafe {
        ShowWindow(app.menu_bar, SW_SHOW);
        MoveWindow(app.menu_bar, 0, 0, width.max(0), menu_height, 1);
        MoveWindow(
            app.path_bar,
            0,
            menu_height,
            width.max(0),
            PATH_BAR_HEIGHT,
            1,
        );
        MoveWindow(
            app.tab_bar,
            0,
            menu_height + PATH_BAR_HEIGHT,
            width.max(0),
            TAB_BAR_HEIGHT,
            1,
        );
        ShowWindow(
            app.gutter,
            if app.line_numbers_visible {
                SW_SHOW
            } else {
                SW_HIDE
            },
        );
        MoveWindow(app.gutter, 0, top_height, gutter_width, editor_height, 1);
        MoveWindow(
            app.edit,
            gutter_width,
            top_height,
            (left_width - gutter_width).max(0),
            editor_height,
            1,
        );
        MoveWindow(
            app.line_probe,
            gutter_width,
            (top_height + editor_height).saturating_sub(1),
            (left_width - gutter_width).max(0),
            1,
            0,
        );
        ShowWindow(app.line_probe, SW_HIDE);
        if compare_open {
            ShowWindow(app.compare_splitter, SW_SHOW);
            ShowWindow(
                app.compare_gutter,
                if app.line_numbers_visible {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
            ShowWindow(app.compare_edit, SW_SHOW);
            MoveWindow(
                app.compare_splitter,
                left_width,
                top_height,
                COMPARE_SPLITTER_WIDTH,
                editor_height,
                1,
            );
            MoveWindow(
                app.compare_gutter,
                right_left,
                top_height,
                gutter_width,
                editor_height,
                1,
            );
            MoveWindow(
                app.compare_edit,
                right_left + gutter_width,
                top_height,
                (right_width - gutter_width).max(0),
                editor_height,
                1,
            );
        } else {
            ShowWindow(app.compare_splitter, SW_HIDE);
            ShowWindow(app.compare_gutter, SW_HIDE);
            ShowWindow(app.compare_edit, SW_HIDE);
        }
        ShowWindow(app.status, SW_SHOW);
        MoveWindow(app.status, 0, status_top, width.max(0), status_height, 1);
        InvalidateRect(app.menu_bar, null(), 0);
        InvalidateRect(app.path_bar, null(), 1);
        InvalidateRect(app.tab_bar, null(), 1);
        InvalidateRect(app.gutter, null(), 0);
        InvalidateRect(app.compare_gutter, null(), 0);
        InvalidateRect(app.line_probe, null(), 0);
        InvalidateRect(app.compare_splitter, null(), 1);
        InvalidateRect(app.compare_edit, null(), 1);
        InvalidateRect(app.status, null(), 0);
    }
}

fn compare_layout_columns(app: &AppData, width: i32, gutter_width: i32) -> (i32, i32, i32) {
    if width <= COMPARE_SPLITTER_WIDTH {
        return (width.max(0), width.max(0), 0);
    }

    let min_left = (gutter_width + 120).min(width.saturating_sub(COMPARE_SPLITTER_WIDTH));
    let max_left = (width - COMPARE_SPLITTER_WIDTH - 120).max(min_left);
    let desired = (width * app.compare_split_ratio / 10_000).clamp(min_left, max_left);
    let right_left = desired + COMPARE_SPLITTER_WIDTH;
    let right_width = (width - right_left).max(0);
    (desired, right_left, right_width)
}

fn handle_edit_notification(app: &mut AppData, notification: u16) {
    if app.programmatic_text_update {
        return;
    }

    match notification {
        EN_CHANGE => {
            sync_active_document_text(app);
            sync_line_probe(app);
            invalidate_gutter(app);
            refresh_status_if_changed(app);
        }
        EN_UPDATE => {}
        EN_VSCROLL => {
            sync_line_probe(app);
            invalidate_gutter_if_scrolled(app);
        }
        _ => {}
    }
}

fn handle_editor_key_command(edit: Hwnd, wparam: Wparam) -> bool {
    let key = wparam as u16;
    let control_down = key_is_down(VK_CONTROL);
    let shift_down = key_is_down(VK_SHIFT);
    let alt_down = key_is_down(VK_MENU);

    if key == VK_RETURN && !control_down && !alt_down {
        let parent = unsafe { GetParent(edit) };
        let handled = with_app_data(parent, |app| {
            if edit != app.edit {
                return false;
            }

            insert_auto_indented_newline(app);
            true
        })
        .unwrap_or(false);

        if handled {
            return true;
        }
    }

    if (key == VK_NEXT || key == VK_PRIOR) && !control_down && !shift_down && !alt_down {
        let parent = unsafe { GetParent(edit) };
        let handled = with_app_data(parent, |app| {
            page_editor_by_visible_span(app, edit, key == VK_NEXT)
        })
        .unwrap_or(false);

        if handled {
            return true;
        }
    }

    if control_down && !shift_down && key == b'F' as u16 {
        let parent = unsafe { GetParent(edit) };
        with_app_data(parent, open_find_dialog);
        return true;
    }

    if control_down && !shift_down && key == b'R' as u16 {
        let parent = unsafe { GetParent(edit) };
        with_app_data(parent, open_replace_dialog);
        return true;
    }

    if control_down && shift_down && key == b'F' as u16 {
        let parent = unsafe { GetParent(edit) };
        with_app_data(parent, choose_editor_font);
        return true;
    }

    if key == VK_F3 {
        let parent = unsafe { GetParent(edit) };
        if shift_down {
            with_app_data(parent, find_previous_from_menu);
        } else {
            with_app_data(parent, find_next_from_menu);
        }
        return true;
    }

    false
}

fn key_is_down(key: u16) -> bool {
    unsafe { GetKeyState(key as Int) < 0 }
}

fn handle_editor_char(edit: Hwnd, wparam: Wparam) -> bool {
    let Some(character) = char::from_u32(wparam as u32) else {
        return false;
    };

    let parent = unsafe { GetParent(edit) };
    let mut handled = false;
    with_app_data(parent, |app| {
        if edit != app.edit {
            return;
        }

        if character == '\r' {
            handled = true;
            return;
        }

        if should_skip_paired_closer(app, character) {
            move_caret_by(app.edit, 1);
            handled = true;
            return;
        }

        if let Some(close) = auto_pair_close(character) {
            if should_insert_auto_pair(app, character) {
                insert_auto_pair(app, character, close);
                handled = true;
            }
        }
    });

    handled
}

fn insert_auto_indented_newline(app: &mut AppData) {
    let text = get_edit_text(app.edit);
    let (selection_start, _) = edit_selection(app.edit);
    let caret_byte = byte_index_from_utf16_pos(&text, selection_start);
    let line_start_byte = line_start_byte_before(&text, caret_byte);
    let line_before_caret = &text[line_start_byte..caret_byte];
    let base_indent = leading_indentation(line_before_caret);
    let trimmed_before = line_before_caret.trim_end();
    let adds_block_indent = trimmed_before.ends_with('{')
        || trimmed_before.ends_with('[')
        || trimmed_before.ends_with('(');
    let inner_indent = if adds_block_indent {
        format!("{base_indent}    ")
    } else {
        base_indent.clone()
    };
    let next = text[caret_byte..]
        .chars()
        .find(|character| !character.is_whitespace());
    let closes_block = adds_block_indent && next.is_some_and(is_closing_bracket);
    let line_break = editor_insert_line_break(app);

    let replacement = if closes_block {
        format!("{line_break}{inner_indent}{line_break}{base_indent}")
    } else {
        format!("{line_break}{inner_indent}")
    };
    let caret = selection_start + utf16_len(&format!("{line_break}{inner_indent}"));
    replace_selection_with_text(app, &replacement, caret, caret);
}

fn line_start_byte_before(text: &str, byte_index: usize) -> usize {
    let limit = byte_index.min(text.len());
    let mut start = 0;
    let mut chars = text[..limit].char_indices().peekable();

    while let Some((index, character)) = chars.next() {
        if character == '\r' {
            if let Some((next_index, '\n')) = chars.peek().copied() {
                chars.next();
                start = next_index + 1;
            } else {
                start = index + 1;
            }
        } else if character == '\n' {
            start = index + 1;
        }
    }

    start
}

fn editor_insert_line_break(app: &AppData) -> &'static str {
    if app.use_rich_edit { "\r" } else { "\r\n" }
}

fn leading_indentation(line: &str) -> String {
    line.chars()
        .take_while(|character| *character == ' ' || *character == '\t')
        .collect()
}

fn auto_pair_close(open: char) -> Option<char> {
    match open {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '`' => Some('`'),
        _ => None,
    }
}

fn should_insert_auto_pair(app: &AppData, open: char) -> bool {
    if open != '\'' {
        return true;
    }

    let text = get_edit_text(app.edit);
    let (selection_start, selection_end) = edit_selection(app.edit);
    if selection_start != selection_end {
        return true;
    }

    !char_before_utf16(&text, selection_start)
        .is_some_and(|(_, character)| character.is_alphanumeric())
}

fn insert_auto_pair(app: &mut AppData, open: char, close: char) {
    let text = get_edit_text(app.edit);
    let (selection_start, selection_end) = edit_selection(app.edit);
    let selected = if selection_start == selection_end {
        String::new()
    } else {
        let start_byte = byte_index_from_utf16_pos(&text, selection_start);
        let end_byte = byte_index_from_utf16_pos(&text, selection_end);
        text[start_byte..end_byte].to_string()
    };

    let replacement = format!("{open}{selected}{close}");
    let inner_start = selection_start + open.len_utf16() as i32;
    let inner_end = inner_start + utf16_len(&selected);
    replace_selection_with_text(app, &replacement, inner_start, inner_end);
}

fn should_skip_paired_closer(app: &AppData, typed: char) -> bool {
    if !is_closing_pair_character(typed) {
        return false;
    }

    let (selection_start, selection_end) = edit_selection(app.edit);
    if selection_start != selection_end {
        return false;
    }

    let text = get_edit_text(app.edit);
    char_at_utf16(&text, selection_start).is_some_and(|(_, next)| next == typed)
}

fn is_closing_pair_character(character: char) -> bool {
    matches!(character, ')' | ']' | '}' | '"' | '\'' | '`')
}

fn is_closing_bracket(character: char) -> bool {
    matches!(character, ')' | ']' | '}')
}

fn replace_selection_with_text(
    app: &mut AppData,
    text: &str,
    selection_start: i32,
    selection_end: i32,
) {
    let wide = to_wide(text);
    unsafe {
        SendMessageW(app.edit, WM_SETREDRAW, 0, 0);
        SendMessageW(app.edit, EM_REPLACESEL, 1, wide.as_ptr() as Lparam);
        SendMessageW(
            app.edit,
            EM_SETSEL,
            selection_start.max(0) as Wparam,
            selection_end.max(0) as Lparam,
        );
        SetFocus(app.edit);
        SendMessageW(app.edit, WM_SETREDRAW, 1, 0);
        InvalidateRect(app.edit, null(), 1);
    }

    sync_active_document_text(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

fn move_caret_by(edit: Hwnd, delta: i32) {
    let (selection_start, _) = edit_selection(edit);
    let target = (selection_start + delta).max(0);
    unsafe {
        SendMessageW(edit, EM_SETSEL, target as Wparam, target as Lparam);
        SendMessageW(edit, EM_SCROLLCARET, 0, 0);
    }
}

fn page_editor_by_visible_span(app: &mut AppData, edit: Hwnd, page_down: bool) -> bool {
    if !is_editor_child(app, edit) || app.fold_formats_active {
        return false;
    }

    let line_count = editor_document_line_count(app, edit).min(i32::MAX as usize) as i32;
    let first_visible = first_visible_editor_line(app, edit).min(line_count.saturating_sub(1));
    let page_delta = visible_page_line_delta(app, edit, first_visible);
    if page_delta <= 0 {
        return true;
    }

    let target_top = if page_down {
        (first_visible + page_delta).min(line_count.saturating_sub(1))
    } else {
        first_visible.saturating_sub(page_delta)
    };
    let line_delta = target_top - first_visible;
    if line_delta == 0 {
        return true;
    }

    let column = caret_column_offset(app, edit);
    unsafe {
        SendMessageW(edit, EM_LINESCROLL, 0, line_delta as Lparam);
    }

    let actual_top = first_visible_editor_line(app, edit).min(line_count.saturating_sub(1));
    let actual_delta = actual_top - first_visible;
    set_caret_to_line(app, edit, actual_top, column);

    unsafe {
        SetFocus(edit);
        InvalidateRect(edit, null(), 1);
    }

    sync_compare_page_scroll_for_app(app, edit, actual_delta);
    if edit == app.edit {
        ensure_gutter_sync(app);
        refresh_status_if_changed(app);
        invalidate_gutter(app);
    }

    true
}

fn visible_page_line_delta(app: &AppData, edit: Hwnd, first_visible: i32) -> i32 {
    let mut rect = empty_rect();
    if unsafe { GetClientRect(edit, &mut rect) } == 0 {
        return 1;
    }

    let bottom_y = (rect.bottom - rect.top - 2).max(0);
    editor_line_at_point(app, edit, 4, bottom_y)
        .map(|bottom_line| bottom_line.saturating_sub(first_visible))
        .filter(|delta| *delta > 0)
        .unwrap_or(1)
}

fn caret_column_offset(app: &AppData, edit: Hwnd) -> i32 {
    let (selection_start, _) = edit_selection(edit);
    let document = document_for_editor(app, edit);
    let line = line_index_from_starts(&document.line_starts, selection_start);
    let line_start = line_start_for_index(&document.line_starts, line);
    (selection_start - line_start).max(0)
}

fn set_caret_to_line(app: &AppData, edit: Hwnd, line_index: i32, column: i32) {
    let document = document_for_editor(app, edit);
    let Some(line_start) = document_line_start_utf16(document, line_index.max(0)) else {
        return;
    };
    let Some(line_end) = document_line_end_utf16(document, line_index.max(0)) else {
        return;
    };
    let position = (line_start + column.max(0)).min(line_end.saturating_sub(1).max(line_start));

    unsafe {
        SendMessageW(edit, EM_SETSEL, position as Wparam, position as Lparam);
    }
}

fn refresh_editor_visuals(edit: Hwnd) {
    let parent = unsafe { GetParent(edit) };
    with_app_data(parent, |app| {
        if edit == app.edit {
            refresh_status_if_changed(app);
        }
    });
}

fn refresh_editor_scroll_visuals(edit: Hwnd) {
    let parent = unsafe { GetParent(edit) };
    with_app_data(parent, |app| {
        if edit == app.edit {
            ensure_gutter_sync(app);
            invalidate_gutter(app);
        }
    });
}

fn char_at_utf16(text: &str, position: i32) -> Option<(i32, char)> {
    let byte_index = byte_index_from_utf16_pos(text, position);
    let character = text[byte_index..].chars().next()?;
    Some((utf16_pos_from_byte_index(text, byte_index), character))
}

fn char_before_utf16(text: &str, position: i32) -> Option<(i32, char)> {
    let byte_index = byte_index_from_utf16_pos(text, position);
    let (start_byte, character) = text[..byte_index].char_indices().next_back()?;
    Some((utf16_pos_from_byte_index(text, start_byte), character))
}

#[allow(dead_code)]
fn schedule_fold_refresh(app: &mut AppData) {
    if !app.use_rich_edit || app.edit.is_null() || edit_text_is_large(app.edit) {
        app.fold_refresh_pending = false;
        return;
    }

    app.fold_refresh_pending = true;
    app.fold_refresh_timer_active = true;
    unsafe {
        SetTimer(
            app.hwnd,
            FOLD_REFRESH_TIMER_ID,
            FOLD_REFRESH_TIMER_MS,
            null_mut(),
        );
    }
}

#[allow(dead_code)]
fn run_pending_fold_refresh(app: &mut AppData) {
    if !app.fold_refresh_pending {
        return;
    }

    app.fold_refresh_pending = false;
    if !app.use_rich_edit || app.edit.is_null() || edit_text_is_large(app.edit) {
        app.fold_ranges.clear();
        app.fold_formats_active = false;
        invalidate_gutter(app);
        return;
    }

    refresh_fold_ranges(app);
    apply_fold_formats(app);
    sync_line_probe(app);
    invalidate_gutter(app);
}

#[allow(dead_code)]
fn refresh_fold_ranges(app: &mut AppData) {
    if !app.use_rich_edit || app.edit.is_null() || edit_text_is_large(app.edit) {
        app.fold_ranges.clear();
        app.fold_formats_active = false;
        return;
    }

    let collapsed_ranges: HashSet<(i32, i32)> = app
        .fold_ranges
        .iter()
        .filter(|range| range.collapsed)
        .map(|range| (range.start_utf16, range.end_utf16))
        .collect();
    let text = get_edit_text(app.edit);
    let mut ranges = fold_ranges_for_text(&text);
    for range in &mut ranges {
        range.collapsed = collapsed_ranges.contains(&(range.start_utf16, range.end_utf16));
    }
    app.fold_ranges = ranges;
}

#[allow(dead_code)]
fn fold_ranges_for_text(text: &str) -> Vec<FoldRange> {
    let mut line_starts = vec![0];
    let mut stack = Vec::<(i32, i32)>::new();
    let mut ranges = Vec::new();
    let mut current_line = 0i32;
    let mut position = 0i32;
    let mut characters = text.chars().peekable();

    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string: Option<char> = None;
    let mut escaped = false;

    while let Some(character) = characters.next() {
        let character_position = position;
        let mut consumed_newline = false;

        if character == '\r' {
            position += 1;
            if characters.peek() == Some(&'\n') {
                characters.next();
                position += 1;
            }
            line_starts.push(position);
            current_line += 1;
            in_line_comment = false;
            escaped = false;
            consumed_newline = true;
        } else if character == '\n' {
            position += 1;
            line_starts.push(position);
            current_line += 1;
            in_line_comment = false;
            escaped = false;
            consumed_newline = true;
        } else {
            position += character.len_utf16() as i32;
        }

        if consumed_newline {
            continue;
        }

        if in_line_comment {
            continue;
        }

        if in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                position += 1;
                in_block_comment = false;
            }
            continue;
        }

        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                in_string = None;
            }
            continue;
        }

        if character == '/' && characters.peek() == Some(&'/') {
            characters.next();
            position += 1;
            in_line_comment = true;
            continue;
        }

        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            position += 1;
            in_block_comment = true;
            continue;
        }

        if character == '#' {
            in_line_comment = true;
            continue;
        }

        if character == '"' || character == '\'' {
            in_string = Some(character);
            escaped = false;
            continue;
        }

        match character {
            '{' => stack.push((character_position, current_line)),
            '}' => {
                if let Some((start, start_line)) = stack.pop() {
                    let end_line = current_line;
                    if end_line > start_line + 1 {
                        let hidden_start = line_start_for_index(&line_starts, start_line + 1);
                        let hidden_end = line_start_for_index(&line_starts, end_line);
                        if hidden_start < hidden_end {
                            ranges.push(FoldRange {
                                start_utf16: start,
                                end_utf16: character_position,
                                hidden_start_utf16: hidden_start,
                                hidden_end_utf16: hidden_end,
                                start_line,
                                end_line,
                                collapsed: false,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ranges.sort_by_key(|range| (range.start_line, range.end_line, range.start_utf16));
    ranges
}

fn line_starts_utf16(text: &str) -> Vec<i32> {
    let mut starts = vec![0];
    let mut utf16_position = 0i32;
    let mut characters = text.chars().peekable();

    while let Some(character) = characters.next() {
        if character == '\r' {
            utf16_position += 1;
            if characters.peek() == Some(&'\n') {
                characters.next();
                utf16_position += 1;
            }
            starts.push(utf16_position);
        } else if character == '\n' {
            utf16_position += 1;
            starts.push(utf16_position);
        } else {
            utf16_position += character.len_utf16() as i32;
        }
    }

    starts
}

fn logical_line_count(text: &str) -> usize {
    let mut count = 1usize;
    let bytes = text.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                count += 1;
                index += 1;
                if index < bytes.len() && bytes[index] == b'\n' {
                    index += 1;
                }
            }
            b'\n' => {
                count += 1;
                index += 1;
            }
            _ => {
                index += 1;
            }
        }
    }

    count
}

fn detect_line_ending(text: &str) -> LineEnding {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    let mut crlf = 0usize;
    let mut lf = 0usize;
    let mut cr = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\n' {
                    crlf += 1;
                    index += 2;
                } else {
                    cr += 1;
                    index += 1;
                }
            }
            b'\n' => {
                lf += 1;
                index += 1;
            }
            _ => {
                index += 1;
            }
        }
    }

    let kinds = usize::from(crlf > 0) + usize::from(lf > 0) + usize::from(cr > 0);
    match kinds {
        0 => LineEnding::Lf,
        1 if crlf > 0 => LineEnding::Crlf,
        1 if lf > 0 => LineEnding::Lf,
        1 if cr > 0 => LineEnding::Cr,
        _ => LineEnding::Mixed,
    }
}

fn line_index_from_starts(line_starts: &[i32], position: i32) -> i32 {
    match line_starts.binary_search(&position.max(0)) {
        Ok(index) => index as i32,
        Err(index) => index.saturating_sub(1) as i32,
    }
}

fn line_start_for_index(line_starts: &[i32], line_index: i32) -> i32 {
    line_starts
        .get(line_index.max(0) as usize)
        .copied()
        .unwrap_or_else(|| line_starts.last().copied().unwrap_or(0))
}

#[allow(dead_code)]
fn apply_fold_formats(app: &mut AppData) {
    if !app.use_rich_edit || app.edit.is_null() || edit_text_is_large(app.edit) {
        app.fold_formats_active = false;
        return;
    }

    let collapsed_ranges: Vec<(i32, i32)> = app
        .fold_ranges
        .iter()
        .filter(|range| range.collapsed)
        .map(|range| (range.hidden_start_utf16, range.hidden_end_utf16))
        .collect();

    if collapsed_ranges.is_empty() {
        if app.fold_formats_active {
            let text_len = edit_text_len(app.edit).min(i32::MAX as usize) as i32;
            apply_hidden_format_changes(app.edit, &[(0, text_len, false)]);
        }
        app.fold_formats_active = false;
        ensure_gutter_sync(app);
        invalidate_gutter(app);
        return;
    }

    let text_len = edit_text_len(app.edit).min(i32::MAX as usize) as i32;
    let mut changes = Vec::with_capacity(collapsed_ranges.len() + 1);
    changes.push((0, text_len, false));
    changes.extend(
        collapsed_ranges
            .iter()
            .map(|(start, end)| (*start, *end, true)),
    );
    apply_hidden_format_changes(app.edit, &changes);
    app.fold_formats_active = true;
    ensure_gutter_sync(app);
    invalidate_gutter(app);
}

#[allow(dead_code)]
fn apply_fold_toggle_format(app: &mut AppData, index: usize) {
    if !app.use_rich_edit || app.edit.is_null() || edit_text_is_large(app.edit) {
        return;
    }

    let Some(range) = app.fold_ranges.get(index).cloned() else {
        return;
    };

    let mut changes = vec![(
        range.hidden_start_utf16,
        range.hidden_end_utf16,
        range.collapsed,
    )];

    if !range.collapsed {
        for (other_index, other) in app.fold_ranges.iter().enumerate() {
            if other_index != index && other.collapsed && fold_ranges_intersect(&range, other) {
                changes.push((other.hidden_start_utf16, other.hidden_end_utf16, true));
            }
        }
    }

    apply_hidden_format_changes(app.edit, &changes);
    app.fold_formats_active = app.fold_ranges.iter().any(|range| range.collapsed);
    ensure_gutter_sync(app);
    invalidate_gutter(app);
}

#[allow(dead_code)]
fn fold_ranges_intersect(left: &FoldRange, right: &FoldRange) -> bool {
    left.hidden_start_utf16 < right.hidden_end_utf16
        && right.hidden_start_utf16 < left.hidden_end_utf16
}

fn apply_hidden_format_changes(edit: Hwnd, changes: &[(i32, i32, bool)]) {
    let (selection_start, selection_end) = edit_selection(edit);
    let scroll_pos = edit_scroll_pos(edit);

    unsafe {
        SendMessageW(edit, WM_SETREDRAW, 0, 0);
        for (start, end, hidden) in changes {
            if start >= end {
                continue;
            }
            SendMessageW(edit, EM_SETSEL, *start as Wparam, *end as Lparam);
            set_selected_hidden(edit, *hidden);
        }
        SendMessageW(
            edit,
            EM_SETSEL,
            selection_start as Wparam,
            selection_end as Lparam,
        );
        set_edit_scroll_pos(edit, scroll_pos);
        SendMessageW(edit, WM_SETREDRAW, 1, 0);
        set_edit_scroll_pos(edit, scroll_pos);
        InvalidateRect(edit, null(), 1);
        UpdateWindow(edit);
    }
}

fn edit_scroll_pos(edit: Hwnd) -> Point {
    let mut point = Point { x: 0, y: 0 };
    if !edit.is_null() {
        unsafe {
            SendMessageW(
                edit,
                EM_GETSCROLLPOS,
                0,
                (&mut point as *mut Point) as Lparam,
            );
        }
    }
    point
}

fn set_edit_scroll_pos(edit: Hwnd, mut point: Point) {
    if edit.is_null() {
        return;
    }

    unsafe {
        SendMessageW(
            edit,
            EM_SETSCROLLPOS,
            0,
            (&mut point as *mut Point) as Lparam,
        );
    }
}

fn set_selected_hidden(edit: Hwnd, hidden: bool) {
    let mut format: CharFormat2W = unsafe { zeroed() };
    format.cbSize = size_of::<CharFormat2W>() as Uint;
    format.dwMask = CFM_HIDDEN;
    format.dwEffects = if hidden { CFE_HIDDEN } else { 0 };
    unsafe {
        SendMessageW(
            edit,
            EM_SETCHARFORMAT,
            SCF_SELECTION,
            (&mut format as *mut CharFormat2W) as Lparam,
        );
    }
}

#[allow(dead_code)]
fn toggle_nearest_fold(app: &mut AppData) {
    if !app.use_rich_edit {
        return;
    }

    if app.fold_ranges.is_empty() {
        refresh_fold_ranges(app);
    }
    let (caret, _) = edit_selection(app.edit);
    let Some(index) = nearest_fold_index(app, caret) else {
        return;
    };

    app.fold_ranges[index].collapsed = !app.fold_ranges[index].collapsed;
    apply_fold_toggle_format(app, index);
}

#[allow(dead_code)]
fn collapse_all_folds(app: &mut AppData) {
    if !app.use_rich_edit {
        return;
    }

    refresh_fold_ranges(app);
    for range in &mut app.fold_ranges {
        range.collapsed = true;
    }
    apply_fold_formats(app);
}

#[allow(dead_code)]
fn expand_all_folds(app: &mut AppData) {
    if !app.use_rich_edit {
        return;
    }

    refresh_fold_ranges(app);
    for range in &mut app.fold_ranges {
        range.collapsed = false;
    }
    apply_fold_formats(app);
}

#[allow(dead_code)]
fn nearest_fold_index(app: &AppData, caret: i32) -> Option<usize> {
    let containing = app
        .fold_ranges
        .iter()
        .enumerate()
        .filter(|(_, range)| range.start_utf16 <= caret && caret <= range.end_utf16)
        .min_by_key(|(_, range)| range.end_utf16 - range.start_utf16)
        .map(|(index, _)| index);

    if containing.is_some() {
        return containing;
    }

    let text = get_edit_text(app.edit);
    let line_starts = line_starts_utf16(&text);
    let line = line_index_from_starts(&line_starts, caret);
    app.fold_ranges
        .iter()
        .enumerate()
        .filter(|(_, range)| range.start_line == line)
        .min_by_key(|(_, range)| range.end_utf16 - range.start_utf16)
        .map(|(index, _)| index)
}

#[allow(dead_code)]
fn update_gutter_hover(app: &mut AppData, x: i32, y: i32) {
    let hover_line = if (FOLD_HIT_LEFT..=FOLD_HIT_RIGHT).contains(&x) {
        gutter_line_at_y(app, y)
            .filter(|line| fold_range_starting_on_line(app, *line).is_some())
    } else {
        None
    };

    if app.gutter_hover_line != hover_line {
        app.gutter_hover_line = hover_line;
        invalidate_gutter(app);
    }
}

#[allow(dead_code)]
fn toggle_fold_from_gutter_y(app: &mut AppData, y: i32) {
    if !app.use_rich_edit {
        return;
    }

    if app.fold_ranges.is_empty() {
        refresh_fold_ranges(app);
    }
    let Some(line) = gutter_line_at_y(app, y) else {
        return;
    };
    let Some(index) = app
        .fold_ranges
        .iter()
        .enumerate()
        .filter(|(_, range)| range.start_line == line)
        .min_by_key(|(_, range)| range.end_utf16 - range.start_utf16)
        .map(|(index, _)| index)
    else {
        return;
    };

    app.fold_ranges[index].collapsed = !app.fold_ranges[index].collapsed;
    apply_fold_toggle_format(app, index);
}

fn gutter_line_at_y(app: &AppData, y: i32) -> Option<i32> {
    gutter_line_at_y_for(app, y, app.edit, app.gutter)
}

fn gutter_line_at_y_for(app: &AppData, y: i32, edit: Hwnd, gutter: Hwnd) -> Option<i32> {
    if edit.is_null() || gutter.is_null() {
        return None;
    }

    let hdc = unsafe { GetDC(gutter) };
    if hdc.is_null() {
        return None;
    }

    let old_font = if app.font.is_null() {
        null_mut()
    } else {
        unsafe { SelectObject(hdc, app.font as Hgdiobj) }
    };
    let mut rect = empty_rect();
    unsafe {
        GetClientRect(gutter, &mut rect);
    }

    let mut hit = None;
    let rows = visible_gutter_rows(app, hdc, rect, edit, gutter);
    for (index, row) in rows.iter().enumerate() {
        let bottom = rows
            .get(index + 1)
            .map(|next| next.top)
            .unwrap_or(row.top + row.height);
        if y >= row.top && y < bottom {
            hit = Some(row.line_index);
            break;
        }
    }

    if !old_font.is_null() {
        unsafe {
            SelectObject(hdc, old_font);
        }
    }
    unsafe {
        ReleaseDC(gutter, hdc);
    }

    hit
}

fn should_sync_compare_page_scroll(edit: Hwnd) -> bool {
    let parent = unsafe { GetParent(edit) };
    let app = app_data_ptr(parent);
    if app.is_null() {
        return false;
    }

    unsafe {
        (*app).compare_page_sync && (*app).compare_tab.is_some() && is_editor_child(&*app, edit)
    }
}

fn should_sync_compare_page_key(edit: Hwnd, wparam: Wparam) -> bool {
    let key = wparam as u16;
    if key != VK_PRIOR && key != VK_NEXT {
        return false;
    }

    if key_is_down(VK_CONTROL) || key_is_down(VK_MENU) {
        return false;
    }

    should_sync_compare_page_scroll(edit)
}

fn first_visible_line(edit: Hwnd) -> i32 {
    (unsafe { SendMessageW(edit, EM_GETFIRSTVISIBLELINE, 0, 0) } as i32).max(0)
}

fn first_visible_editor_line(app: &AppData, edit: Hwnd) -> i32 {
    let line_count = editor_document_line_count(app, edit).min(i32::MAX as usize) as i32;
    if line_count <= 0 || edit.is_null() {
        return 0;
    }

    let max_line = line_count.saturating_sub(1);

    // EM_GETFIRSTVISIBLELINE returns a display line for RichEdit. Wrapped
    // long lines count as extra display lines, so using it as a document
    // line makes the gutter drift farther down in large/project files.
    // Ask RichEdit which character is actually at the top-left instead.
    if app.use_rich_edit {
        let fallback = first_visible_line(edit).min(max_line);
        return [0, 1, 2, 4, 8, 12, 16]
            .into_iter()
            .filter_map(|y| editor_line_at_point(app, edit, 4, y))
            .filter(|line| *line >= 0 && *line <= max_line)
            .find(|line| edit_line_y(app, edit, *line).is_some_and(|y| (-64..=64).contains(&y)))
            .unwrap_or(fallback)
            .clamp(0, max_line);
    }

    first_visible_line(edit).min(max_line)
}

fn editor_line_at_point(app: &AppData, edit: Hwnd, x: i32, y: i32) -> Option<i32> {
    if edit.is_null() {
        return None;
    }

    if app.use_rich_edit {
        let mut point = Point { x, y };
        let char_index = unsafe {
            SendMessageW(
                edit,
                EM_CHARFROMPOS,
                0,
                (&mut point as *mut Point) as Lparam,
            )
        };
        if char_index < 0 {
            return None;
        }

        let line = unsafe { SendMessageW(edit, EM_LINEFROMCHAR, char_index as Wparam, 0) };
        if line < 0 {
            None
        } else {
            Some(line.min(i32::MAX as isize) as i32)
        }
    } else {
        let result = unsafe { SendMessageW(edit, EM_CHARFROMPOS, 0, point_lparam(x, y)) };
        if result < 0 {
            return None;
        }
        let line = high_word(result as usize);
        if line == u16::MAX {
            None
        } else {
            Some(line as i32)
        }
    }
}

fn sync_compare_page_key(scrolled_edit: Hwnd, message: Uint, wparam: Wparam, lparam: Lparam) {
    let parent = unsafe { GetParent(scrolled_edit) };
    with_app_data(parent, |app| {
        if !app.compare_page_sync || app.compare_tab.is_none() {
            return;
        }

        let target = if scrolled_edit == app.edit {
            app.compare_edit
        } else if scrolled_edit == app.compare_edit {
            app.edit
        } else {
            return;
        };

        if target.is_null() {
            return;
        }

        call_previous_edit_proc(target, message, wparam, lparam);
        unsafe {
            InvalidateRect(target, null(), 1);
        }
        ensure_gutter_sync(app);
        invalidate_gutter(app);
    });
}

fn sync_compare_page_scroll(scrolled_edit: Hwnd, line_delta: i32) {
    if line_delta == 0 {
        return;
    }

    let parent = unsafe { GetParent(scrolled_edit) };
    with_app_data(parent, |app| {
        sync_compare_page_scroll_for_app(app, scrolled_edit, line_delta);
    });
}

fn sync_compare_page_scroll_for_app(app: &mut AppData, scrolled_edit: Hwnd, line_delta: i32) {
    if line_delta == 0 || !app.compare_page_sync || app.compare_tab.is_none() {
        return;
    }

    let target = if scrolled_edit == app.edit {
        app.compare_edit
    } else if scrolled_edit == app.compare_edit {
        app.edit
    } else {
        return;
    };

    unsafe {
        SendMessageW(target, EM_LINESCROLL, 0, line_delta as Lparam);
        InvalidateRect(target, null(), 1);
    }
    ensure_gutter_sync(app);
    invalidate_gutter(app);
}

fn ensure_gutter_sync(app: &mut AppData) {
    if app.edit.is_null() {
        app.last_gutter_first_visible_line = 0;
        app.line_probe_metrics = None;
        return;
    }

    let line_count = editor_document_line_count(app, app.edit).min(i32::MAX as usize) as i32;
    if line_count <= 0 {
        app.last_gutter_first_visible_line = 0;
        app.line_probe_metrics = None;
        return;
    }

    let max_line = line_count.saturating_sub(1);
    let display_first_visible = first_visible_line(app.edit).clamp(0, max_line);
    let logical_first_visible = if app.fold_formats_active {
        first_visible_editor_line(app, app.edit)
    } else {
        display_first_visible
    };

    app.last_gutter_first_visible_line =
        visible_line_at_or_after(app, logical_first_visible, max_line);
    sync_line_probe(app);
}

fn sync_line_probe(app: &mut AppData) {
    if app.edit.is_null() || app.line_probe.is_null() {
        app.line_probe_metrics = None;
        return;
    }

    let mut edit_rect = empty_rect();
    unsafe {
        GetClientRect(app.edit, &mut edit_rect);
    }

    if edit_rect.bottom <= edit_rect.top {
        app.line_probe_metrics = None;
        return;
    }

    let max_line = editor_document_line_count(app, app.edit)
        .max(1)
        .min(i32::MAX as usize) as i32
        - 1;
    let sample_y = (edit_rect.bottom - 1).max(edit_rect.top);
    let sampled_line = editor_line_at_point(app, app.edit, 4, sample_y)
        .unwrap_or_else(|| first_visible_editor_line(app, app.edit))
        .clamp(0, max_line);
    let line_index = visible_line_at_or_after(app, sampled_line, max_line);
    let edit_origin = edit_client_origin_in_gutter(app.edit, app.gutter);
    let top = edit_origin.y + edit_line_y(app, app.edit, line_index).unwrap_or(sample_y);
    let height = measured_line_height_for_probe(app, line_index).max(1);
    let metrics = LineProbeMetrics {
        line_index,
        top,
        height,
        bottom: top + height,
    };
    app.line_probe_metrics = Some(metrics);

    let probe_text = format!(
        "line={};top={};height={};bottom={}",
        metrics.line_index + 1,
        metrics.top,
        metrics.height,
        metrics.bottom,
    );
    if probe_text != app.line_probe_cache {
        let wide = to_wide(&probe_text);
        unsafe {
            SendMessageW(app.line_probe, WM_SETTEXT, 0, wide.as_ptr() as Lparam);
        }
        app.line_probe_cache = probe_text;
    }
}

fn measured_line_height_for_probe(app: &AppData, line_index: i32) -> i32 {
    if let (Some(top), Some(next_top)) = (
        edit_line_y(app, app.edit, line_index),
        edit_line_y(app, app.edit, line_index + 1),
    ) {
        let height = next_top - top;
        if height > 0 {
            return height;
        }
    }

    let hdc = unsafe { GetDC(app.edit) };
    if hdc.is_null() {
        return 16;
    }
    let old_font = if app.font.is_null() {
        null_mut()
    } else {
        unsafe { SelectObject(hdc, app.font as Hgdiobj) }
    };
    let height = text_metric_line_height(hdc);
    if !old_font.is_null() {
        unsafe {
            SelectObject(hdc, old_font);
        }
    }
    unsafe {
        ReleaseDC(app.edit, hdc);
    }
    height
}

fn invalidate_window(hwnd: Hwnd) {
    if !hwnd.is_null() {
        unsafe {
            InvalidateRect(hwnd, null(), 0);
            UpdateWindow(hwnd);
        }
    }
}

fn invalidate_gutter(app: &AppData) {
    if app.line_numbers_visible && !app.gutter.is_null() {
        unsafe {
            InvalidateRect(app.gutter, null(), 0);
            UpdateWindow(app.gutter);
        }
    }
    if app.line_numbers_visible && app.compare_tab.is_some() && !app.compare_gutter.is_null() {
        unsafe {
            InvalidateRect(app.compare_gutter, null(), 0);
            UpdateWindow(app.compare_gutter);
        }
    }
}

fn invalidate_gutter_if_scrolled(app: &mut AppData) {
    ensure_gutter_sync(app);
    invalidate_gutter(app);
}

fn refresh_main_editor_view_state(app: &mut AppData) {
    ensure_gutter_sync(app);
    sync_line_probe(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

fn invalidate_status(app: &AppData) {
    if !app.status.is_null() {
        unsafe {
            InvalidateRect(app.status, null(), 0);
        }
    }
}

fn invalidate_status_text_regions(app: &AppData, _left_changed: bool, _right_changed: bool) {
    // The status bar can be split into left/right compare panels, so invalidate the
    // full bar instead of trying to guess stale text rectangles after splitter moves.
    invalidate_status(app);
}

fn apply_editor_colors(app: &AppData, hdc: Hdc) -> Lresult {
    let palette = app.theme.palette();

    unsafe {
        SetTextColor(hdc, palette.editor_text);
        SetBkColor(hdc, palette.editor_background);
    }

    app.editor_background_brush as Lresult
}

fn apply_rich_edit_theme(app: &AppData) {
    if !app.use_rich_edit {
        return;
    }

    let palette = app.theme.palette();
    for edit in [app.edit, app.compare_edit] {
        if edit.is_null() {
            continue;
        }

        unsafe {
            SendMessageW(
                edit,
                EM_SETBKGNDCOLOR,
                0,
                palette.editor_background as Lparam,
            );
        }
        apply_editor_char_format(app, edit);
    }
}

fn apply_native_window_theme(app: &AppData) {
    set_native_process_theme(app.theme);
    allow_native_dark_mode_for_window(app.hwnd, app.theme == Theme::Dark);
    set_dwm_window_dark_mode(app.hwnd, app.theme == Theme::Dark);
    flush_native_menu_themes();
    unsafe {
        DrawMenuBar(app.hwnd);
    }
}

fn set_native_process_theme(theme: Theme) {
    type SetPreferredAppMode = unsafe extern "system" fn(Int) -> Int;

    let Some(function) = uxtheme_ordinal(UXTHEME_ORDINAL_SET_PREFERRED_APP_MODE) else {
        return;
    };

    let mode = match theme {
        Theme::Dark => PREFERRED_APP_MODE_FORCE_DARK,
        Theme::Light => PREFERRED_APP_MODE_FORCE_LIGHT,
    };
    unsafe {
        let set_preferred_app_mode: SetPreferredAppMode = transmute(function);
        set_preferred_app_mode(mode);
    }
}

fn allow_native_dark_mode_for_window(hwnd: Hwnd, enabled: bool) {
    type AllowDarkModeForWindow = unsafe extern "system" fn(Hwnd, Bool) -> Bool;

    if hwnd.is_null() {
        return;
    }

    let Some(function) = uxtheme_ordinal(UXTHEME_ORDINAL_ALLOW_DARK_MODE_FOR_WINDOW) else {
        return;
    };

    unsafe {
        let allow_dark_mode_for_window: AllowDarkModeForWindow = transmute(function);
        allow_dark_mode_for_window(hwnd, enabled as Bool);
    }
}

fn flush_native_menu_themes() {
    type FlushMenuThemes = unsafe extern "system" fn();

    let Some(function) = uxtheme_ordinal(UXTHEME_ORDINAL_FLUSH_MENU_THEMES) else {
        return;
    };

    unsafe {
        let flush_menu_themes: FlushMenuThemes = transmute(function);
        flush_menu_themes();
    }
}

fn uxtheme_ordinal(ordinal: usize) -> Option<*mut c_void> {
    let library = to_wide("uxtheme.dll");
    let module = unsafe { LoadLibraryW(library.as_ptr()) };
    if module.is_null() {
        return None;
    }

    let proc = unsafe { GetProcAddress(module, ordinal as *const u8) };
    if proc.is_null() { None } else { Some(proc) }
}

fn set_dwm_window_dark_mode(hwnd: Hwnd, enabled: bool) {
    type DwmSetWindowAttribute =
        unsafe extern "system" fn(Hwnd, Dword, *const c_void, Dword) -> Long;

    if hwnd.is_null() {
        return;
    }

    let library = to_wide("dwmapi.dll");
    let module = unsafe { LoadLibraryW(library.as_ptr()) };
    if module.is_null() {
        return;
    }

    let function = unsafe { GetProcAddress(module, c"DwmSetWindowAttribute".as_ptr().cast()) };
    if function.is_null() {
        return;
    }

    let enabled: Bool = enabled as Bool;
    let value = (&enabled as *const Bool).cast::<c_void>();
    unsafe {
        let dwm_set_window_attribute: DwmSetWindowAttribute = transmute(function);
        let _ = dwm_set_window_attribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            value,
            size_of::<Bool>() as Dword,
        );
        let _ = dwm_set_window_attribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1,
            value,
            size_of::<Bool>() as Dword,
        );
    }
}

fn apply_editor_char_format(app: &AppData, edit: Hwnd) {
    if edit.is_null() || !app.use_rich_edit {
        return;
    }

    let palette = app.theme.palette();
    let mut format: CharFormat2W = unsafe { zeroed() };
    format.cbSize = size_of::<CharFormat2W>() as Uint;
    format.dwMask = CFM_FACE | CFM_SIZE | CFM_COLOR | CFM_BACKCOLOR | CFM_WEIGHT | CFM_ITALIC;
    format.dwEffects = if app.font_italic { CFE_ITALIC } else { 0 };
    format.yHeight = (app.font_size_pt * app.zoom_percent / 100).max(1) * 20;
    format.crTextColor = palette.editor_text;
    format.crBackColor = palette.editor_background;
    format.wWeight = app.font_weight as Word;
    copy_wide_string_into_fixed(&app.font_face, &mut format.szFaceName);

    unsafe {
        SendMessageW(
            edit,
            EM_SETCHARFORMAT,
            SCF_ALL,
            (&mut format as *mut CharFormat2W) as Lparam,
        );
        SendMessageW(
            edit,
            EM_SETCHARFORMAT,
            0,
            (&mut format as *mut CharFormat2W) as Lparam,
        );
    }
}

fn apply_editor_char_formats(app: &AppData) {
    apply_editor_char_format(app, app.edit);
    apply_editor_char_format(app, app.compare_edit);
}

fn is_editor_child(app: &AppData, hwnd: Hwnd) -> bool {
    hwnd == app.edit || hwnd == app.compare_edit
}

fn create_theme_editor_brush(theme: Theme) -> io::Result<Brush> {
    create_solid_brush(theme.palette().editor_background)
}

fn create_solid_brush(color: Dword) -> io::Result<Brush> {
    let brush = unsafe { CreateSolidBrush(color) };
    if brush.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(brush)
    }
}

fn fill_rect_with_color(hdc: Hdc, rect: &Rect, color: Dword) {
    if let Ok(brush) = create_solid_brush(color) {
        unsafe {
            FillRect(hdc, rect, brush);
            DeleteObject(brush as Hgdiobj);
        }
    }
}
