fn refresh_status_if_changed(app: &mut AppData) {
    sync_line_probe(app);
    let left = status_left_text(app);
    let right = status_right_text(app);
    let left_changed = left != app.status_left_cache;
    let right_changed = right != app.status_right_cache;

    if left_changed || right_changed {
        app.status_left_cache = left;
        app.status_right_cache = right;
        invalidate_status_text_regions(app, left_changed, right_changed);
    }
}

fn invalidate_document_chrome(app: &AppData) {
    unsafe {
        InvalidateRect(app.menu_bar, null(), 0);
        InvalidateRect(app.path_bar, null(), 1);
        InvalidateRect(app.tab_bar, null(), 1);
    }
}

fn paint_menu_bar(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }

    let app = app_data_ptr(parent);
    if app.is_null() {
        fill_rect_with_color(hdc, &rect, Theme::Dark.palette().path_background);
    } else {
        unsafe {
            draw_menu_bar(&*app, hdc, rect);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn draw_menu_bar(app: &AppData, hdc: Hdc, rect: Rect) {
    let palette = app.theme.palette();
    fill_rect_with_color(hdc, &rect, palette.path_background);

    let old_font = select_gui_font(hdc);
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, palette.path_text);
    }

    let mut x = rect.left + 2;
    for item in menu_bar_items(app) {
        let width = menu_bar_item_width(hdc, item.label);
        let mut item_rect = Rect {
            left: x,
            top: rect.top,
            right: (x + width).min(rect.right),
            bottom: rect.bottom,
        };

        if app.active_menu_index == Some(item.index) {
            fill_rect_with_color(hdc, &item_rect, palette.tab_inactive_background);
            unsafe {
                SetTextColor(hdc, palette.tab_active_text);
            }
        } else {
            unsafe {
                SetTextColor(hdc, palette.path_text);
            }
        }
        draw_single_line(hdc, item.label, &mut item_rect, 0);
        x += width;
        if x >= rect.right {
            break;
        }
    }

    restore_font(hdc, old_font);
}

fn handle_menu_bar_click(parent: Hwnd, x: i32) {
    if menu_bar_popup_is_active(parent) {
        request_menu_bar_switch_from_x(parent, x);
        unsafe {
            EndMenu();
        }
    } else {
        open_menu_bar_popup_from_x(parent, x);
    }
}

fn menu_bar_popup_is_active(parent: Hwnd) -> bool {
    with_app_data(parent, |app| app.menu_popup_active).unwrap_or(false)
}

fn open_menu_bar_popup_for_key(parent: Hwnd, wparam: Wparam) -> bool {
    if key_is_down(VK_CONTROL) {
        return false;
    }

    let Some(index) = menu_bar_index_for_key(wparam) else {
        return false;
    };

    open_menu_bar_popup(parent, index);
    true
}

fn menu_bar_index_for_key(wparam: Wparam) -> Option<usize> {
    match (wparam as u32) & 0xFFFF {
        key if key == b'F' as u32 || key == b'f' as u32 => Some(0),
        key if key == b'E' as u32 || key == b'e' as u32 => Some(1),
        key if key == b'V' as u32 || key == b'v' as u32 => Some(2),
        key if key == b'H' as u32 || key == b'h' as u32 => Some(3),
        _ => None,
    }
}

fn open_menu_bar_popup_from_x(parent: Hwnd, x: i32) {
    let index = with_app_data(parent, |app| menu_bar_index_at_x(app, x)).flatten();
    if let Some(index) = index {
        open_menu_bar_popup(parent, index);
    }
}

fn open_menu_bar_popup(parent: Hwnd, start_index: usize) {
    let mut next_index = Some(start_index);

    while let Some(index) = next_index.take() {
        let Some((menu, owner, x, y)) = prepare_menu_bar_popup(parent, index) else {
            return;
        };

        unsafe {
            SetTimer(
                owner,
                MENU_SWITCH_TIMER_ID,
                MENU_SWITCH_TIMER_MS,
                null_mut(),
            );
        }

        let command = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_RETURNCMD,
                x,
                y,
                0,
                owner,
                null(),
            )
        };

        unsafe {
            KillTimer(owner, MENU_SWITCH_TIMER_ID);
        }

        let pending_index = finish_menu_bar_popup(parent);
        if let Some(pending_index) = pending_index {
            next_index = Some(pending_index);
            continue;
        }

        if command != 0 {
            with_app_data(parent, |app| {
                handle_command(app, command as u16);
            });
        }
    }
}

fn prepare_menu_bar_popup(parent: Hwnd, index: usize) -> Option<(Hmenu, Hwnd, i32, i32)> {
    with_app_data(parent, |app| {
        let item = menu_bar_item_by_index(app, index)?;
        let (left, _right) = menu_bar_item_bounds(app, index)?;
        let mut point = Point {
            x: left,
            y: MENU_BAR_HEIGHT,
        };

        app.active_menu_index = Some(index);
        app.pending_menu_index = None;
        app.menu_popup_active = true;

        unsafe {
            ClientToScreen(app.menu_bar, &mut point);
            InvalidateRect(app.menu_bar, null(), 0);
        }

        Some((item.menu, app.hwnd, point.x, point.y))
    })
    .flatten()
}

fn finish_menu_bar_popup(parent: Hwnd) -> Option<usize> {
    with_app_data(parent, |app| {
        let pending_index = app.pending_menu_index.take();
        app.active_menu_index = None;
        app.menu_popup_active = false;

        unsafe {
            InvalidateRect(app.menu_bar, null(), 0);
        }

        pending_index
    })
    .flatten()
}

fn request_menu_bar_switch_from_x(parent: Hwnd, x: i32) -> bool {
    with_app_data(parent, |app| {
        if !app.menu_popup_active {
            return false;
        }

        let Some(index) = menu_bar_index_at_x(app, x) else {
            return false;
        };

        if app.active_menu_index == Some(index) {
            return false;
        }

        app.pending_menu_index = Some(index);
        true
    })
    .unwrap_or(false)
}

fn poll_menu_bar_hover(parent: Hwnd) -> bool {
    let mut point = Point { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut point) } == 0 {
        return false;
    }

    with_app_data(parent, |app| {
        if !app.menu_popup_active {
            return false;
        }

        let mut local_point = point;
        let mut rect = empty_rect();
        unsafe {
            if ScreenToClient(app.menu_bar, &mut local_point) == 0 {
                return false;
            }
            GetClientRect(app.menu_bar, &mut rect);
        }

        if local_point.x < rect.left
            || local_point.x >= rect.right
            || local_point.y < rect.top
            || local_point.y >= rect.bottom
        {
            return false;
        }

        let Some(index) = menu_bar_index_at_x(app, local_point.x) else {
            return false;
        };

        if app.active_menu_index == Some(index) {
            return false;
        }

        app.pending_menu_index = Some(index);
        true
    })
    .unwrap_or(false)
}

fn menu_bar_item_by_index(app: &AppData, index: usize) -> Option<MenuBarItem> {
    menu_bar_items(app)
        .iter()
        .copied()
        .find(|item| item.index == index)
}

fn menu_bar_item_bounds(app: &AppData, index: usize) -> Option<(i32, i32)> {
    let hdc = unsafe { GetDC(app.menu_bar) };
    if hdc.is_null() {
        return None;
    }

    let old_font = select_gui_font(hdc);
    let mut left = 2;
    let mut bounds = None;
    for item in menu_bar_items(app) {
        let right = left + menu_bar_item_width(hdc, item.label);
        if item.index == index {
            bounds = Some((left, right));
            break;
        }
        left = right;
    }

    restore_font(hdc, old_font);
    unsafe {
        ReleaseDC(app.menu_bar, hdc);
    }

    bounds
}

