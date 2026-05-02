#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> Hinstance;
    fn GetLocalTime(lpSystemTime: *mut SystemTime);
    fn GlobalAlloc(uFlags: Uint, dwBytes: usize) -> Hglobal;
    fn GlobalFree(hMem: Hglobal) -> Hglobal;
    fn GlobalLock(hMem: Hglobal) -> *mut c_void;
    fn GlobalSize(hMem: Hglobal) -> usize;
    fn GlobalUnlock(hMem: Hglobal) -> Bool;
    fn GetProcAddress(hModule: Hinstance, lpProcName: *const u8) -> *mut c_void;
    fn LoadLibraryW(lpLibFileName: *const u16) -> Hinstance;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    fn ChooseFontW(param: *mut ChooseFontW) -> Bool;
    fn FindTextW(param: *mut FindReplaceW) -> Hwnd;
    fn GetOpenFileNameW(param: *mut OpenFileNameW) -> Bool;
    fn GetSaveFileNameW(param: *mut OpenFileNameW) -> Bool;
    fn ReplaceTextW(param: *mut FindReplaceW) -> Hwnd;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn DragAcceptFiles(hWnd: Hwnd, fAccept: Bool);
    fn DragFinish(hDrop: Hdrop);
    fn DragQueryFileW(hDrop: Hdrop, iFile: Uint, lpszFile: *mut u16, cch: Uint) -> Uint;
    fn IsUserAnAdmin() -> Bool;
    fn ShellExecuteW(
        hwnd: Hwnd,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: Int,
    ) -> isize;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateFontW(
        cHeight: Int,
        cWidth: Int,
        cEscapement: Int,
        cOrientation: Int,
        cWeight: Int,
        bItalic: Dword,
        bUnderline: Dword,
        bStrikeOut: Dword,
        iCharSet: Dword,
        iOutPrecision: Dword,
        iClipPrecision: Dword,
        iQuality: Dword,
        iPitchAndFamily: Dword,
        pszFaceName: *const u16,
    ) -> Hfont;
    fn CreateSolidBrush(color: Dword) -> Brush;
    fn DeleteObject(ho: Hgdiobj) -> Bool;
    fn GetDeviceCaps(hdc: Hdc, index: Int) -> Int;
    fn GetStockObject(i: Int) -> Handle;
    fn GetTextExtentPoint32W(hdc: Hdc, lpString: *const u16, c: Int, psizl: *mut Size) -> Bool;
    fn GetTextMetricsW(hdc: Hdc, lptm: *mut TextMetricW) -> Bool;
    fn SelectObject(hdc: Hdc, h: Hgdiobj) -> Hgdiobj;
    fn SetBkColor(hdc: Hdc, color: Dword) -> Dword;
    fn SetBkMode(hdc: Hdc, mode: Int) -> Int;
    fn SetTextColor(hdc: Hdc, color: Dword) -> Dword;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn AppendMenuW(
        hMenu: Hmenu,
        uFlags: Uint,
        uIDNewItem: usize,
        lpNewItem: *const u16,
    ) -> Bool;
    fn BeginPaint(hWnd: Hwnd, lpPaint: *mut PaintStruct) -> Hdc;
    fn CallWindowProcW(
        lpPrevWndFunc: isize,
        hWnd: Hwnd,
        Msg: Uint,
        wParam: Wparam,
        lParam: Lparam,
    ) -> Lresult;
    fn CheckMenuItem(hMenu: Hmenu, uIDCheckItem: Uint, uCheck: Uint) -> Dword;
    fn ClientToScreen(hWnd: Hwnd, lpPoint: *mut Point) -> Bool;
    fn CloseClipboard() -> Bool;
    fn CreateAcceleratorTableW(lpaccl: *mut Accel, cEntries: Int) -> Haccel;
    fn CreateMenu() -> Hmenu;
    fn CreatePopupMenu() -> Hmenu;
    fn CreateWindowExW(
        dwExStyle: Dword,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: Dword,
        x: Int,
        y: Int,
        nWidth: Int,
        nHeight: Int,
        hWndParent: Hwnd,
        hMenu: Hmenu,
        hInstance: Hinstance,
        lpParam: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hWnd: Hwnd, Msg: Uint, wParam: Wparam, lParam: Lparam) -> Lresult;
    fn DeleteMenu(hMenu: Hmenu, uPosition: Uint, uFlags: Uint) -> Bool;
    fn DestroyAcceleratorTable(hAccel: Haccel) -> Bool;
    fn DestroyMenu(hMenu: Hmenu) -> Bool;
    fn DestroyWindow(hWnd: Hwnd) -> Bool;
    fn DispatchMessageW(lpMsg: *const Msg) -> Lresult;
    fn DrawTextW(
        hdc: Hdc,
        lpchText: *const u16,
        cchText: Int,
        lprc: *mut Rect,
        format: Uint,
    ) -> Int;
    fn DrawMenuBar(hWnd: Hwnd) -> Bool;
    fn EmptyClipboard() -> Bool;
    fn EndMenu() -> Bool;
    fn EnableWindow(hWnd: Hwnd, bEnable: Bool) -> Bool;
    fn EndPaint(hWnd: Hwnd, lpPaint: *const PaintStruct) -> Bool;
    fn FillRect(hDC: Hdc, lprc: *const Rect, hbr: Brush) -> Int;
    fn GetClipboardData(uFormat: Uint) -> Handle;
    fn GetClientRect(hWnd: Hwnd, lpRect: *mut Rect) -> Bool;
    fn GetCursorPos(lpPoint: *mut Point) -> Bool;
    fn GetDC(hWnd: Hwnd) -> Hdc;
    fn GetMenuItemCount(hMenu: Hmenu) -> Int;
    fn GetMessageW(
        lpMsg: *mut Msg,
        hWnd: Hwnd,
        wMsgFilterMin: Uint,
        wMsgFilterMax: Uint,
    ) -> Bool;
    fn GetMonitorInfoW(hMonitor: Hmonitor, lpmi: *mut MonitorInfo) -> Bool;
    fn GetParent(hWnd: Hwnd) -> Hwnd;
    fn GetKeyState(nVirtKey: Int) -> i16;
    fn GetSystemMetrics(nIndex: Int) -> Int;
    fn GetWindowLongPtrW(hWnd: Hwnd, nIndex: Int) -> isize;
    fn GetWindowRect(hWnd: Hwnd, lpRect: *mut Rect) -> Bool;
    fn InvalidateRect(hWnd: Hwnd, lpRect: *const Rect, bErase: Bool) -> Bool;
    fn IsDialogMessageW(hDlg: Hwnd, lpMsg: *mut Msg) -> Bool;
    fn LoadCursorW(hInstance: Hinstance, lpCursorName: *const u16) -> Cursor;
    fn LoadImageW(
        hInst: Hinstance,
        name: *const u16,
        type_: Uint,
        cx: Int,
        cy: Int,
        fuLoad: Uint,
    ) -> Handle;
    fn MessageBoxW(hWnd: Hwnd, lpText: *const u16, lpCaption: *const u16, uType: Uint) -> Int;
    fn MonitorFromPoint(pt: Point, dwFlags: Dword) -> Hmonitor;
    fn MonitorFromWindow(hwnd: Hwnd, dwFlags: Dword) -> Hmonitor;
    fn MoveWindow(
        hWnd: Hwnd,
        X: Int,
        Y: Int,
        nWidth: Int,
        nHeight: Int,
        bRepaint: Bool,
    ) -> Bool;
    fn OpenClipboard(hWndNewOwner: Hwnd) -> Bool;
    fn PostQuitMessage(nExitCode: Int);
    fn RegisterClassW(lpWndClass: *const WndClassW) -> Word;
    fn RegisterWindowMessageW(lpString: *const u16) -> Uint;
    fn ReleaseCapture() -> Bool;
    fn ReleaseDC(hWnd: Hwnd, hDC: Hdc) -> Int;
    fn ScreenToClient(hWnd: Hwnd, lpPoint: *mut Point) -> Bool;
    fn SendMessageW(hWnd: Hwnd, Msg: Uint, wParam: Wparam, lParam: Lparam) -> Lresult;
    fn SetActiveWindow(hWnd: Hwnd) -> Hwnd;
    fn SetClipboardData(uFormat: Uint, hMem: Hglobal) -> Handle;
    fn SetCapture(hWnd: Hwnd) -> Hwnd;
    fn SetFocus(hWnd: Hwnd) -> Hwnd;
    fn SetTimer(hWnd: Hwnd, nIDEvent: usize, uElapse: Uint, lpTimerFunc: *mut c_void) -> usize;
    fn SetWindowLongPtrW(hWnd: Hwnd, nIndex: Int, dwNewLong: isize) -> isize;
    fn SetWindowPos(
        hWnd: Hwnd,
        hWndInsertAfter: Hwnd,
        X: Int,
        Y: Int,
        cx: Int,
        cy: Int,
        uFlags: Uint,
    ) -> Bool;
    fn SetWindowTextW(hWnd: Hwnd, lpString: *const u16) -> Bool;
    fn ShowWindow(hWnd: Hwnd, nCmdShow: Int) -> Bool;
    fn TranslateAcceleratorW(hWnd: Hwnd, hAccTable: Haccel, lpMsg: *mut Msg) -> Int;
    fn TranslateMessage(lpMsg: *const Msg) -> Bool;
    fn TrackPopupMenu(
        hMenu: Hmenu,
        uFlags: Uint,
        x: Int,
        y: Int,
        nReserved: Int,
        hWnd: Hwnd,
        prcRect: *const Rect,
    ) -> Uint;
    fn KillTimer(hWnd: Hwnd, uIDEvent: usize) -> Bool;
    fn UpdateWindow(hWnd: Hwnd) -> Bool;
}
