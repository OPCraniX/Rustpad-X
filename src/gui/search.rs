fn open_find_dialog(app: &mut AppData) {
    seed_find_buffer_from_selection(app);

    if !app.find_dialog.is_null() {
        unsafe {
            SetFocus(app.find_dialog);
        }
        return;
    }

    prepare_find_dialog_state(app);
    let dialog = unsafe { FindTextW(&mut app.find_state) };
    if dialog.is_null() {
        message_box(
            app.hwnd,
            "Could not open Find.",
            "Find",
            MB_OK | MB_ICONERROR,
        );
        return;
    }

    app.find_dialog = dialog;
}

fn open_replace_dialog(app: &mut AppData) {
    seed_find_buffer_from_selection(app);

    if !app.replace_dialog.is_null() {
        unsafe {
            SetFocus(app.replace_dialog);
        }
        return;
    }

    prepare_replace_dialog_state(app);
    let dialog = unsafe { ReplaceTextW(&mut app.replace_state) };
    if dialog.is_null() {
        message_box(
            app.hwnd,
            "Could not open Replace.",
            "Replace",
            MB_OK | MB_ICONERROR,
        );
        return;
    }

    app.replace_dialog = dialog;
}

fn prepare_find_dialog_state(app: &mut AppData) {
    app.find_state = FindReplaceW {
        lStructSize: size_of::<FindReplaceW>() as Dword,
        hwndOwner: app.hwnd,
        hInstance: null_mut(),
        Flags: find_dialog_flags(app.last_find_options),
        lpstrFindWhat: app.find_buffer.as_mut_ptr(),
        lpstrReplaceWith: null_mut(),
        wFindWhatLen: app.find_buffer.len() as Word,
        wReplaceWithLen: 0,
        lCustData: 0,
        lpfnHook: null_mut(),
        lpTemplateName: null(),
    };
}

fn prepare_replace_dialog_state(app: &mut AppData) {
    app.replace_state = FindReplaceW {
        lStructSize: size_of::<FindReplaceW>() as Dword,
        hwndOwner: app.hwnd,
        hInstance: null_mut(),
        Flags: find_dialog_flags(app.last_find_options),
        lpstrFindWhat: app.find_buffer.as_mut_ptr(),
        lpstrReplaceWith: app.replace_buffer.as_mut_ptr(),
        wFindWhatLen: app.find_buffer.len() as Word,
        wReplaceWithLen: app.replace_buffer.len() as Word,
        lCustData: 0,
        lpfnHook: null_mut(),
        lpTemplateName: null(),
    };
}

fn find_dialog_flags(options: FindOptions) -> Dword {
    let mut flags = 0;
    if options.down {
        flags |= FR_DOWN;
    }
    if options.match_case {
        flags |= FR_MATCHCASE;
    }
    if options.whole_word {
        flags |= FR_WHOLEWORD;
    }
    flags
}

fn handle_find_replace_message(app: &mut AppData, find_replace: *mut FindReplaceW) {
    if find_replace.is_null() {
        return;
    }

    let flags = unsafe { (*find_replace).Flags };
    if flags & FR_DIALOGTERM != 0 {
        let find_ptr = &mut app.find_state as *mut FindReplaceW;
        let replace_ptr = &mut app.replace_state as *mut FindReplaceW;
        if find_replace == find_ptr {
            app.find_dialog = null_mut();
        } else if find_replace == replace_ptr {
            app.replace_dialog = null_mut();
        }
        return;
    }

    let options = FindOptions {
        down: flags & FR_DOWN != 0,
        match_case: flags & FR_MATCHCASE != 0,
        whole_word: flags & FR_WHOLEWORD != 0,
    };
    app.last_find_options = options;

    if flags & FR_REPLACEALL != 0 {
        replace_all_matches(app, options);
        refocus_find_replace_dialog(app, find_replace);
    } else if flags & FR_REPLACE != 0 {
        replace_current_then_find(app, options);
        refocus_find_replace_dialog(app, find_replace);
    } else if flags & FR_FINDNEXT != 0 {
        find_with_options(app, options, true);
        refocus_find_replace_dialog(app, find_replace);
    }
}