fn menu_bar_index_at_x(app: &AppData, x: i32) -> Option<usize> {
    let hdc = unsafe { GetDC(app.menu_bar) };
    if hdc.is_null() {
        return None;
    }

    let old_font = select_gui_font(hdc);
    let mut left = 2;
    let mut hit = None;
    for item in menu_bar_items(app) {
        let right = left + menu_bar_item_width(hdc, item.label);
        if x >= left && x < right {
            hit = Some(item.index);
            break;
        }
        left = right;
    }

    restore_font(hdc, old_font);
    unsafe {
        ReleaseDC(app.menu_bar, hdc);
    }

    hit
}

fn menu_bar_item_width(hdc: Hdc, label: &str) -> i32 {
    measure_text_width(hdc, label) + 24
}

fn menu_bar_items(app: &AppData) -> [MenuBarItem; 4] {
    [
        MenuBarItem {
            index: 0,
            label: " File",
            menu: app.file_menu,
        },
        MenuBarItem {
            index: 1,
            label: " Edit",
            menu: app.edit_menu,
        },
        MenuBarItem {
            index: 2,
            label: " View",
            menu: app.view_menu,
        },
        MenuBarItem {
            index: 3,
            label: " Help",
            menu: app.help_menu,
        },
    ]
}

fn paint_path_bar(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }

    let app = app_data_ptr(parent);
    if app.is_null() {
        fill_rect_with_color(hdc, &rect, Theme::Dark.palette().path_background);
    } else {
        unsafe {
            let palette = (*app).theme.palette();
            fill_rect_with_color(hdc, &rect, palette.path_background);
            draw_path_bar_text(&*app, hdc, rect);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn draw_path_bar_text(app: &AppData, hdc: Hdc, rect: Rect) {
    let palette = app.theme.palette();
    let old_font = select_gui_font(hdc);
    let name = active_file_name(app);
    let location = active_file_location(app);
    let separator = "  >  ";

    let name_width = measure_text_width(hdc, &name);
    let separator_width = measure_text_width(hdc, separator);
    let left = rect.left + 10;
    let center_y = rect.top;
    let name_right = (left + name_width).min(rect.right - 8);

    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, palette.path_text);
    }

    let mut name_rect = Rect {
        left,
        top: center_y,
        right: name_right,
        bottom: rect.bottom,
    };
    draw_single_line(hdc, &name, &mut name_rect, DT_END_ELLIPSIS);

    let mut separator_rect = Rect {
        left: name_right,
        top: center_y,
        right: (name_right + separator_width).min(rect.right - 8),
        bottom: rect.bottom,
    };
    unsafe {
        SetTextColor(hdc, palette.path_muted_text);
    }
    draw_single_line(hdc, separator, &mut separator_rect, 0);

    let mut location_rect = Rect {
        left: separator_rect.right,
        top: center_y,
        right: rect.right - 8,
        bottom: rect.bottom,
    };
    draw_single_line(hdc, &location, &mut location_rect, DT_END_ELLIPSIS);

    restore_font(hdc, old_font);
}

