pub fn run() -> io::Result<()> {
    let startup = startup_request();
    let use_rich_edit = load_rich_edit_library();
    set_native_process_theme(Theme::Dark);
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err(io::Error::last_os_error());
    }

    let large_icon = load_app_icon(instance, unsafe { GetSystemMetrics(SM_CXICON) }, unsafe {
        GetSystemMetrics(SM_CYICON)
    });
    let small_icon =
        load_app_icon(instance, unsafe { GetSystemMetrics(SM_CXSMICON) }, unsafe {
            GetSystemMetrics(SM_CYSMICON)
        });

    let class_name = to_wide(CLASS_NAME);
    let window_title = to_wide(APP_TITLE);
    let window_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: large_icon,
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: (COLOR_WINDOW + 1) as Brush,
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let menu_bar_class_name = to_wide(MENU_BAR_CLASS_NAME);
    let menu_bar_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(menu_bar_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: menu_bar_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&menu_bar_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let path_bar_class_name = to_wide(PATH_BAR_CLASS_NAME);
    let path_bar_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(path_bar_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: path_bar_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&path_bar_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let tab_bar_class_name = to_wide(TAB_BAR_CLASS_NAME);
    let tab_bar_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(tab_bar_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: tab_bar_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&tab_bar_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let gutter_class_name = to_wide(GUTTER_CLASS_NAME);
    let gutter_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(gutter_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: (COLOR_BTNFACE + 1) as Brush,
        lpszMenuName: null(),
        lpszClassName: gutter_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&gutter_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let status_class_name = to_wide(STATUS_CLASS_NAME);
    let status_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(status_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: status_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&status_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let splitter_class_name = to_wide(COMPARE_SPLITTER_CLASS_NAME);
    let splitter_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(compare_splitter_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: splitter_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&splitter_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let goto_line_class_name = to_wide(GOTO_LINE_CLASS_NAME);
    let goto_line_class = WndClassW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(goto_line_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: (COLOR_BTNFACE + 1) as Brush,
        lpszMenuName: null(),
        lpszClassName: goto_line_class_name.as_ptr(),
    };

    if unsafe { RegisterClassW(&goto_line_class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let menus = create_main_menu()?;

    let app_data = Box::into_raw(Box::new(AppDataCell::new(AppData::new(
        menus,
        use_rich_edit,
        startup,
    )?)));
    let (x, y, window_width, window_height) = saved_window_placement().unwrap_or_else(|| {
        let (x, y) = centered_window_position(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT);
        (x, y, INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
    });

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            (WS_OVERLAPPEDWINDOW | WS_VISIBLE) as Dword,
            x,
            y,
            window_width,
            window_height,
            null_mut(),
            null_mut(),
            instance,
            app_data as *mut c_void,
        )
    };

    if hwnd.is_null() {
        unsafe {
            drop(Box::from_raw(app_data));
        }
        return Err(io::Error::last_os_error());
    }

    unsafe {
        if !large_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG, large_icon as Lparam);
        }
        if !small_icon.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL, small_icon as Lparam);
        }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let accelerators = create_accelerators();
    let mut message: Msg = unsafe { zeroed() };

    loop {
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result == -1 {
            if !accelerators.is_null() {
                unsafe {
                    DestroyAcceleratorTable(accelerators);
                }
            }
            return Err(io::Error::last_os_error());
        }

        if result == 0 {
            break;
        }

        if translate_find_replace_dialog_message(hwnd, &mut message) {
            continue;
        }

        let handled = if accelerators.is_null() {
            0
        } else {
            unsafe { TranslateAcceleratorW(hwnd, accelerators, &mut message) }
        };

        if handled == 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    if !accelerators.is_null() {
        unsafe {
            DestroyAcceleratorTable(accelerators);
        }
    }

    Ok(())
}

pub fn show_fatal_error(message: &str) {
    message_box(null_mut(), message, "Rustpad-X Error", MB_OK | MB_ICONERROR);
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    if message != WM_NCCREATE {
        let handled_find_replace = with_app_data(hwnd, |app| {
            if app.find_message != 0 && message == app.find_message {
                handle_find_replace_message(app, lparam as *mut FindReplaceW);
                true
            } else {
                false
            }
        })
        .unwrap_or(false);

        if handled_find_replace {
            return 0;
        }
    }

    match message {
        WM_NCCREATE => {
            let create = lparam as *const CreateStructW;
            if create.is_null() {
                return 0;
            }

            let app_cell = unsafe { (*create).lpCreateParams as *mut AppDataCell };
            if app_cell.is_null() {
                return 0;
            }

            unsafe {
                (*(*app_cell).data.get()).hwnd = hwnd;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, app_cell as isize);
            }

            1
        }
        WM_CREATE => {
            let create_result = with_app_data(hwnd, |app| -> io::Result<()> {
                apply_native_window_theme(app);
                create_editor_controls(app)?;
                update_line_number_menu_item(app);
                update_word_wrap_menu_item(app);
                update_theme_menu_items(app);
                update_compare_page_sync_menu_item(app);
                update_recent_files_menu(app);
                refresh_theme(app);
                update_window_title(app);
                refresh_status_if_changed(app);
                unsafe {
                    DragAcceptFiles(hwnd, 1);
                    SetTimer(hwnd, STATUS_TIMER_ID, STATUS_TIMER_MS, null_mut());
                }
                Ok(())
            });

            if let Some(Err(error)) = create_result {
                message_box(
                    hwnd,
                    &format!("Could not create editor controls:\n\n{error}"),
                    "Rustpad-X Error",
                    MB_OK | MB_ICONERROR,
                );
                unsafe {
                    DestroyWindow(hwnd);
                }
            }
            0
        }
        WM_SIZE => {
            with_app_data(hwnd, |app| {
                let width = low_word(lparam as usize) as i32;
                let height = high_word(lparam as usize) as i32;
                layout_editor(app, width, height);
            });
            0
        }
        WM_SETFOCUS => {
            with_app_data(hwnd, |app| unsafe {
                SetFocus(app.edit);
            });
            0
        }
        WM_SYSKEYDOWN => {
            if open_menu_bar_popup_for_key(hwnd, wparam) {
                0
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_SYSCHAR => {
            if !key_is_down(VK_CONTROL) && menu_bar_index_for_key(wparam).is_some() {
                0
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_COMMAND => {
            with_app_data(hwnd, |app| {
                let command_id = low_word(wparam);
                if command_id as usize == EDIT_CONTROL_ID {
                    handle_edit_notification(app, high_word(wparam));
                } else if command_id as usize == COMPARE_EDIT_CONTROL_ID {
                    refresh_status_if_changed(app);
                    invalidate_gutter(app);
                } else {
                    handle_command(app, command_id);
                }
            });
            0
        }
        WM_DROPFILES => {
            with_app_data(hwnd, |app| {
                open_dropped_files(app, wparam as Hdrop);
            });
            0
        }
        WM_TIMER => {
            if wparam == STATUS_TIMER_ID {
                with_app_data(hwnd, |app| {
                    refresh_status_if_changed(app);
                    if app.compare_tab.is_some() && !app.compare_edit.is_null() {
                        invalidate_status(app);
                    }
                });
            } else if wparam == FOLD_REFRESH_TIMER_ID {
                unsafe {
                    KillTimer(hwnd, FOLD_REFRESH_TIMER_ID);
                }
                with_app_data(hwnd, |app| {
                    app.fold_refresh_timer_active = false;
                    run_pending_fold_refresh(app);
                });
            } else if wparam == MENU_SWITCH_TIMER_ID && poll_menu_bar_hover(hwnd) {
                unsafe {
                    EndMenu();
                }
            }
            0
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            let app = app_data_ptr(hwnd);
            if !app.is_null() && unsafe { is_editor_child(&*app, lparam as Hwnd) } {
                unsafe { apply_editor_colors(&*app, wparam as Hdc) }
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_MOUSEWHEEL => {
            if mouse_wheel_has_control(wparam) {
                with_app_data(hwnd, |app| {
                    adjust_zoom(app, mouse_wheel_zoom_delta(wparam));
                });
                0
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
        WM_CLOSE => {
            with_app_data(hwnd, |app| {
                save_session_state(app);
            });
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                DragAcceptFiles(hwnd, 0);
                KillTimer(hwnd, STATUS_TIMER_ID);
                KillTimer(hwnd, FOLD_REFRESH_TIMER_ID);
                PostQuitMessage(0);
            }
            0
        }
        WM_NCDESTROY => {
            let ptr = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *mut AppDataCell };
            if !ptr.is_null() {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn menu_bar_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_PAINT => {
            paint_menu_bar(hwnd);
            0
        }
        WM_LBUTTONDOWN => {
            let parent = unsafe { GetParent(hwnd) };
            let x = signed_low_word(lparam as usize);
            handle_menu_bar_click(parent, x);
            0
        }
        WM_MOUSEMOVE => {
            let parent = unsafe { GetParent(hwnd) };
            let x = signed_low_word(lparam as usize);
            if request_menu_bar_switch_from_x(parent, x) {
                unsafe {
                    EndMenu();
                }
            }
            0
        }
        WM_ERASEBKGND => 1,
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn path_bar_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_PAINT => {
            paint_path_bar(hwnd);
            0
        }
        WM_LBUTTONUP => {
            let parent = unsafe { GetParent(hwnd) };
            let x = signed_low_word(lparam as usize);
            with_app_data(parent, |app| {
                handle_path_bar_click(app, x);
            });
            0
        }
        WM_ERASEBKGND => 1,
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn tab_bar_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_PAINT => {
            paint_tab_bar(hwnd);
            0
        }
        WM_MOUSEMOVE => {
            let parent = unsafe { GetParent(hwnd) };
            let x = signed_low_word(lparam as usize);
            let y = signed_high_word(lparam as usize);
            with_app_data(parent, |app| {
                update_tab_hover(app, x, y);
            });
            0
        }
        WM_LBUTTONUP => {
            let parent = unsafe { GetParent(hwnd) };
            let x = signed_low_word(lparam as usize);
            let y = signed_high_word(lparam as usize);
            with_app_data(parent, |app| {
                if let Some(index) = tab_close_index_at_point(app, x, y) {
                    close_tab(app, index);
                } else if let Some(index) = tab_index_at_x(app, x) {
                    switch_to_tab(app, index);
                }
                update_tab_hover(app, x, y);
            });
            0
        }
        WM_RBUTTONUP => {
            let parent = unsafe { GetParent(hwnd) };
            let point = Point {
                x: signed_low_word(lparam as usize),
                y: signed_high_word(lparam as usize),
            };
            with_app_data(parent, |app| {
                show_tab_context_menu(app, point);
            });
            0
        }
        WM_ERASEBKGND => 1,
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn gutter_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_PAINT => {
            paint_line_gutter(hwnd);
            0
        }
        WM_MOUSEMOVE => 0,
        WM_LBUTTONUP => 0,
        WM_ERASEBKGND => 1,
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn status_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_PAINT => {
            paint_status_bar(hwnd);
            0
        }
        WM_ERASEBKGND => 1,
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn compare_splitter_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_PAINT => {
            paint_compare_splitter(hwnd);
            0
        }
        WM_LBUTTONDOWN => {
            let parent = unsafe { GetParent(hwnd) };
            with_app_data(parent, |app| {
                app.compare_dragging = true;
            });
            unsafe {
                SetCapture(hwnd);
            }
            0
        }
        WM_MOUSEMOVE => {
            if wparam as u16 & MK_LBUTTON != 0 {
                let parent = unsafe { GetParent(hwnd) };
                let local_x = signed_low_word(lparam as usize);
                with_app_data(parent, |app| {
                    if app.compare_dragging {
                        drag_compare_splitter(app, hwnd, local_x);
                    }
                });
            }
            0
        }
        WM_LBUTTONUP => {
            let parent = unsafe { GetParent(hwnd) };
            with_app_data(parent, |app| {
                app.compare_dragging = false;
            });
            unsafe {
                ReleaseCapture();
            }
            0
        }
        WM_ERASEBKGND => 1,
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn goto_line_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    match message {
        WM_NCCREATE => {
            let create = lparam as *const CreateStructW;
            if create.is_null() {
                return 0;
            }

            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*create).lpCreateParams as isize);
            }
            1
        }
        WM_CREATE => {
            create_goto_line_controls(hwnd);
            0
        }
        WM_SETFOCUS => {
            let state = goto_line_state(hwnd);
            if !state.is_null() {
                unsafe {
                    SetFocus((*state).edit);
                }
            }
            0
        }
        WM_COMMAND => {
            match low_word(wparam) {
                ID_GOTO_LINE_OK => accept_goto_line(hwnd),
                ID_GOTO_LINE_CANCEL => close_goto_line_dialog(hwnd),
                _ => {}
            }
            0
        }
        WM_KEYDOWN => {
            match wparam as u16 {
                VK_RETURN => accept_goto_line(hwnd),
                VK_ESCAPE => close_goto_line_dialog(hwnd),
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            close_goto_line_dialog(hwnd);
            0
        }
        WM_NCDESTROY => {
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn edit_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    if message == WM_ERASEBKGND {
        let parent = unsafe { GetParent(hwnd) };
        let app = app_data_ptr(parent);
        if !app.is_null() && unsafe { is_editor_child(&*app, hwnd) } {
            let mut rect = empty_rect();
            unsafe {
                GetClientRect(hwnd, &mut rect);
                FillRect(wparam as Hdc, &rect, (*app).editor_background_brush);
            }
            return 1;
        }
    }

    if message == WM_CONTEXTMENU {
        let parent = unsafe { GetParent(hwnd) };
        let point = if lparam == -1 {
            let mut cursor = Point { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut cursor);
            }
            cursor
        } else {
            Point {
                x: signed_low_word(lparam as usize),
                y: signed_high_word(lparam as usize),
            }
        };

        with_app_data(parent, |app| {
            if hwnd == app.edit {
                show_editor_context_menu(app, point);
            }
        });
        return 0;
    }

    if message == WM_MOUSEWHEEL && mouse_wheel_has_control(wparam) {
        let parent = unsafe { GetParent(hwnd) };
        with_app_data(parent, |app| {
            adjust_zoom(app, mouse_wheel_zoom_delta(wparam));
        });
        return 0;
    }

    if message == WM_MOUSEWHEEL {
        let line_delta = mouse_wheel_scroll_lines(wparam);
        if line_delta != 0 {
            let before = first_visible_line(hwnd);
            unsafe {
                SendMessageW(hwnd, EM_LINESCROLL, 0, line_delta as Lparam);
            }
            let after = first_visible_line(hwnd);
            sync_compare_page_scroll(hwnd, after - before);
            refresh_editor_scroll_visuals(hwnd);
            return 0;
        }
    }

    if message == WM_SYSKEYDOWN {
        let parent = unsafe { GetParent(hwnd) };
        if open_menu_bar_popup_for_key(parent, wparam) {
            return 0;
        }
    }

    if message == WM_SYSCHAR
        && !key_is_down(VK_CONTROL)
        && menu_bar_index_for_key(wparam).is_some()
    {
        return 0;
    }

    if message == WM_KEYDOWN && handle_editor_key_command(hwnd, wparam) {
        return 0;
    }

    if message == WM_CHAR && handle_editor_char(hwnd, wparam) {
        return 0;
    }

    if message == WM_KEYDOWN && should_sync_compare_page_key(hwnd, wparam) {
        let result = call_previous_edit_proc(hwnd, message, wparam, lparam);
        sync_compare_page_key(hwnd, message, wparam, lparam);
        refresh_editor_visuals(hwnd);
        refresh_editor_scroll_visuals(hwnd);
        return result;
    }

    let before = if matches!(message, WM_MOUSEWHEEL | WM_VSCROLL)
        && should_sync_compare_page_scroll(hwnd)
    {
        Some(first_visible_line(hwnd))
    } else {
        None
    };
    let result = call_previous_edit_proc(hwnd, message, wparam, lparam);
    if let Some(before) = before {
        let after = first_visible_line(hwnd);
        sync_compare_page_scroll(hwnd, after - before);
    }
    if matches!(message, WM_KEYDOWN | WM_LBUTTONUP) {
        refresh_editor_visuals(hwnd);
        refresh_editor_scroll_visuals(hwnd);
    } else if matches!(message, WM_MOUSEWHEEL | WM_VSCROLL | WM_HSCROLL) {
        refresh_editor_scroll_visuals(hwnd);
    }
    result
}

fn call_previous_edit_proc(
    hwnd: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    let previous = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if previous != 0 {
        unsafe { CallWindowProcW(previous, hwnd, message, wparam, lparam) }
    } else {
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}

fn create_goto_line_controls(hwnd: Hwnd) {
    let state = goto_line_state(hwnd);
    if state.is_null() {
        return;
    }

    let static_class = to_wide("STATIC");
    let edit_class = to_wide("EDIT");
    let button_class = to_wide("BUTTON");
    let label = to_wide("Line number:");
    let range = unsafe { to_wide(&format!("Enter a line from 1 to {}", (*state).max_line)) };
    let initial = unsafe { to_wide(&current_line_number((*state).parent).to_string()) };
    let ok = to_wide("OK");
    let cancel = to_wide("Cancel");
    let gui_font = unsafe { GetStockObject(DEFAULT_GUI_FONT) as Hfont };

    let label_hwnd = unsafe {
        CreateWindowExW(
            0,
            static_class.as_ptr(),
            label.as_ptr(),
            (WS_CHILD | WS_VISIBLE | SS_LEFT) as Dword,
            14,
            16,
            290,
            18,
            hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    let edit_hwnd = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            initial.as_ptr(),
            (WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL | ES_NUMBER)
                as Dword,
            14,
            38,
            290,
            24,
            hwnd,
            ID_GOTO_LINE_EDIT as Hmenu,
            null_mut(),
            null_mut(),
        )
    };

    let range_hwnd = unsafe {
        CreateWindowExW(
            0,
            static_class.as_ptr(),
            range.as_ptr(),
            (WS_CHILD | WS_VISIBLE | SS_LEFT) as Dword,
            14,
            68,
            290,
            18,
            hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        )
    };

    let ok_hwnd = unsafe {
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            ok.as_ptr(),
            (WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON) as Dword,
            134,
            98,
            80,
            26,
            hwnd,
            ID_GOTO_LINE_OK as Hmenu,
            null_mut(),
            null_mut(),
        )
    };

    let cancel_hwnd = unsafe {
        CreateWindowExW(
            0,
            button_class.as_ptr(),
            cancel.as_ptr(),
            (WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON) as Dword,
            224,
            98,
            80,
            26,
            hwnd,
            ID_GOTO_LINE_CANCEL as Hmenu,
            null_mut(),
            null_mut(),
        )
    };

    unsafe {
        (*state).edit = edit_hwnd;
        for child in [label_hwnd, edit_hwnd, range_hwnd, ok_hwnd, cancel_hwnd] {
            if !child.is_null() && !gui_font.is_null() {
                SendMessageW(child, WM_SETFONT, gui_font as Wparam, 1);
            }
        }
        SendMessageW(edit_hwnd, EM_SETSEL, 0, -1);
        SetFocus(edit_hwnd);
    }
}

fn goto_line_state(hwnd: Hwnd) -> *mut GotoLineState {
    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut GotoLineState }
}

fn accept_goto_line(hwnd: Hwnd) {
    let state = goto_line_state(hwnd);
    if state.is_null() {
        return;
    }

    let input = unsafe { window_text((*state).edit).trim().to_string() };
    let max_line = unsafe { (*state).max_line };
    let Ok(line) = input.parse::<usize>() else {
        show_goto_line_error(hwnd, max_line);
        return;
    };

    if line == 0 || line > max_line {
        show_goto_line_error(hwnd, max_line);
        return;
    }

    unsafe {
        (*state).result = Some(line);
    }
    close_goto_line_dialog(hwnd);
}

fn show_goto_line_error(hwnd: Hwnd, max_line: usize) {
    message_box(
        hwnd,
        &format!(
            "The line number must be between 1 and {}.",
            format_number(max_line)
        ),
        "Go To Line",
        MB_OK | MB_ICONERROR,
    );

    let state = goto_line_state(hwnd);
    if !state.is_null() {
        unsafe {
            SendMessageW((*state).edit, EM_SETSEL, 0, -1);
            SetFocus((*state).edit);
        }
    }
}

fn close_goto_line_dialog(hwnd: Hwnd) {
    let state = goto_line_state(hwnd);
    if !state.is_null() {
        unsafe {
            (*state).done = true;
        }
    }
    unsafe {
        DestroyWindow(hwnd);
    }
}