fn refocus_find_replace_dialog(app: &AppData, find_replace: *mut FindReplaceW) {
    let find_ptr = &app.find_state as *const FindReplaceW as *mut FindReplaceW;
    let replace_ptr = &app.replace_state as *const FindReplaceW as *mut FindReplaceW;
    let dialog = if find_replace == find_ptr {
        app.find_dialog
    } else if find_replace == replace_ptr {
        app.replace_dialog
    } else {
        null_mut()
    };

    if !dialog.is_null() {
        unsafe {
            SetActiveWindow(dialog);
            SetFocus(dialog);
        }
    }
}

fn find_next_from_menu(app: &mut AppData) {
    let mut options = app.last_find_options;
    options.down = true;
    find_with_options(app, options, true);
}

fn find_previous_from_menu(app: &mut AppData) {
    let mut options = app.last_find_options;
    options.down = false;
    find_with_options(app, options, true);
}

fn find_with_options(app: &mut AppData, options: FindOptions, wrap: bool) -> bool {
    if find_buffer_text(app).is_empty() {
        seed_find_buffer_from_selection(app);
    }

    let needle = find_buffer_text(app);
    if needle.is_empty() {
        open_find_dialog(app);
        return false;
    }

    app.last_find_options = options;
    if let Some(text_match) = find_match_from_current(app, &needle, options, wrap) {
        select_text_match(app, text_match);
        true
    } else {
        show_find_not_found(app, &needle);
        false
    }
}

fn replace_current_then_find(app: &mut AppData, options: FindOptions) {
    if find_buffer_text(app).is_empty() {
        seed_find_buffer_from_selection(app);
    }

    let needle = find_buffer_text(app);
    if needle.is_empty() {
        open_replace_dialog(app);
        return;
    }

    let replaced_current = if current_selection_matches_find(app, &needle, options) {
        let replacement = replace_buffer_text(app);
        insert_text_at_caret(app, &replacement);
        true
    } else {
        false
    };

    if !find_with_options_silent(app, options, true) && !replaced_current {
        show_find_not_found(app, &needle);
    }
}

fn find_with_options_silent(app: &mut AppData, options: FindOptions, wrap: bool) -> bool {
    if find_buffer_text(app).is_empty() {
        seed_find_buffer_from_selection(app);
    }

    let needle = find_buffer_text(app);
    if needle.is_empty() {
        open_find_dialog(app);
        return false;
    }

    app.last_find_options = options;
    if let Some(text_match) = find_match_from_current(app, &needle, options, wrap) {
        select_text_match(app, text_match);
        true
    } else {
        false
    }
}

fn replace_all_matches(app: &mut AppData, options: FindOptions) {
    if find_buffer_text(app).is_empty() {
        seed_find_buffer_from_selection(app);
    }

    let needle = find_buffer_text(app);
    if needle.is_empty() {
        open_replace_dialog(app);
        return;
    }

    let replacement = replace_buffer_text(app);
    let replacement_wide = to_wide(&replacement);
    let replacement_len = utf16_len(&replacement);
    let mut caret = 0;
    let mut replaced = 0usize;
    let forward_options = FindOptions {
        down: true,
        match_case: options.match_case,
        whole_word: options.whole_word,
    };

    unsafe {
        SendMessageW(app.edit, WM_SETREDRAW, 0, 0);
        SendMessageW(app.edit, EM_SETSEL, 0, 0);
    }

    loop {
        let text = get_edit_text(app.edit);
        let Some(text_match) =
            find_match_in_text(&text, &needle, caret, forward_options, false)
        else {
            break;
        };

        unsafe {
            SendMessageW(
                app.edit,
                EM_SETSEL,
                text_match.start_utf16 as Wparam,
                text_match.end_utf16 as Lparam,
            );
            SendMessageW(
                app.edit,
                EM_REPLACESEL,
                1,
                replacement_wide.as_ptr() as Lparam,
            );
        }

        caret = text_match.start_utf16 + replacement_len;
        unsafe {
            SendMessageW(app.edit, EM_SETSEL, caret as Wparam, caret as Lparam);
        }
        replaced += 1;
    }

    unsafe {
        SendMessageW(app.edit, WM_SETREDRAW, 1, 0);
        InvalidateRect(app.edit, null(), 1);
        SetFocus(app.edit);
    }

    sync_active_document_text(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);

    let message = if replaced == 1 {
        "Replaced 1 occurrence.".to_string()
    } else {
        format!("Replaced {} occurrences.", format_number(replaced))
    };
    message_box(
        app.hwnd,
        &message,
        "Replace All",
        MB_OK | MB_ICONINFORMATION,
    );
}