fn paint_copied_indicator(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }

    let app = app_data_ptr(parent);
    if app.is_null() {
        fill_rect_with_color(hdc, &rect, Theme::Dark.palette().tab_active_background);
    } else {
        unsafe {
            draw_copied_indicator(&*app, hdc, rect);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn paint_tab_bar(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }

    let app = app_data_ptr(parent);
    if app.is_null() {
        fill_rect_with_color(hdc, &rect, Theme::Dark.palette().tab_strip_background);
    } else {
        unsafe {
            let palette = (*app).theme.palette();
            fill_rect_with_color(hdc, &rect, palette.tab_strip_background);
            draw_tabs(&*app, hdc, rect);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn draw_tabs(app: &AppData, hdc: Hdc, rect: Rect) {
    let palette = app.theme.palette();
    let old_font = select_gui_font(hdc);
    let mut x = rect.left;

    for (index, document) in app.documents.iter().enumerate() {
        if x >= rect.right {
            break;
        }

        let active = index == app.active_tab;
        let hovered = app.tab_hover_index == Some(index);
        let close_hovered = app.tab_hover_close_index == Some(index);
        let label = document.tab_label();
        let width = tab_width(hdc, &label);
        let right = (x + width).min(rect.right);
        let tab_rect = Rect {
            left: x,
            top: rect.top,
            right,
            bottom: rect.bottom,
        };

        let mut background = if active {
            palette.tab_active_background
        } else {
            palette.tab_inactive_background
        };
        if hovered && !active {
            background = lighten_color(background, 0x00101010);
        }

        fill_rect_with_color(hdc, &tab_rect, background);

        let border = Rect {
            left: right - 1,
            top: tab_rect.top + 3,
            right,
            bottom: tab_rect.bottom - 3,
        };
        fill_rect_with_color(hdc, &border, palette.tab_border);

        let close_rect = tab_close_rect(tab_rect);
        let mut text_rect = Rect {
            left: tab_rect.left + 14,
            top: tab_rect.top,
            right: close_rect.left - 6,
            bottom: tab_rect.bottom,
        };

        unsafe {
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(
                hdc,
                if active {
                    palette.tab_active_text
                } else {
                    palette.tab_text
                },
            );
        }
        draw_single_line(hdc, &label, &mut text_rect, DT_END_ELLIPSIS);

        draw_tab_close_button(app, hdc, close_rect, active, close_hovered);

        x += width;
    }

    restore_font(hdc, old_font);
}

fn draw_tab_close_button(app: &AppData, hdc: Hdc, rect: Rect, active: bool, hovered: bool) {
    let palette = app.theme.palette();
    let mut button_rect = rect;
    if hovered {
        fill_rect_with_color(hdc, &button_rect, palette.tab_border);
    }

    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(
            hdc,
            if active {
                palette.tab_active_text
            } else {
                palette.tab_text
            },
        );
    }

    draw_single_line(
        hdc,
        "×",
        &mut button_rect,
        DT_SINGLELINE | DT_VCENTER | DT_CENTER,
    );
}

fn tab_close_rect(tab_rect: Rect) -> Rect {
    let top = tab_rect.top + ((tab_rect.bottom - tab_rect.top - TAB_CLOSE_BUTTON_SIZE) / 2);
    Rect {
        left: tab_rect.right - TAB_CLOSE_BUTTON_MARGIN - TAB_CLOSE_BUTTON_SIZE,
        top,
        right: tab_rect.right - TAB_CLOSE_BUTTON_MARGIN,
        bottom: top + TAB_CLOSE_BUTTON_SIZE,
    }
}

fn lighten_color(color: Dword, amount: Dword) -> Dword {
    let blue = ((color & 0x000000FF) + (amount & 0x000000FF)).min(0xFF);
    let green = (((color >> 8) & 0xFF) + ((amount >> 8) & 0xFF)).min(0xFF);
    let red = (((color >> 16) & 0xFF) + ((amount >> 16) & 0xFF)).min(0xFF);
    blue | (green << 8) | (red << 16)
}

fn adjust_zoom(app: &mut AppData, delta_percent: i32) {
    if delta_percent == 0 {
        return;
    }

    let target = (app.zoom_percent + delta_percent).clamp(MIN_ZOOM_PERCENT, MAX_ZOOM_PERCENT);
    if target == app.zoom_percent {
        return;
    }

    if set_zoom(app, target).is_err() {
        message_box(
            app.hwnd,
            "Could not change the zoom level.",
            "Zoom",
            MB_OK | MB_ICONERROR,
        );
    }
}

fn set_zoom(app: &mut AppData, zoom_percent: i32) -> io::Result<()> {
    let new_font = create_font_from_settings(
        app.hwnd,
        &app.font_face,
        app.font_size_pt,
        zoom_percent,
        app.font_weight,
        app.font_italic,
    )?;
    let old_font = app.font;
    app.font = new_font;
    app.zoom_percent = zoom_percent;

    unsafe {
        SendMessageW(app.edit, WM_SETFONT, new_font as Wparam, 1);
        SendMessageW(app.compare_edit, WM_SETFONT, new_font as Wparam, 1);
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
    invalidate_status(app);

    Ok(())
}

fn create_editor_font(app: &AppData) -> io::Result<Hfont> {
    create_font_from_settings(
        app.hwnd,
        &app.font_face,
        app.font_size_pt,
        app.zoom_percent,
        app.font_weight,
        app.font_italic,
    )
}

fn create_font_from_settings(
    hwnd: Hwnd,
    face_name: &str,
    points: i32,
    zoom_percent: i32,
    weight: Int,
    italic: bool,
) -> io::Result<Hfont> {
    let face = to_wide(face_name);
    let height = font_height_for_zoom(hwnd, points, zoom_percent);
    let font = unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            if italic { 1 } else { 0 },
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            FF_DONTCARE,
            face.as_ptr(),
        )
    };

    if font.is_null() {
        Err(io::Error::last_os_error())
    } else {
        Ok(font)
    }
}

fn font_height_for_zoom(hwnd: Hwnd, points: i32, zoom_percent: i32) -> i32 {
    let scaled_points = (points.max(1) * zoom_percent + 50) / 100;
    font_height_for_points(hwnd, scaled_points.max(1))
}

fn font_height_for_points(hwnd: Hwnd, points: i32) -> i32 {
    let hdc = unsafe { GetDC(hwnd) };
    let dpi = if hdc.is_null() {
        96
    } else {
        let value = unsafe { GetDeviceCaps(hdc, LOGPIXELSY) };
        unsafe {
            ReleaseDC(hwnd, hdc);
        }

        if value > 0 { value } else { 96 }
    };

    -((points.max(1) * dpi + 36) / 72)
}

fn mouse_wheel_has_control(wparam: Wparam) -> bool {
    low_word(wparam) & MK_CONTROL != 0
}

fn mouse_wheel_zoom_delta(wparam: Wparam) -> i32 {
    match signed_high_word(wparam) {
        delta if delta > 0 => ZOOM_STEP_PERCENT,
        delta if delta < 0 => -ZOOM_STEP_PERCENT,
        _ => 0,
    }
}

fn mouse_wheel_scroll_lines(wparam: Wparam) -> i32 {
    match signed_high_word(wparam) {
        delta if delta > 0 => -3,
        delta if delta < 0 => 3,
        _ => 0,
    }
}

fn paint_compare_splitter(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }

    let app = app_data_ptr(parent);
    let palette = if app.is_null() {
        Theme::Dark.palette()
    } else {
        unsafe { (*app).theme.palette() }
    };

    fill_rect_with_color(hdc, &rect, palette.gutter_separator);
    let grip = Rect {
        left: rect.left + (rect.right - rect.left) / 2,
        top: rect.top,
        right: rect.left + (rect.right - rect.left) / 2 + 1,
        bottom: rect.bottom,
    };
    fill_rect_with_color(hdc, &grip, palette.gutter_text);

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn drag_compare_splitter(app: &mut AppData, splitter: Hwnd, local_x: i32) {
    let mut splitter_rect = empty_rect();
    let mut client_rect = empty_rect();
    if unsafe { GetWindowRect(splitter, &mut splitter_rect) } == 0 {
        return;
    }
    if unsafe { GetClientRect(app.hwnd, &mut client_rect) } == 0 {
        return;
    }

    let mut splitter_origin = Point {
        x: splitter_rect.left,
        y: splitter_rect.top,
    };
    unsafe {
        ScreenToClient(app.hwnd, &mut splitter_origin);
    }

    let width = (client_rect.right - client_rect.left).max(1);
    let desired = (splitter_origin.x + local_x).clamp(0, width);
    app.compare_split_ratio = (desired * 10_000 / width).clamp(1_000, 9_000);

    layout_editor(app, width, client_rect.bottom - client_rect.top);
}

fn paint_line_gutter(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }

    let app = app_data_ptr(parent);
    if app.is_null() {
        fill_rect_with_color(hdc, &rect, Theme::Dark.palette().gutter_background);
    } else {
        unsafe {
            let palette = (*app).theme.palette();
            fill_rect_with_color(hdc, &rect, palette.gutter_background);
            let edit = if hwnd == (*app).compare_gutter {
                (*app).compare_edit
            } else {
                (*app).edit
            };
            draw_line_numbers(&*app, hdc, rect, edit, hwnd);

            let separator = Rect {
                left: (rect.right - 1).max(rect.left),
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            };
            fill_rect_with_color(hdc, &separator, palette.gutter_separator);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn paint_status_bar(hwnd: Hwnd) {
    let parent = unsafe { GetParent(hwnd) };
    let mut paint: PaintStruct = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };

    if hdc.is_null() {
        return;
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(hwnd, &mut rect);
    }
    let paint_rect = paint.rcPaint;

    let app = app_data_ptr(parent);
    if app.is_null() {
        let palette = Theme::Dark.palette();
        fill_rect_with_color(hdc, &paint_rect, palette.status_background);
    } else {
        unsafe {
            let palette = (*app).theme.palette();
            fill_rect_with_color(hdc, &paint_rect, palette.status_background);
            let top_line = Rect {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.top + 1,
            };
            fill_rect_with_color(hdc, &top_line, palette.status_separator);
            draw_status_text(&*app, hdc, rect);
        }
    }

    unsafe {
        EndPaint(hwnd, &paint);
    }
}

fn draw_status_text(app: &AppData, hdc: Hdc, rect: Rect) {
    let status_font = unsafe { GetStockObject(DEFAULT_GUI_FONT) as Hfont };
    let old_font = if status_font.is_null() {
        null_mut()
    } else {
        unsafe { SelectObject(hdc, status_font as Hgdiobj) }
    };

    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, app.theme.palette().status_text);
    }

    if app.compare_tab.is_some() && !app.compare_edit.is_null() {
        draw_compare_status_text(app, hdc, rect);
    } else {
        let left = to_wide(&app.status_left_cache);
        let right = to_wide(&app.status_right_cache);
        let (mut left_rect, mut right_rect) = status_text_rects(rect);

        unsafe {
            DrawTextW(
                hdc,
                left.as_ptr(),
                -1,
                &mut left_rect,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            DrawTextW(
                hdc,
                right.as_ptr(),
                -1,
                &mut right_rect,
                DT_RIGHT | DT_SINGLELINE | DT_VCENTER,
            );
        }
    }

    if !old_font.is_null() {
        unsafe {
            SelectObject(hdc, old_font);
        }
    }
}

fn draw_compare_status_text(app: &AppData, hdc: Hdc, rect: Rect) {
    let gutter_width = if app.line_numbers_visible {
        GUTTER_WIDTH
    } else {
        0
    };
    let width = (rect.right - rect.left).max(0);
    let (left_width, right_left, _right_width) =
        compare_layout_columns(app, width, gutter_width);
    let palette = app.theme.palette();

    let separator_rect = Rect {
        left: left_width,
        top: rect.top,
        right: (left_width + COMPARE_SPLITTER_WIDTH).min(rect.right),
        bottom: rect.bottom,
    };
    fill_rect_with_color(hdc, &separator_rect, palette.status_separator);

    // Keep the clock/date pinned to the far-right when comparing files.
    // Right editor status uses the middle-right area and the clock keeps its normal spot.
    let clock_width = 260;
    let clock_left = (rect.right - clock_width).max(right_left + 80);
    let mut left_rect = Rect {
        left: rect.left + 4,
        top: rect.top,
        right: (left_width - 6).max(rect.left + 4),
        bottom: rect.bottom,
    };
    let mut right_rect = Rect {
        left: right_left + 4,
        top: rect.top,
        right: (clock_left - 8).max(right_left + 4),
        bottom: rect.bottom,
    };
    let mut clock_rect = Rect {
        left: clock_left,
        top: rect.top,
        right: rect.right - 6,
        bottom: rect.bottom,
    };

    let left = to_wide(&app.status_left_cache);
    let right = to_wide(&app.status_right_cache);
    let clock = to_wide(&status_clock_text());

    unsafe {
        DrawTextW(
            hdc,
            left.as_ptr(),
            -1,
            &mut left_rect,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
        DrawTextW(
            hdc,
            right.as_ptr(),
            -1,
            &mut right_rect,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
        DrawTextW(
            hdc,
            clock.as_ptr(),
            -1,
            &mut clock_rect,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER,
        );
    }
}
fn status_text_rects(rect: Rect) -> (Rect, Rect) {
    let mut left_rect = Rect {
        left: rect.left + 4,
        top: rect.top,
        right: rect.right - 260,
        bottom: rect.bottom,
    };
    let mut right_rect = Rect {
        left: rect.right - 260,
        top: rect.top,
        right: rect.right - 6,
        bottom: rect.bottom,
    };

    if left_rect.right < left_rect.left + 80 {
        left_rect.right = rect.right - 6;
        right_rect.left = right_rect.right;
    }

    (left_rect, right_rect)
}

fn status_left_text(app: &AppData) -> String {
    if app.compare_tab.is_some() && !app.compare_edit.is_null() {
        return editor_status_compact(app, app.edit);
    }

    let (line, _, column) = editor_position(app, app.edit);
    let total_lines = active_document_line_count(app);
    let selection = editor_selection_status(app);
    let characters = editor_character_count(app.edit);
    let line_ending = active_document(app).line_ending.label();
    let large_file_prefix = if edit_text_is_large(app.edit) {
        "Large file - features disabled | "
    } else {
        ""
    };

    format!(
        "{}Ln {} of {}, Col {} | {} | {} chars | UTF-8 | {} | {}%",
        large_file_prefix,
        format_number(line.min(total_lines).max(1)),
        format_number(total_lines),
        format_number(column),
        format_selection_status(&selection),
        format_number(characters),
        line_ending,
        app.zoom_percent,
    )
}

fn editor_status_compact(app: &AppData, edit: Hwnd) -> String {
    let (line, total_lines, column) = editor_position(app, edit);
    let document = document_for_editor(app, edit);
    format!(
        "Ln {} of {}, Col {} | {} chars | UTF-8 | {}",
        format_number(line.min(total_lines).max(1)),
        format_number(total_lines.max(1)),
        format_number(column),
        format_number(editor_character_count(edit)),
        document.line_ending.label(),
    )
}

fn status_right_text(app: &AppData) -> String {
    if app.compare_tab.is_some() && !app.compare_edit.is_null() {
        return format!("{}", editor_status_compact(app, app.compare_edit));
    }

    status_clock_text()
}

fn status_clock_text() -> String {
    let mut local_time: SystemTime = unsafe { zeroed() };
    unsafe {
        GetLocalTime(&mut local_time);
    }

    let weekdays = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let weekday = weekdays
        .get(local_time.wDayOfWeek as usize)
        .copied()
        .unwrap_or("Monday");
    let suffix = if local_time.wHour >= 12 { "PM" } else { "AM" };
    let hour = match local_time.wHour % 12 {
        0 => 12,
        value => value,
    };

    format!(
        "{} | {:02}:{:02} {} | {:02}/{:02}/{:04}",
        weekday,
        hour,
        local_time.wMinute,
        suffix,
        local_time.wMonth,
        local_time.wDay,
        local_time.wYear,
    )
}

fn format_selection_status(selection: &SelectionStatus) -> String {
    if selection.chars == 0 {
        return "Sel: 0 chars".to_string();
    }

    if let Some(bytes) = selection.bytes {
        format!(
            "Sel: {} chars, {} bytes, {} lines",
            format_number(selection.chars),
            format_number(bytes),
            format_number(selection.lines)
        )
    } else {
        format!(
            "Sel: {} chars, {} lines",
            format_number(selection.chars),
            format_number(selection.lines)
        )
    }
}

fn editor_selection_status(app: &AppData) -> SelectionStatus {
    if app.edit.is_null() {
        return SelectionStatus {
            chars: 0,
            bytes: Some(0),
            lines: 0,
        };
    }

    let (start, end) = edit_selection(app.edit);
    if start == end {
        return SelectionStatus {
            chars: 0,
            bytes: Some(0),
            lines: 0,
        };
    }

    if !edit_text_is_large(app.edit) {
        let text = get_edit_text(app.edit);
        let start_byte = byte_index_from_utf16_pos(&text, start);
        let end_byte = byte_index_from_utf16_pos(&text, end);
        let selected = from_edit_line_endings(&text[start_byte..end_byte]);
        return SelectionStatus {
            chars: selected.chars().count(),
            bytes: Some(selected.len()),
            lines: logical_line_count(&selected),
        };
    }

    let start_line =
        line_index_from_starts(&active_document(app).line_starts, start).max(0) as usize;
    let end_position = end.saturating_sub(1).max(start);
    let end_line =
        line_index_from_starts(&active_document(app).line_starts, end_position).max(0) as usize;

    SelectionStatus {
        chars: (end - start).max(0) as usize,
        bytes: None,
        lines: end_line.saturating_sub(start_line) + 1,
    }
}

fn editor_position(app: &AppData, edit: Hwnd) -> (usize, usize, usize) {
    if edit.is_null() {
        return (1, 1, 1);
    }

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

    let document = document_for_editor(app, edit);

    // RichEdit reports caret offsets using its own internal text stream where CRLF
    // paragraph breaks are effectively counted differently than the String buffer we
    // keep for save/session logic. Mapping EM_GETSEL through document.line_starts can
    // drift lower as blank lines accumulate. Ask the edit control for the actual line
    // and actual line-start offset so the status bar matches the visible gutter/caret.
    if app.use_rich_edit {
        let line_index =
            unsafe { SendMessageW(edit, EM_LINEFROMCHAR, selection_start as Wparam, 0) }.max(0)
                as i32;

        let line_start = unsafe { SendMessageW(edit, EM_LINEINDEX, line_index as Wparam, 0) };
        let column = if line_start < 0 {
            1
        } else {
            (selection_start as i32 - line_start as i32).max(0) as usize + 1
        };

        return (
            (line_index + 1) as usize,
            document.line_count.max(1),
            column,
        );
    }

    let line_index =
        line_index_from_starts(&document.line_starts, selection_start as i32).max(0);
    let line_start = line_start_for_index(&document.line_starts, line_index);
    let column = (selection_start as i32 - line_start).max(0) as usize + 1;

    (
        (line_index + 1) as usize,
        document.line_count.max(1),
        column,
    )
}

fn editor_character_count(edit: Hwnd) -> usize {
    if edit.is_null() {
        return 0;
    }

    let text_len = edit_text_len(edit);
    if text_len > LARGE_TEXT_FEATURE_LIMIT {
        return text_len;
    }

    from_edit_line_endings(&get_edit_text(edit))
        .chars()
        .filter(|character| *character != '\n' && *character != '\r')
        .count()
}

fn format_number(value: usize) -> String {
    let text = value.to_string();
    let mut output = String::with_capacity(text.len() + text.len() / 3);

    for (index, character) in text.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(character);
    }

    output.chars().rev().collect()
}

fn draw_line_numbers(app: &AppData, hdc: Hdc, rect: Rect, edit: Hwnd, gutter: Hwnd) {
    if edit.is_null() {
        return;
    }

    let old_font = if app.font.is_null() {
        null_mut()
    } else {
        unsafe { SelectObject(hdc, app.font as Hgdiobj) }
    };

    let show_folds = false;
    let rows = visible_gutter_rows(app, hdc, rect, edit, gutter);

    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, app.theme.palette().gutter_text);
    }

    for row in rows {
        draw_line_number_label(
            app,
            hdc,
            rect,
            row.line_index,
            row.top,
            row.height,
            show_folds,
        );
    }

    if !old_font.is_null() {
        unsafe {
            SelectObject(hdc, old_font);
        }
    }
}

fn visible_gutter_rows(
    app: &AppData,
    hdc: Hdc,
    rect: Rect,
    edit: Hwnd,
    gutter: Hwnd,
) -> Vec<GutterRow> {
    if edit.is_null() {
        return Vec::new();
    }

    let max_line = editor_document_line_count(app, edit)
        .max(1)
        .min(i32::MAX as usize) as i32
        - 1;
    let edit_origin = edit_client_origin_in_gutter(edit, gutter);
    let first_visible = first_visible_editor_line(app, edit).clamp(0, max_line);
    let fallback_height = edit_line_height(app, hdc, first_visible).max(1);
    let mut rows = Vec::new();

    let mut line = first_visible;
    let mut safety = 0usize;
    while line <= max_line && safety < 10_000 {
        safety += 1;

        let visible_line = visible_line_at_or_after(app, line, max_line);
        if visible_line != line {
            line = visible_line;
            continue;
        }

        let Some(y) = edit_line_y(app, edit, line) else {
            line += 1;
            continue;
        };

        let top = edit_origin.y + y;
        let next_y = edit_line_y(app, edit, line + 1);
        let height = next_y
            .map(|next| next - y)
            .filter(|height| *height > 0 && *height < fallback_height * 4)
            .unwrap_or(fallback_height);

        if top >= rect.bottom {
            break;
        }
        if top + height > rect.top {
            rows.push(GutterRow {
                line_index: line,
                top,
                height,
            });
        }

        line += 1;
    }

    if rows.is_empty() {
        let top = edit_origin.y + edit_line_y(app, edit, first_visible).unwrap_or(0);
        rows.push(GutterRow {
            line_index: first_visible,
            top,
            height: fallback_height,
        });
    }

    rows
}

fn draw_line_number_label(
    app: &AppData,
    hdc: Hdc,
    rect: Rect,
    line_index: i32,
    top: i32,
    line_height: i32,
    show_folds: bool,
) {
    let palette = app.theme.palette();
    let row_rect = Rect {
        left: rect.left,
        top,
        right: (rect.right - 1).max(rect.left),
        bottom: top + line_height,
    };
    let hovered = app.gutter_hover_line == Some(line_index);
    fill_rect_with_color(
        hdc,
        &row_rect,
        if hovered {
            palette.tab_inactive_background
        } else {
            palette.gutter_background
        },
    );

    if show_folds {
        draw_fold_guides(app, hdc, line_index, top, line_height);
        if let Some(range) = fold_range_starting_on_line(app, line_index) {
            let depth = fold_nesting_depth_for_range(app, range);
            draw_fold_arrow(app, hdc, top, line_height, range.collapsed, depth, hovered);
        }
    }

    let label = to_wide(&(line_index + 1).to_string());
    // Keep the number column wide enough for large files. The fold hit area
    // lives on the left, while line numbers are right-aligned in the rest
    // of the gutter. Older builds used a very narrow text rect here, which
    // clipped the leading digits and made lines look like ".33", ".34", etc.
    let mut text_rect = Rect {
        left: rect.left + 4,
        top,
        right: rect.right - 6,
        bottom: top + line_height,
    };

    if text_rect.right - text_rect.left < 40 {
        text_rect.left = rect.left + 4;
    }

    unsafe {
        DrawTextW(
            hdc,
            label.as_ptr(),
            -1,
            &mut text_rect,
            DT_RIGHT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
    }
}

#[allow(dead_code)]
fn fold_range_starting_on_line(app: &AppData, line_index: i32) -> Option<&FoldRange> {
    let mut left = 0usize;
    let mut right = app.fold_ranges.len();
    while left < right {
        let mid = (left + right) / 2;
        if app.fold_ranges[mid].start_line < line_index {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let mut best = None;
    for range in &app.fold_ranges[left..] {
        if range.start_line != line_index {
            break;
        }

        if best
            .map(|candidate: &FoldRange| {
                range.end_utf16 - range.start_utf16
                    < candidate.end_utf16 - candidate.start_utf16
            })
            .unwrap_or(true)
        {
            best = Some(range);
        }
    }

    best
}

#[allow(dead_code)]
fn fold_nesting_depth_for_range(app: &AppData, target: &FoldRange) -> usize {
    app.fold_ranges
        .iter()
        .filter(|range| {
            range.start_line < target.start_line
                && target.end_line <= range.end_line
                && range.start_utf16 < target.start_utf16
                && target.end_utf16 <= range.end_utf16
        })
        .count()
}

fn collapsed_fold_containing_hidden_line(app: &AppData, line_index: i32) -> Option<&FoldRange> {
    app.fold_ranges
        .iter()
        .filter(|range| {
            range.collapsed && range.start_line < line_index && line_index < range.end_line
        })
        .max_by_key(|range| range.end_line - range.start_line)
}

fn visible_line_at_or_after(app: &AppData, line_index: i32, max_line: i32) -> i32 {
    let mut current = line_index.clamp(0, max_line);
    while let Some(range) = collapsed_fold_containing_hidden_line(app, current) {
        let next = range.end_line.min(max_line);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

#[allow(dead_code)]
fn draw_fold_guides(app: &AppData, hdc: Hdc, line_index: i32, line_top: i32, line_height: i32) {
    let palette = app.theme.palette();
    let mut depth = 0i32;

    for range in &app.fold_ranges {
        if range.start_line < line_index && line_index < range.end_line {
            let guide_x = FOLD_BOX_LEFT
                + (depth * FOLD_NEST_INDENT).min(FOLD_MAX_NEST_INDENT)
                + FOLD_BOX_SIZE / 2;
            let guide = Rect {
                left: guide_x,
                top: line_top,
                right: guide_x + 1,
                bottom: line_top + line_height,
            };
            fill_rect_with_color(hdc, &guide, palette.gutter_separator);
            depth += 1;
            if depth * FOLD_NEST_INDENT >= FOLD_MAX_NEST_INDENT {
                break;
            }
        }
    }
}

#[allow(dead_code)]
fn draw_fold_arrow(
    app: &AppData,
    hdc: Hdc,
    line_top: i32,
    line_height: i32,
    collapsed: bool,
    depth: usize,
    hovered: bool,
) {
    let palette = app.theme.palette();
    let indent = ((depth as i32) * FOLD_NEST_INDENT).min(FOLD_MAX_NEST_INDENT);
    let center_y = line_top + line_height / 2;
    let left = FOLD_BOX_LEFT + indent;
    let mut arrow_rect = Rect {
        left,
        top: center_y - FOLD_BOX_SIZE / 2,
        right: left + FOLD_BOX_SIZE + 4,
        bottom: center_y + FOLD_BOX_SIZE / 2 + 1,
    };

    // VSCode-style folding marker: a chevron instead of a square +/- box.
    // Collapsed blocks point right; expanded blocks point down.
    let arrow = if collapsed { "▸" } else { "▾" };
    let arrow_text = to_wide(arrow);
    unsafe {
        SetTextColor(
            hdc,
            if hovered {
                palette.tab_active_text
            } else {
                palette.gutter_text
            },
        );
        DrawTextW(
            hdc,
            arrow_text.as_ptr(),
            -1,
            &mut arrow_rect,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
        SetTextColor(hdc, palette.gutter_text);
    }
}

fn edit_line_height(app: &AppData, hdc: Hdc, first_visible: i32) -> i32 {
    if let (Some(first_y), Some(next_y)) = (
        edit_line_y(app, app.edit, first_visible),
        edit_line_y(app, app.edit, first_visible + 1),
    ) {
        let height = next_y - first_y;
        if height > 0 {
            return height;
        }
    }

    text_metric_line_height(hdc)
}

fn text_metric_line_height(hdc: Hdc) -> i32 {
    let mut metrics: TextMetricW = unsafe { zeroed() };
    if unsafe { GetTextMetricsW(hdc, &mut metrics) } == 0 {
        16
    } else {
        (metrics.tmHeight + metrics.tmExternalLeading).max(1)
    }
}

fn edit_line_y(app: &AppData, edit: Hwnd, line_index: i32) -> Option<i32> {
    if line_index < 0 || edit.is_null() {
        return None;
    }

    // Use the editor's own line index instead of our document cache. This
    // is the critical sync point for project-open files: RichEdit decides
    // where CRLF/blank lines render, and the gutter follows that exactly.
    let char_index = unsafe { SendMessageW(edit, EM_LINEINDEX, line_index as Wparam, 0) };
    if char_index < 0 {
        return None;
    }

    if app.use_rich_edit {
        let mut point = Point { x: 0, y: 0 };
        unsafe {
            SendMessageW(
                edit,
                EM_POSFROMCHAR,
                (&mut point as *mut Point) as Wparam,
                char_index as Lparam,
            );
        }
        Some(point.y)
    } else {
        let position = unsafe { SendMessageW(edit, EM_POSFROMCHAR, char_index as Wparam, 0) };
        if position == -1 {
            return None;
        }

        Some(signed_high_word(position as usize))
    }
}

fn edit_client_origin_in_gutter(edit: Hwnd, gutter: Hwnd) -> Point {
    let mut point = Point { x: 0, y: 0 };

    unsafe {
        ClientToScreen(edit, &mut point);
        ScreenToClient(gutter, &mut point);
    }

    point
}

fn reset_editor_visual_state(app: &mut AppData) {
    app.fold_ranges.clear();
    app.fold_formats_active = false;
}

fn active_document(app: &AppData) -> &Document {
    &app.documents[app.active_tab]
}

fn active_document_mut(app: &mut AppData) -> &mut Document {
    &mut app.documents[app.active_tab]
}

fn document_for_editor(app: &AppData, edit: Hwnd) -> &Document {
    if edit == app.compare_edit {
        app.compare_tab
            .and_then(|index| app.documents.get(index))
            .unwrap_or_else(|| active_document(app))
    } else {
        active_document(app)
    }
}

fn editor_document_line_count(app: &AppData, edit: Hwnd) -> usize {
    document_for_editor(app, edit).line_count.max(1)
}

fn document_line_start_utf16(document: &Document, line_index: i32) -> Option<i32> {
    if line_index < 0 {
        return None;
    }

    document.line_starts.get(line_index as usize).copied()
}

fn document_line_end_utf16(document: &Document, line_index: i32) -> Option<i32> {
    if line_index < 0 {
        return None;
    }

    let index = line_index as usize;
    if index >= document.line_starts.len() {
        return None;
    }

    Some(
        document
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or_else(|| utf16_len(&document.text)),
    )
}

fn document_line_text(document: &Document, line_index: i32) -> Option<String> {
    let start = document_line_start_utf16(document, line_index)?;
    let end = document_line_end_utf16(document, line_index)?;
    let start_byte = byte_index_from_utf16_pos(&document.text, start);
    let end_byte = byte_index_from_utf16_pos(&document.text, end);

    Some(
        document.text[start_byte..end_byte]
            .trim_end_matches(['\r', '\n'])
            .to_string(),
    )
}

fn active_document_line_count(app: &AppData) -> usize {
    app.documents
        .get(app.active_tab)
        .map(|document| document.line_count.max(1))
        .unwrap_or(1)
}

fn active_file_name(app: &AppData) -> String {
    active_document(app).display_name()
}

fn active_file_location(app: &AppData) -> String {
    active_document(app).location()
}

fn sync_active_document_text(app: &mut AppData) {
    if app.edit.is_null() || app.documents.is_empty() {
        return;
    }

    let text = get_edit_text(app.edit);
    active_document_mut(app).set_text(text);
}

fn sync_active_document_text_for_session(app: &mut AppData) {
    if app.documents.is_empty() {
        return;
    }

    if active_document(app).path.is_some()
        && edit_text_len(app.edit) > SESSION_INLINE_TEXT_LIMIT
    {
        return;
    }

    sync_active_document_text(app);
}

fn switch_to_tab(app: &mut AppData, index: usize) {
    if index >= app.documents.len() || index == app.active_tab {
        return;
    }

    sync_active_document_text(app);
    close_compare_tabs(app);
    app.active_tab = index;

    let text = active_document(app).text.clone();
    reset_editor_visual_state(app);
    set_active_edit_text(app, &text);
    unsafe {
        SetFocus(app.edit);
    }

    update_window_title(app);
    invalidate_document_chrome(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
    save_session_state(app);
}

fn cycle_tab(app: &mut AppData, direction: i32) {
    let tab_count = app.documents.len();
    if tab_count < 2 {
        return;
    }

    let next = if direction < 0 {
        (app.active_tab + tab_count - 1) % tab_count
    } else {
        (app.active_tab + 1) % tab_count
    };

    switch_to_tab(app, next);
}

fn close_active_tab(app: &mut AppData) {
    close_tab(app, app.active_tab);
}

fn close_tab(app: &mut AppData, index: usize) {
    if index >= app.documents.len() {
        return;
    }

    sync_active_document_text(app);
    close_compare_tabs(app);

    if app.documents.len() == 1 {
        if active_document(app).is_initial_untitled() {
            return;
        }

        app.documents[0] = Document::untitled(1);
        app.next_untitled_index = 2;
        app.active_tab = 0;
        reset_editor_visual_state(app);
        set_active_edit_text(app, "");
        unsafe {
            SetFocus(app.edit);
        }
        update_window_title(app);
        invalidate_document_chrome(app);
        invalidate_gutter(app);
        refresh_status_if_changed(app);
        save_session_state(app);
        return;
    }

    let closed_active = index == app.active_tab;
    app.documents.remove(index);

    if app.active_tab > index {
        app.active_tab -= 1;
    } else if app.active_tab >= app.documents.len() {
        app.active_tab = app.documents.len() - 1;
    }

    if closed_active {
        let text = active_document(app).text.clone();
        reset_editor_visual_state(app);
        set_active_edit_text(app, &text);
        unsafe {
            SetFocus(app.edit);
        }
    }

    update_window_title(app);
    invalidate_document_chrome(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
    save_session_state(app);
}

fn show_compare_tab_picker(app: &mut AppData) {
    sync_active_document_text(app);

    if app.documents.len() < 2 {
        message_box(
            app.hwnd,
            "Open another tab before comparing tabs.",
            "Compare Tabs",
            MB_OK | MB_ICONINFORMATION,
        );
        return;
    }

    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }

    for (index, document) in app.documents.iter().enumerate() {
        if index == app.active_tab {
            continue;
        }

        let label = menu_label_escape(&document.display_name());
        let _ = append_menu(
            menu,
            MF_STRING,
            (ID_COMPARE_TAB_BASE + index as u16) as usize,
            &label,
        );
    }

    let mut rect = empty_rect();
    unsafe {
        GetWindowRect(app.hwnd, &mut rect);
    }
    let x = rect.left + (rect.right - rect.left) / 2;
    let y = rect.top + (rect.bottom - rect.top) / 2;
    let command = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD,
            x,
            y,
            0,
            app.hwnd,
            null(),
        )
    };

    unsafe {
        DestroyMenu(menu);
    }

    if let Some(index) = compare_tab_command_index(command as u16, app.documents.len()) {
        open_compare_tabs(app, index);
    }
}

fn compare_tab_command_index(command_id: u16, tab_count: usize) -> Option<usize> {
    if command_id < ID_COMPARE_TAB_BASE {
        return None;
    }

    let index = (command_id - ID_COMPARE_TAB_BASE) as usize;
    if index < tab_count { Some(index) } else { None }
}

fn open_compare_tabs(app: &mut AppData, tab_index: usize) {
    if tab_index >= app.documents.len() || tab_index == app.active_tab {
        return;
    }

    sync_active_document_text(app);
    app.compare_tab = Some(tab_index);
    app.compare_split_ratio = COMPARE_SPLIT_CENTER;
    refresh_compare_text(app);
    unsafe {
        SetFocus(app.edit);
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(app.hwnd, &mut rect);
    }
    layout_editor(app, rect.right - rect.left, rect.bottom - rect.top);
}

fn close_compare_tabs(app: &mut AppData) {
    if app.compare_tab.is_none() {
        return;
    }

    app.compare_tab = None;
    app.compare_dragging = false;
    unsafe {
        SetWindowTextW(app.compare_edit, to_wide("").as_ptr());
        ShowWindow(app.compare_edit, SW_HIDE);
        ShowWindow(app.compare_gutter, SW_HIDE);
        ShowWindow(app.compare_splitter, SW_HIDE);
        SetFocus(app.edit);
    }

    let mut rect = empty_rect();
    unsafe {
        GetClientRect(app.hwnd, &mut rect);
    }
    layout_editor(app, rect.right - rect.left, rect.bottom - rect.top);
}

fn close_compare_or_exit(app: &mut AppData) {
    if app.compare_tab.is_some() {
        close_compare_tabs(app);
    } else {
        unsafe {
            DestroyWindow(app.hwnd);
        }
    }
}

fn toggle_compare_page_sync(app: &mut AppData) {
    app.compare_page_sync = !app.compare_page_sync;
    update_compare_page_sync_menu_item(app);
    save_session_state(app);
}

fn refresh_compare_text(app: &mut AppData) {
    let Some(tab_index) = app.compare_tab else {
        return;
    };
    let Some(document) = app.documents.get(tab_index) else {
        return;
    };

    let text = document.text.clone();
    set_compare_edit_text(app, &text);
}

fn replace_active_document(app: &mut AppData, document: Document) {
    app.documents[app.active_tab] = document;
    let text = active_document(app).text.clone();
    reset_editor_visual_state(app);
    set_active_edit_text(app, &text);
    update_window_title(app);
    invalidate_document_chrome(app);
    invalidate_gutter(app);
    refresh_status_if_changed(app);
}

fn remove_tab_without_loading(app: &mut AppData, index: usize) {
    if app.documents.len() <= 1 || index >= app.documents.len() {
        return;
    }

    app.documents.remove(index);
    if app.active_tab > index {
        app.active_tab -= 1;
    } else if app.active_tab >= app.documents.len() {
        app.active_tab = app.documents.len() - 1;
    }
}

fn tab_index_at_x(app: &AppData, x: i32) -> Option<usize> {
    let hdc = unsafe { GetDC(app.tab_bar) };
    if hdc.is_null() {
        return None;
    }

    let old_font = select_gui_font(hdc);
    let mut current_x = 0;
    let mut hit = None;

    for (index, document) in app.documents.iter().enumerate() {
        let width = tab_width(hdc, &document.tab_label());
        if x >= current_x && x < current_x + width {
            hit = Some(index);
            break;
        }
        current_x += width;
    }

    restore_font(hdc, old_font);
    unsafe {
        ReleaseDC(app.tab_bar, hdc);
    }

    hit
}

fn tab_close_index_at_point(app: &AppData, x: i32, y: i32) -> Option<usize> {
    let hdc = unsafe { GetDC(app.tab_bar) };
    if hdc.is_null() {
        return None;
    }

    let old_font = select_gui_font(hdc);
    let mut current_x = 0;
    let mut hit = None;

    for (index, document) in app.documents.iter().enumerate() {
        let width = tab_width(hdc, &document.tab_label());
        let tab_rect = Rect {
            left: current_x,
            top: 0,
            right: current_x + width,
            bottom: TAB_BAR_HEIGHT,
        };
        if point_in_rect(x, y, tab_close_rect(tab_rect)) {
            hit = Some(index);
            break;
        }
        current_x += width;
    }

    restore_font(hdc, old_font);
    unsafe {
        ReleaseDC(app.tab_bar, hdc);
    }

    hit
}

fn update_tab_hover(app: &mut AppData, x: i32, y: i32) {
    let old_tab_hover = app.tab_hover_index;
    let old_close_hover = app.tab_hover_close_index;

    app.tab_hover_index = tab_index_at_x(app, x);
    app.tab_hover_close_index = tab_close_index_at_point(app, x, y);

    if old_tab_hover != app.tab_hover_index || old_close_hover != app.tab_hover_close_index {
        invalidate_window(app.tab_bar);
    }
}

fn point_in_rect(x: i32, y: i32, rect: Rect) -> bool {
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
}

fn show_editor_context_menu(app: &mut AppData, point: Point) {
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }

    let append_result = append_menu(menu, MF_STRING, ID_EDIT_CUT as usize, "Cut")
        .and_then(|_| append_menu(menu, MF_STRING, ID_EDIT_COPY as usize, "Copy"))
        .and_then(|_| append_menu(menu, MF_STRING, ID_EDIT_PASTE as usize, "Paste"))
        .and_then(|_| append_menu(menu, MF_SEPARATOR, 0, ""))
        .and_then(|_| append_menu(menu, MF_STRING, ID_EDIT_SELECT_ALL as usize, "Select All"));

    if append_result.is_err() {
        unsafe {
            DestroyMenu(menu);
        }
        return;
    }

    let command = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD,
            point.x,
            point.y,
            0,
            app.hwnd,
            null(),
        )
    };

    unsafe {
        DestroyMenu(menu);
    }

    if command != 0 {
        handle_command(app, command as u16);
    }
}

fn show_tab_context_menu(app: &mut AppData, point: Point) {
    let Some(index) = tab_index_at_x(app, point.x) else {
        return;
    };

    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }

    let append_result = append_menu(menu, MF_STRING, ID_FILE_SAVE as usize, "Save")
        .and_then(|_| append_menu(menu, MF_STRING, ID_FILE_SAVE_AS as usize, "Save As..."))
        .and_then(|_| append_menu(menu, MF_SEPARATOR, 0, ""))
        .and_then(|_| append_menu(menu, MF_STRING, ID_TAB_COPY_PATH as usize, "Copy Path"))
        .and_then(|_| append_menu(menu, MF_STRING, ID_TAB_COPY_NAME as usize, "Copy Name"))
        .and_then(|_| {
            append_menu(
                menu,
                MF_STRING,
                ID_TAB_REVEAL as usize,
                "Reveal in File Explorer",
            )
        })
        .and_then(|_| append_menu(menu, MF_SEPARATOR, 0, ""))
        .and_then(|_| append_menu(menu, MF_STRING, ID_FILE_CLOSE_TAB as usize, "Close Tab"));
    if append_result.is_err() {
        unsafe {
            DestroyMenu(menu);
        }
        return;
    }

    let mut screen_point = point;
    unsafe {
        ClientToScreen(app.tab_bar, &mut screen_point);
    }

    let command = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD,
            screen_point.x,
            screen_point.y,
            0,
            app.hwnd,
            null(),
        )
    };

    unsafe {
        DestroyMenu(menu);
    }

    match command as u16 {
        ID_FILE_SAVE => {
            if index != app.active_tab {
                switch_to_tab(app, index);
            }
            save_file(app);
        }
        ID_FILE_SAVE_AS => {
            if index != app.active_tab {
                switch_to_tab(app, index);
            }
            save_file_as(app);
        }
        ID_TAB_COPY_PATH => copy_tab_path(app, index),
        ID_TAB_COPY_NAME => copy_tab_name(app, index),
        ID_TAB_REVEAL => reveal_tab_in_explorer(app, index),
        ID_FILE_CLOSE_TAB => close_tab(app, index),
        _ => {}
    }
}

fn copy_tab_path(app: &mut AppData, index: usize) {
    let Some(document) = app.documents.get(index) else {
        return;
    };
    let Some(path) = document.path.as_deref() else {
        message_box(
            app.hwnd,
            "This tab has not been saved yet, so it does not have a file path.",
            "Copy Path",
            MB_OK | MB_ICONINFORMATION,
        );
        return;
    };

    if let Err(error) = copy_text_to_clipboard(app.hwnd, &display_path(path)) {
        message_box(
            app.hwnd,
            &format!("Could not copy to the clipboard:\n\n{error}"),
            "Copy Failed",
            MB_OK | MB_ICONERROR,
        );
    } else {
        show_copied_indicator(app);
    }
}

fn copy_tab_name(app: &mut AppData, index: usize) {
    let Some(document) = app.documents.get(index) else {
        return;
    };

    if let Err(error) = copy_text_to_clipboard(app.hwnd, &document.display_name()) {
        message_box(
            app.hwnd,
            &format!("Could not copy to the clipboard:\n\n{error}"),
            "Copy Failed",
            MB_OK | MB_ICONERROR,
        );
    } else {
        show_copied_indicator(app);
    }
}

fn reveal_tab_in_explorer(app: &AppData, index: usize) {
    let Some(path) = app
        .documents
        .get(index)
        .and_then(|document| document.path.as_ref())
    else {
        message_box(
            app.hwnd,
            "Save this tab before revealing it in File Explorer.",
            "Reveal in File Explorer",
            MB_OK | MB_ICONINFORMATION,
        );
        return;
    };

    reveal_path_in_explorer(app.hwnd, path);
}