fn find_match_from_current(
    app: &AppData,
    needle: &str,
    options: FindOptions,
    wrap: bool,
) -> Option<TextMatch> {
    let (selection_start, selection_end) = edit_selection(app.edit);
    let start = if options.down {
        selection_end
    } else {
        selection_start
    };

    rich_edit_find_match(app.edit, needle, start, options, wrap).or_else(|| {
        let text = get_edit_text(app.edit);
        find_match_in_text(&text, needle, start, options, wrap)
    })
}

fn rich_edit_find_match(
    edit: Hwnd,
    needle: &str,
    start: i32,
    options: FindOptions,
    wrap: bool,
) -> Option<TextMatch> {
    if needle.is_empty() {
        return None;
    }

    let text_len = edit_text_len(edit) as i32;
    if options.down {
        if let Some(text_match) = rich_edit_find_in_range(edit, needle, start, -1, options) {
            return Some(text_match);
        }

        if wrap && start > 0 {
            return rich_edit_find_in_range(edit, needle, 0, start, options);
        }
    } else {
        if let Some(text_match) = rich_edit_find_in_range(edit, needle, start, 0, options) {
            return Some(text_match);
        }

        if wrap && start < text_len {
            return rich_edit_find_in_range(edit, needle, text_len, 0, options);
        }
    }

    None
}

fn rich_edit_find_in_range(
    edit: Hwnd,
    needle: &str,
    cp_min: i32,
    cp_max: i32,
    options: FindOptions,
) -> Option<TextMatch> {
    let needle_wide = to_wide(needle);
    let mut find = FindTextExW {
        chrg: CharRange {
            cpMin: cp_min,
            cpMax: cp_max,
        },
        lpstrText: needle_wide.as_ptr(),
        chrgText: CharRange {
            cpMin: -1,
            cpMax: -1,
        },
    };

    let mut flags = 0;
    if options.down {
        flags |= FR_DOWN;
    }
    if options.match_case {
        flags |= FR_MATCHCASE;
    }
    if options.whole_word {
        flags |= FR_WHOLEWORD;
    }

    let result = unsafe {
        SendMessageW(
            edit,
            EM_FINDTEXTEXW,
            flags as Wparam,
            &mut find as *mut FindTextExW as Lparam,
        )
    };

    if result < 0 || find.chrgText.cpMin < 0 || find.chrgText.cpMax < 0 {
        return None;
    }

    Some(TextMatch {
        start_utf16: find.chrgText.cpMin,
        end_utf16: find.chrgText.cpMax,
    })
}

fn find_match_in_text(
    text: &str,
    needle: &str,
    start_utf16: i32,
    options: FindOptions,
    wrap: bool,
) -> Option<TextMatch> {
    if needle.is_empty() {
        return None;
    }

    if options.down {
        if let Some(text_match) = find_forward_match(text, needle, start_utf16, options) {
            return Some(text_match);
        }

        if wrap && start_utf16 > 0 {
            return find_forward_match(text, needle, 0, options);
        }
    } else {
        if let Some(text_match) = find_backward_match(text, needle, start_utf16, options) {
            return Some(text_match);
        }

        let text_len = utf16_len(text);
        if wrap && start_utf16 < text_len {
            return find_backward_match(text, needle, text_len, options);
        }
    }

    None
}