fn reveal_path_in_explorer(hwnd: Hwnd, path: &Path) {
    let operation = to_wide("open");
    let explorer = to_wide("explorer.exe");
    let parameters = to_wide(&format!("/select,\"{}\"", display_path(path)));
    let result = unsafe {
        ShellExecuteW(
            hwnd,
            operation.as_ptr(),
            explorer.as_ptr(),
            parameters.as_ptr(),
            null(),
            SW_SHOW,
        )
    };

    if result <= 32 {
        message_box(
            hwnd,
            "Windows could not reveal this file in File Explorer.",
            "Reveal Failed",
            MB_OK | MB_ICONERROR,
        );
    }
}

fn handle_path_bar_click(app: &mut AppData, x: i32) {
    let name = active_file_name(app);
    let location = active_file_location(app);
    let separator = "  >  ";
    let hdc = unsafe { GetDC(app.path_bar) };

    let (name_end, location_start) = if hdc.is_null() {
        (
            10 + (name.len() as i32 * 7),
            10 + ((name.len() + separator.len()) as i32 * 7),
        )
    } else {
        let old_font = select_gui_font(hdc);
        let name_end = 10 + measure_text_width(hdc, &name);
        let location_start = name_end + measure_text_width(hdc, separator);
        restore_font(hdc, old_font);
        unsafe {
            ReleaseDC(app.path_bar, hdc);
        }
        (name_end, location_start)
    };

    let copy_result = if x >= 10 && x <= name_end {
        copy_text_to_clipboard(app.hwnd, &name)
    } else if x >= location_start {
        copy_text_to_clipboard(app.hwnd, &location)
    } else {
        return;
    };

    if let Err(error) = copy_result {
        message_box(
            app.hwnd,
            &format!("Could not copy to the clipboard:\n\n{error}"),
            "Copy Failed",
            MB_OK | MB_ICONERROR,
        );
    } else {
        show_copied_indicator(app);
    }
}

fn copy_gutter_line_at_y(app: &mut AppData, gutter: Hwnd, y: i32) {
    let edit = if gutter == app.compare_gutter {
        app.compare_edit
    } else {
        sync_active_document_text(app);
        app.edit
    };

    let Some(line_index) = gutter_line_at_y_for(app, y, edit, gutter) else {
        return;
    };
    let Some(line_text) = document_line_text(document_for_editor(app, edit), line_index) else {
        return;
    };

    if let Err(error) = copy_text_to_clipboard(app.hwnd, &line_text) {
        message_box(
            app.hwnd,
            &format!("Could not copy to the clipboard:\n\n{error}"),
            "Copy Failed",
            MB_OK | MB_ICONERROR,
        );
    } else {
        show_copied_indicator(app);
    }
}

fn show_copied_indicator(app: &mut AppData) {
    if app.copy_indicator.is_null() {
        return;
    }

    app.copied_indicator_visible = true;

    let mut point = Point { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut point) == 0 || ScreenToClient(app.hwnd, &mut point) == 0 {
            point = Point { x: 16, y: 16 };
        }
    }

    let mut client_rect = empty_rect();
    unsafe {
        GetClientRect(app.hwnd, &mut client_rect);
    }
    let left = (point.x + 14)
        .min(client_rect.right - COPY_INDICATOR_WIDTH - 4)
        .max(client_rect.left + 4);
    let top = (point.y + 18)
        .min(client_rect.bottom - COPY_INDICATOR_HEIGHT - 4)
        .max(client_rect.top + 4);

    unsafe {
        SetWindowPos(
            app.copy_indicator,
            null_mut(),
            left,
            top,
            COPY_INDICATOR_WIDTH,
            COPY_INDICATOR_HEIGHT,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        SetTimer(
            app.hwnd,
            COPY_INDICATOR_TIMER_ID,
            COPY_INDICATOR_TIMER_MS,
            null_mut(),
        );
        InvalidateRect(app.copy_indicator, null(), 0);
        UpdateWindow(app.copy_indicator);
    }
}