fn find_forward_match(
    text: &str,
    needle: &str,
    start_utf16: i32,
    options: FindOptions,
) -> Option<TextMatch> {
    let start_byte = byte_index_from_utf16_pos(text, start_utf16);
    for (offset, _) in text[start_byte..].char_indices() {
        let candidate_start = start_byte + offset;
        if let Some(text_match) = match_at(text, needle, candidate_start, options) {
            return Some(text_match);
        }
    }

    None
}

fn find_backward_match(
    text: &str,
    needle: &str,
    start_utf16: i32,
    options: FindOptions,
) -> Option<TextMatch> {
    let limit_byte = byte_index_from_utf16_pos(text, start_utf16);
    for (candidate_start, _) in text.char_indices().rev() {
        if candidate_start >= limit_byte {
            continue;
        }
        if let Some(text_match) = match_at(text, needle, candidate_start, options) {
            return Some(text_match);
        }
    }

    None
}

fn match_at(
    text: &str,
    needle: &str,
    start_byte: usize,
    options: FindOptions,
) -> Option<TextMatch> {
    let end_byte = byte_index_after_chars(text, start_byte, needle.chars().count())?;
    let candidate = &text[start_byte..end_byte];

    if !text_equals(candidate, needle, options.match_case) {
        return None;
    }

    if options.whole_word && !has_word_boundaries(text, start_byte, end_byte) {
        return None;
    }

    Some(TextMatch {
        start_utf16: utf16_pos_from_byte_index(text, start_byte),
        end_utf16: utf16_pos_from_byte_index(text, end_byte),
    })
}

fn current_selection_matches_find(app: &AppData, needle: &str, options: FindOptions) -> bool {
    let text = get_edit_text(app.edit);
    let (selection_start, selection_end) = edit_selection(app.edit);
    if selection_start == selection_end {
        return false;
    }

    let start_byte = byte_index_from_utf16_pos(&text, selection_start);
    let end_byte = byte_index_from_utf16_pos(&text, selection_end);
    let selected = &text[start_byte..end_byte];

    text_equals(selected, needle, options.match_case)
        && (!options.whole_word || has_word_boundaries(&text, start_byte, end_byte))
}

fn select_text_match(app: &mut AppData, text_match: TextMatch) {
    unsafe {
        SendMessageW(
            app.edit,
            EM_SETSEL,
            text_match.start_utf16 as Wparam,
            text_match.end_utf16 as Lparam,
        );
        SendMessageW(app.edit, EM_SCROLLCARET, 0, 0);
        SetFocus(app.edit);
    }
    refresh_main_editor_view_state(app);
}

fn show_find_not_found(app: &AppData, needle: &str) {
    message_box(
        app.hwnd,
        &format!("Cannot find \"{needle}\"."),
        "Find",
        MB_OK | MB_ICONINFORMATION,
    );
}

fn seed_find_buffer_from_selection(app: &mut AppData) {
    let selected = selected_edit_text(app);
    if selected.is_empty() || selected.contains('\r') || selected.contains('\n') {
        return;
    }

    write_string_to_wide_buffer(&mut app.find_buffer, &selected);
}

fn selected_edit_text(app: &AppData) -> String {
    let text = get_edit_text(app.edit);
    let (selection_start, selection_end) = edit_selection(app.edit);
    if selection_start == selection_end {
        return String::new();
    }

    let start_byte = byte_index_from_utf16_pos(&text, selection_start);
    let end_byte = byte_index_from_utf16_pos(&text, selection_end);
    text[start_byte..end_byte].to_string()
}

fn find_buffer_text(app: &AppData) -> String {
    wide_buffer_to_string(&app.find_buffer)
}

fn replace_buffer_text(app: &AppData) -> String {
    wide_buffer_to_string(&app.replace_buffer)
}