fn hide_copied_indicator(app: &mut AppData) {
    if !app.copied_indicator_visible {
        return;
    }

    app.copied_indicator_visible = false;
    unsafe {
        ShowWindow(app.copy_indicator, SW_HIDE);
    }
}

fn draw_copied_indicator(app: &AppData, hdc: Hdc, rect: Rect) {
    let palette = app.theme.palette();
    fill_rect_with_color(hdc, &rect, palette.tab_border);

    let inner = Rect {
        left: rect.left + 1,
        top: rect.top + 1,
        right: rect.right - 1,
        bottom: rect.bottom - 1,
    };
    fill_rect_with_color(hdc, &inner, palette.tab_active_background);

    let mut text_rect = inner;
    unsafe {
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, palette.tab_active_text);
    }
    draw_single_line(
        hdc,
        "Copied",
        &mut text_rect,
        DT_SINGLELINE | DT_VCENTER | DT_CENTER,
    );
}

fn tab_width(hdc: Hdc, label: &str) -> i32 {
    (measure_text_width(hdc, label) + 56).max(112)
}

fn select_gui_font(hdc: Hdc) -> Hgdiobj {
    let font = unsafe { GetStockObject(DEFAULT_GUI_FONT) as Hfont };
    if font.is_null() {
        null_mut()
    } else {
        unsafe { SelectObject(hdc, font as Hgdiobj) }
    }
}

fn restore_font(hdc: Hdc, old_font: Hgdiobj) {
    if !old_font.is_null() {
        unsafe {
            SelectObject(hdc, old_font);
        }
    }
}

fn measure_text_width(hdc: Hdc, text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }

    let wide = to_wide(text);
    let mut size = Size { cx: 0, cy: 0 };
    let ok = unsafe {
        GetTextExtentPoint32W(hdc, wide.as_ptr(), (wide.len() - 1) as Int, &mut size)
    };

    if ok == 0 {
        text.len() as i32 * 7
    } else {
        size.cx
    }
}

fn draw_single_line(hdc: Hdc, text: &str, rect: &mut Rect, flags: Uint) {
    let wide = to_wide(text);
    unsafe {
        DrawTextW(
            hdc,
            wide.as_ptr(),
            -1,
            rect,
            DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | flags,
        );
    }
}

fn copy_text_to_clipboard(hwnd: Hwnd, text: &str) -> io::Result<()> {
    let safe_text = text.replace('\0', "\u{FFFD}");
    let wide = to_wide(&safe_text);
    let byte_count = wide.len().checked_mul(size_of::<u16>()).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Clipboard text is too large")
    })?;
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, byte_count) };
    if memory.is_null() {
        return Err(io::Error::last_os_error());
    }

    let target = unsafe { GlobalLock(memory) as *mut u16 };
    if target.is_null() {
        unsafe {
            GlobalFree(memory);
        }
        return Err(io::Error::last_os_error());
    }

    unsafe {
        copy_nonoverlapping(wide.as_ptr(), target, wide.len());
        GlobalUnlock(memory);
    }

    if unsafe { OpenClipboard(hwnd) } == 0 {
        unsafe {
            GlobalFree(memory);
        }
        return Err(io::Error::last_os_error());
    }

    unsafe {
        EmptyClipboard();
    }

    if unsafe { SetClipboardData(CF_UNICODETEXT, memory) }.is_null() {
        let error = io::Error::last_os_error();
        unsafe {
            CloseClipboard();
            GlobalFree(memory);
        }
        return Err(error);
    }

    unsafe {
        CloseClipboard();
    }

    Ok(())
}