fn write_string_to_wide_buffer(buffer: &mut [u16], text: &str) {
    buffer.fill(0);
    if buffer.is_empty() {
        return;
    }

    let mut index = 0;
    for character in text.chars() {
        let mut units = [0u16; 2];
        let encoded = character.encode_utf16(&mut units);
        if index + encoded.len() >= buffer.len() {
            break;
        }

        for unit in encoded {
            buffer[index] = *unit;
            index += 1;
        }
    }
}

fn wide_buffer_to_string(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

fn text_equals(left: &str, right: &str, match_case: bool) -> bool {
    if match_case {
        left == right
    } else {
        left.eq_ignore_ascii_case(right) || left.to_lowercase() == right.to_lowercase()
    }
}

fn has_word_boundaries(text: &str, start_byte: usize, end_byte: usize) -> bool {
    let before = text[..start_byte].chars().next_back();
    let after = text[end_byte..].chars().next();

    !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn byte_index_from_utf16_pos(text: &str, utf16_pos: i32) -> usize {
    let target = utf16_pos.max(0) as usize;
    let mut current = 0usize;

    for (byte_index, character) in text.char_indices() {
        if current >= target {
            return byte_index;
        }
        current += character.len_utf16();
    }

    text.len()
}

fn utf16_pos_from_byte_index(text: &str, byte_index: usize) -> i32 {
    text[..byte_index.min(text.len())].encode_utf16().count() as i32
}

fn byte_index_after_chars(text: &str, start_byte: usize, char_count: usize) -> Option<usize> {
    if char_count == 0 {
        return Some(start_byte);
    }

    let mut seen = 0usize;
    for (offset, character) in text[start_byte..].char_indices() {
        seen += 1;
        if seen == char_count {
            return Some(start_byte + offset + character.len_utf8());
        }
    }

    None
}

fn utf16_len(text: &str) -> i32 {
    text.encode_utf16().count() as i32
}

fn choose_editor_font(app: &mut AppData) {
    let mut log_font = current_editor_log_font(app);
    let mut dialog = ChooseFontW {
        lStructSize: size_of::<ChooseFontW>() as Dword,
        hwndOwner: app.hwnd,
        hDC: null_mut(),
        lpLogFont: &mut log_font,
        iPointSize: app.font_size_pt * 10,
        Flags: CF_SCREENFONTS | CF_INITTOLOGFONTSTRUCT | CF_FORCEFONTEXIST,
        rgbColors: app.theme.palette().editor_text,
        lCustData: 0,
        lpfnHook: null_mut(),
        lpTemplateName: null(),
        hInstance: null_mut(),
        lpszStyle: null_mut(),
        nFontType: 0,
        ___MISSING_ALIGNMENT__: 0,
        nSizeMin: 0,
        nSizeMax: 0,
    };

    if unsafe { ChooseFontW(&mut dialog) } == 0 {
        return;
    }

    let face = fixed_wide_to_string(&log_font.lfFaceName);
    let face = if face.is_empty() {
        DEFAULT_FONT_FACE.to_string()
    } else {
        face
    };
    let points = if dialog.iPointSize > 0 {
        ((dialog.iPointSize + 5) / 10).max(1)
    } else {
        app.font_size_pt
    };
    let weight = log_font.lfWeight;
    let italic = log_font.lfItalic != 0;

    let new_font = match create_font_from_settings(
        app.hwnd,
        &face,
        points,
        app.zoom_percent,
        weight,
        italic,
    ) {
        Ok(font) => font,
        Err(error) => {
            message_box(
                app.hwnd,
                &format!("Could not change the editor font:\n\n{error}"),
                "Font",
                MB_OK | MB_ICONERROR,
            );
            return;
        }
    };

    let old_font = app.font;
    app.font = new_font;
    app.font_face = face;
    app.font_size_pt = points;
    app.font_weight = weight;
    app.font_italic = italic;

    unsafe {
        SendMessageW(app.edit, WM_SETFONT, app.font as Wparam, 1);
        SendMessageW(app.compare_edit, WM_SETFONT, app.font as Wparam, 1);
        apply_editor_char_formats(app);
        InvalidateRect(app.edit, null(), 1);
        InvalidateRect(app.compare_edit, null(), 1);
    }
    if !old_font.is_null() {
        unsafe {
            DeleteObject(old_font as Hgdiobj);
        }
    }

    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

fn current_editor_log_font(app: &AppData) -> LogFontW {
    let mut log_font: LogFontW = unsafe { zeroed() };
    log_font.lfHeight = font_height_for_points(app.hwnd, app.font_size_pt);
    log_font.lfWeight = app.font_weight;
    log_font.lfItalic = if app.font_italic { 1 } else { 0 };
    log_font.lfCharSet = DEFAULT_CHARSET as u8;
    log_font.lfOutPrecision = OUT_DEFAULT_PRECIS as u8;
    log_font.lfClipPrecision = CLIP_DEFAULT_PRECIS as u8;
    log_font.lfQuality = CLEARTYPE_QUALITY as u8;
    log_font.lfPitchAndFamily = FF_DONTCARE as u8;
    write_string_to_wide_buffer(&mut log_font.lfFaceName, &app.font_face);
    log_font
}

fn copy_wide_string_into_fixed(value: &str, destination: &mut [u16]) {
    if destination.is_empty() {
        return;
    }

    for item in destination.iter_mut() {
        *item = 0;
    }

    for (index, unit) in OsStr::new(value)
        .encode_wide()
        .take(destination.len().saturating_sub(1))
        .enumerate()
    {
        destination[index] = unit;
    }
}

fn fixed_wide_to_string(buffer: &[u16]) -> String {
    wide_buffer_to_string(buffer)
}

fn move_to_document_edge(app: &mut AppData, bottom: bool) {
    let position = if bottom {
        unsafe { SendMessageW(app.edit, WM_GETTEXTLENGTH, 0, 0).max(0) }
    } else {
        0
    };

    unsafe {
        SendMessageW(app.edit, EM_SETSEL, position as Wparam, position as Lparam);
        SendMessageW(app.edit, EM_SCROLLCARET, 0, 0);
        SetFocus(app.edit);
    }

    refresh_main_editor_view_state(app);
}

fn show_goto_line(app: &mut AppData) {
    let max_line = active_document_line_count(app);
    let Some(line) = prompt_for_goto_line(app.hwnd, max_line) else {
        unsafe {
            SetFocus(app.edit);
        }
        return;
    };

    jump_to_line(app, line);
}

fn prompt_for_goto_line(parent: Hwnd, max_line: usize) -> Option<usize> {
    let mut state = GotoLineState {
        parent,
        edit: null_mut(),
        max_line,
        result: None,
        done: false,
    };
    let class_name = to_wide(GOTO_LINE_CLASS_NAME);
    let title = to_wide("Go To Line");
    let width = 332;
    let height = 164;
    let (x, y) = centered_dialog_position(parent, width, height);

    let dialog = unsafe {
        CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            class_name.as_ptr(),
            title.as_ptr(),
            (WS_POPUP | WS_CAPTION | WS_SYSMENU) as Dword,
            x,
            y,
            width,
            height,
            parent,
            null_mut(),
            null_mut(),
            (&mut state as *mut GotoLineState).cast(),
        )
    };

    if dialog.is_null() {
        message_box(
            parent,
            "Could not open Go To Line.",
            "Go To Line",
            MB_OK | MB_ICONERROR,
        );
        return None;
    }

    unsafe {
        EnableWindow(parent, 0);
        ShowWindow(dialog, SW_SHOW);
        UpdateWindow(dialog);
    }

    let mut message: Msg = unsafe { zeroed() };
    while !state.done {
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result == -1 {
            break;
        }
        if result == 0 {
            unsafe {
                PostQuitMessage(0);
            }
            break;
        }

        if message.message == WM_KEYDOWN && message.hwnd == state.edit {
            match message.wParam as u16 {
                VK_RETURN => {
                    accept_goto_line(dialog);
                    continue;
                }
                VK_ESCAPE => {
                    close_goto_line_dialog(dialog);
                    continue;
                }
                _ => {}
            }
        }

        if unsafe { IsDialogMessageW(dialog, &mut message) } == 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    unsafe {
        EnableWindow(parent, 1);
    }

    state.result
}

fn centered_dialog_position(parent: Hwnd, width: i32, height: i32) -> (i32, i32) {
    let mut rect = empty_rect();
    if unsafe { GetWindowRect(parent, &mut rect) } == 0 {
        return centered_window_position(width, height);
    }

    let x = rect.left + ((rect.right - rect.left - width).max(0) / 2);
    let y = rect.top + ((rect.bottom - rect.top - height).max(0) / 2);
    (x, y)
}

fn jump_to_line(app: &mut AppData, line: usize) {
    let max_line = active_document_line_count(app);
    let line = line.clamp(1, max_line);
    let Some(position) = document_line_start_utf16(active_document(app), (line - 1) as i32)
    else {
        return;
    };

    unsafe {
        SendMessageW(app.edit, EM_SETSEL, position as Wparam, position as Lparam);
        SendMessageW(app.edit, EM_SCROLLCARET, 0, 0);
        SetFocus(app.edit);
    }

    refresh_main_editor_view_state(app);
}

fn current_line_number(parent: Hwnd) -> usize {
    let app = app_data_ptr(parent);
    if app.is_null() {
        return 1;
    }

    unsafe {
        let (line, _, _) = editor_position(&*app, (*app).edit);
        line
    }
}

fn date_text() -> String {
    let mut local_time: SystemTime = unsafe { zeroed() };
    unsafe {
        GetLocalTime(&mut local_time);
    }

    format!(
        "{:02}/{:02}/{:04}",
        local_time.wMonth, local_time.wDay, local_time.wYear
    )
}

fn time_and_date_text() -> String {
    let mut local_time: SystemTime = unsafe { zeroed() };
    unsafe {
        GetLocalTime(&mut local_time);
    }

    let suffix = if local_time.wHour >= 12 { "PM" } else { "AM" };
    let hour = match local_time.wHour % 12 {
        0 => 12,
        value => value,
    };

    format!(
        "{:02}:{:02} {} {:02}/{:02}/{:04}",
        hour, local_time.wMinute, suffix, local_time.wMonth, local_time.wDay, local_time.wYear
    )
}

fn clipboard_text(hwnd: Hwnd) -> io::Result<String> {
    if unsafe { OpenClipboard(hwnd) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) as Hglobal };
    if handle.is_null() {
        unsafe {
            CloseClipboard();
        }
        return Ok(String::new());
    }

    let byte_count = unsafe { GlobalSize(handle) };
    if byte_count < size_of::<u16>() {
        unsafe {
            CloseClipboard();
        }
        return Ok(String::new());
    }

    let ptr = unsafe { GlobalLock(handle) as *const u16 };
    if ptr.is_null() {
        let error = io::Error::last_os_error();
        unsafe {
            CloseClipboard();
        }
        return Err(error);
    }

    let max_units = byte_count / size_of::<u16>();
    let mut len = 0usize;
    unsafe {
        while len < max_units && *ptr.add(len) != 0 {
            len += 1;
        }
    }

    let text = unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)) };

    unsafe {
        GlobalUnlock(handle);
        CloseClipboard();
    }

    Ok(text)
}

fn edit_selection(edit: Hwnd) -> (i32, i32) {
    let mut selection_start: Dword = 0;
    let mut selection_end: Dword = 0;
    unsafe {
        SendMessageW(
            edit,
            EM_GETSEL,
            (&mut selection_start as *mut Dword) as Wparam,
            (&mut selection_end as *mut Dword) as Lparam,
        );
    }

    let start = clamp_selection_position(selection_start.min(selection_end));
    let end = clamp_selection_position(selection_start.max(selection_end));
    (start, end)
}

fn clamp_selection_position(position: Dword) -> i32 {
    position.min(i32::MAX as Dword) as i32
}

