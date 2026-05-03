use std::cell::{Cell, UnsafeCell};
use std::collections::HashSet;
use std::env;
use std::ffi::{OsStr, OsString, c_void};
use std::io;
use std::iter::once;
use std::mem::{size_of, transmute, zeroed};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr::{copy_nonoverlapping, null, null_mut};

const INITIAL_WINDOW_WIDTH: i32 = 1024;
const INITIAL_WINDOW_HEIGHT: i32 = 768;

const CLASS_NAME: &str = "Rustpad-XWindowClass";
const MENU_BAR_CLASS_NAME: &str = "Rustpad-XMenuBar";
const PATH_BAR_CLASS_NAME: &str = "Rustpad-XPathBar";
const TAB_BAR_CLASS_NAME: &str = "Rustpad-XTabBar";
const GUTTER_CLASS_NAME: &str = "Rustpad-XLineGutter";
const STATUS_CLASS_NAME: &str = "Rustpad-XStatusBar";
const COMPARE_SPLITTER_CLASS_NAME: &str = "Rustpad-XCompareSplitter";
const GOTO_LINE_CLASS_NAME: &str = "Rustpad-XGotoLineDialog";
const RICH_EDIT_CLASS_NAME: &str = "RICHEDIT50W";
const RICH_EDIT_LIBRARY: &str = "Msftedit.dll";
const APP_TITLE: &str = "Rustpad-X";
const APP_ICON_RESOURCE_ID: usize = 1;
const NEW_WINDOW_ARG: &str = "--rustpad-x-new-window";
const EDIT_CONTROL_ID: usize = 5000;
const COMPARE_EDIT_CONTROL_ID: usize = 5001;
const LINE_PROBE_CONTROL_ID: usize = 5002;
const MENU_BAR_HEIGHT: i32 = 24;
const PATH_BAR_HEIGHT: i32 = 26;
const TAB_BAR_HEIGHT: i32 = 32;
const TAB_CLOSE_BUTTON_SIZE: i32 = 16;
const TAB_CLOSE_BUTTON_MARGIN: i32 = 8;
const GUTTER_WIDTH: i32 = 52;
const STATUS_BAR_HEIGHT: i32 = 22;
const COMPARE_SPLITTER_WIDTH: i32 = 6;
const COMPARE_SPLIT_CENTER: i32 = 5000;
const STATUS_TIMER_ID: usize = 7000;
const STATUS_TIMER_MS: Uint = 1000;
const MENU_SWITCH_TIMER_ID: usize = 7001;
const MENU_SWITCH_TIMER_MS: Uint = 50;
const FOLD_REFRESH_TIMER_ID: usize = 7002;
const FOLD_REFRESH_TIMER_MS: Uint = 250;
const MAX_PROJECT_FILES: usize = 200;
const RECENT_FILE_LIMIT: usize = 10;
const LARGE_TEXT_FEATURE_LIMIT: usize = 2 * 1024 * 1024;
const SESSION_INLINE_TEXT_LIMIT: usize = 2 * 1024 * 1024;
const BASE_TEXT_SIZE_PT: i32 = 10;
const DEFAULT_FONT_FACE: &str = "Consolas";
const FIND_TEXT_BUFFER_LEN: usize = 4096;
const REPLACE_TEXT_BUFFER_LEN: usize = 4096;
const SESSION_FILE_MAX_BYTES: u64 = 8 * 1024 * 1024;
const SESSION_MAX_TABS: usize = 128;
const DEFAULT_ZOOM_PERCENT: i32 = 100;
const MIN_ZOOM_PERCENT: i32 = 50;
const MAX_ZOOM_PERCENT: i32 = 300;
const ZOOM_STEP_PERCENT: i32 = 10;
const FIND_REPLACE_MESSAGE_NAME: &str = "commdlg_FindReplace";
const SESSION_FILE_NAME: &str = "session.txt";
const DWMWA_USE_IMMERSIVE_DARK_MODE: Dword = 20;
const DWMWA_USE_IMMERSIVE_DARK_MODE_BEFORE_20H1: Dword = 19;
const PREFERRED_APP_MODE_FORCE_DARK: Int = 2;
const PREFERRED_APP_MODE_FORCE_LIGHT: Int = 3;
const UXTHEME_ORDINAL_ALLOW_DARK_MODE_FOR_WINDOW: usize = 133;
const UXTHEME_ORDINAL_SET_PREFERRED_APP_MODE: usize = 135;
const UXTHEME_ORDINAL_FLUSH_MENU_THEMES: usize = 136;

const ID_FILE_NEW: u16 = 1001;
const ID_FILE_OPEN: u16 = 1002;
const ID_FILE_SAVE: u16 = 1003;
const ID_FILE_SAVE_AS: u16 = 1004;
const ID_FILE_EXIT: u16 = 1005;
const ID_FILE_CLOSE_TAB: u16 = 1006;
const ID_FILE_SAVE_ALL: u16 = 1007;
const ID_FILE_OPEN_PROJECT: u16 = 1008;
const ID_FILE_PRINT: u16 = 1009;
const ID_FILE_RECENT_BASE: u16 = 1100;
const ID_EDIT_UNDO: u16 = 2001;
const ID_EDIT_CUT: u16 = 2002;
const ID_EDIT_COPY: u16 = 2003;
const ID_EDIT_PASTE: u16 = 2004;
const ID_EDIT_SELECT_ALL: u16 = 2005;
const ID_EDIT_REDO: u16 = 2006;
const ID_EDIT_TIME_DATE: u16 = 2007;
const ID_EDIT_DATE: u16 = 2008;
const ID_EDIT_FIND: u16 = 2009;
const ID_EDIT_FIND_NEXT: u16 = 2010;
const ID_EDIT_FIND_PREVIOUS: u16 = 2011;
const ID_EDIT_REPLACE: u16 = 2012;
const ID_EDIT_FONT: u16 = 2013;
const ID_VIEW_LINE_NUMBERS: u16 = 3001;
const ID_VIEW_FULLSCREEN: u16 = 3002;
const ID_VIEW_ZOOM_IN: u16 = 3003;
const ID_VIEW_ZOOM_OUT: u16 = 3004;
const ID_VIEW_LIGHT_MODE: u16 = 3005;
const ID_VIEW_DARK_MODE: u16 = 3006;
const ID_VIEW_NEXT_TAB: u16 = 3007;
const ID_VIEW_PREVIOUS_TAB: u16 = 3008;
const ID_VIEW_GOTO_LINE: u16 = 3009;
const ID_VIEW_TOP_OF_DOCUMENT: u16 = 3010;
const ID_VIEW_BOTTOM_OF_DOCUMENT: u16 = 3011;
const ID_VIEW_COMPARE_TABS: u16 = 3012;
const ID_VIEW_CLOSE_COMPARE_TABS: u16 = 3013;
const ID_VIEW_SYNC_COMPARE_PAGING: u16 = 3014;
const ID_VIEW_WORD_WRAP: u16 = 3015;
const ID_HELP_ABOUT: u16 = 4001;
const ID_GOTO_LINE_EDIT: u16 = 6001;
const ID_GOTO_LINE_OK: u16 = 6002;
const ID_GOTO_LINE_CANCEL: u16 = 6003;
const ID_COMPARE_TAB_BASE: u16 = 6200;
const ID_TAB_COPY_PATH: u16 = 6301;
const ID_TAB_COPY_NAME: u16 = 6302;
const ID_TAB_REVEAL: u16 = 6303;

// Code folding/collapse controls are disabled; the gutter now only shows line numbers.
#[allow(dead_code)]
const FOLD_BOX_LEFT: i32 = 6;
#[allow(dead_code)]
const FOLD_BOX_SIZE: i32 = 13;
#[allow(dead_code)]
const FOLD_HIT_LEFT: i32 = 0;
#[allow(dead_code)]
const FOLD_HIT_RIGHT: i32 = 30;
#[allow(dead_code)]
const FOLD_NEST_INDENT: i32 = 3;
#[allow(dead_code)]
const FOLD_MAX_NEST_INDENT: i32 = 9;

type Bool = i32;
type Brush = *mut c_void;
type Cursor = *mut c_void;
type Dword = u32;
type Handle = *mut c_void;
type Haccel = *mut c_void;
type Hdc = *mut c_void;
type Hfont = *mut c_void;
type Hgdiobj = *mut c_void;
type Hdrop = *mut c_void;
type Hglobal = *mut c_void;
type Hicon = *mut c_void;
type Hinstance = *mut c_void;
type Hmenu = *mut c_void;
type Hmonitor = *mut c_void;
type Hwnd = *mut c_void;
type Int = i32;
type Long = i32;
type Lparam = isize;
type Lresult = isize;
type Uint = u32;
type Wparam = usize;
type Word = u16;
type WndProc = Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>;

const CS_HREDRAW: Uint = 0x0002;
const CS_VREDRAW: Uint = 0x0001;
const COLOR_WINDOW: usize = 5;
const COLOR_BTNFACE: usize = 15;
const DEFAULT_GUI_FONT: Int = 17;
const FW_NORMAL: Int = 400;
const LF_FACESIZE: usize = 32;
const DEFAULT_CHARSET: Dword = 1;
const OUT_DEFAULT_PRECIS: Dword = 0;
const CLIP_DEFAULT_PRECIS: Dword = 0;
const CLEARTYPE_QUALITY: Dword = 5;
const FF_DONTCARE: Dword = 0;
const LOGPIXELSY: Int = 90;
const IDC_ARROW: *const u16 = 32512usize as *const u16;
const IMAGE_ICON: Uint = 1;
const LR_DEFAULTCOLOR: Uint = 0x0000;
const LR_SHARED: Uint = 0x8000;
const SM_CXICON: Int = 11;
const SM_CYICON: Int = 12;
const SM_CXSMICON: Int = 49;
const SM_CYSMICON: Int = 50;

const WS_OVERLAPPEDWINDOW: isize = 0x00CF0000;
const WS_POPUP: isize = 0x80000000u32 as isize;
const WS_CAPTION: isize = 0x00C00000;
const WS_SYSMENU: isize = 0x00080000;
const WS_VISIBLE: isize = 0x10000000;
const WS_CHILD: isize = 0x40000000;
const WS_VSCROLL: isize = 0x00200000;
const WS_HSCROLL: isize = 0x00100000;
const WS_TABSTOP: isize = 0x00010000;

const ES_LEFT: isize = 0x0000;
const ES_MULTILINE: isize = 0x0004;
const ES_AUTOVSCROLL: isize = 0x0040;
const ES_AUTOHSCROLL: isize = 0x0080;
const ES_NOHIDESEL: isize = 0x0100;
const ES_READONLY: isize = 0x0800;
const ES_WANTRETURN: isize = 0x1000;
const ES_NUMBER: isize = 0x2000;
const WS_EX_CLIENTEDGE: Dword = 0x00000200;
const WS_EX_DLGMODALFRAME: Dword = 0x00000001;
const BS_PUSHBUTTON: isize = 0x00000000;
const BS_DEFPUSHBUTTON: isize = 0x00000001;
const SS_LEFT: isize = 0x00000000;

const SW_SHOW: Int = 5;
const SW_HIDE: Int = 0;
const SWP_NOSIZE: Uint = 0x0001;
const SWP_NOMOVE: Uint = 0x0002;
const SWP_NOZORDER: Uint = 0x0004;
const SWP_NOACTIVATE: Uint = 0x0010;
const SWP_FRAMECHANGED: Uint = 0x0020;
const SWP_SHOWWINDOW: Uint = 0x0040;
const SWP_NOOWNERZORDER: Uint = 0x0200;

const GWL_STYLE: Int = -16;
const GWLP_WNDPROC: Int = -4;
const GWLP_USERDATA: Int = -21;

const WM_CREATE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_SIZE: Uint = 0x0005;
const WM_SETFOCUS: Uint = 0x0007;
const WM_SETREDRAW: Uint = 0x000B;
const WM_CLOSE: Uint = 0x0010;
const WM_PAINT: Uint = 0x000F;
const WM_SETICON: Uint = 0x0080;
const WM_COMMAND: Uint = 0x0111;
const WM_TIMER: Uint = 0x0113;
const WM_HSCROLL: Uint = 0x0114;
const WM_VSCROLL: Uint = 0x0115;
const WM_ERASEBKGND: Uint = 0x0014;
const WM_CTLCOLOREDIT: Uint = 0x0133;
const WM_CTLCOLORSTATIC: Uint = 0x0138;
const WM_MOUSEMOVE: Uint = 0x0200;
const WM_LBUTTONDOWN: Uint = 0x0201;
const WM_LBUTTONUP: Uint = 0x0202;
const WM_RBUTTONUP: Uint = 0x0205;
const WM_CONTEXTMENU: Uint = 0x007B;
const WM_MOUSEWHEEL: Uint = 0x020A;
const ICON_SMALL: Wparam = 0;
const ICON_BIG: Wparam = 1;
const WM_DROPFILES: Uint = 0x0233;
const WM_NCCREATE: Uint = 0x0081;
const WM_NCDESTROY: Uint = 0x0082;
const WM_KEYDOWN: Uint = 0x0100;
const WM_CHAR: Uint = 0x0102;
const WM_SYSKEYDOWN: Uint = 0x0104;
const WM_SYSCHAR: Uint = 0x0106;
const WM_SETFONT: Uint = 0x0030;
const WM_GETTEXT: Uint = 0x000D;
const WM_GETTEXTLENGTH: Uint = 0x000E;
const WM_SETTEXT: Uint = 0x000C;
const WM_UNDO: Uint = 0x0304;
const WM_CUT: Uint = 0x0300;
const WM_COPY: Uint = 0x0301;
const EM_REDO: Uint = 0x0454;
const EM_LINESCROLL: Uint = 0x00B6;
const EM_SCROLLCARET: Uint = 0x00B7;
const EM_GETSEL: Uint = 0x00B0;
const EM_SETSEL: Uint = 0x00B1;
const EM_LINEINDEX: Uint = 0x00BB;
const EM_LINEFROMCHAR: Uint = 0x00C9;
const EM_REPLACESEL: Uint = 0x00C2;
const EM_EMPTYUNDOBUFFER: Uint = 0x00CD;
const EM_GETFIRSTVISIBLELINE: Uint = 0x00CE;
const EM_POSFROMCHAR: Uint = 0x00D6;
const EM_CHARFROMPOS: Uint = 0x00D7;
const EM_SETBKGNDCOLOR: Uint = 0x0443;
const EM_SETCHARFORMAT: Uint = 0x0444;
const EM_SETTARGETDEVICE: Uint = 0x0448;
const EM_FINDTEXTEXW: Uint = 0x047C;
const EM_GETSCROLLPOS: Uint = 0x04DD;
const EM_SETSCROLLPOS: Uint = 0x04DE;

const EN_CHANGE: u16 = 0x0300;
const EN_UPDATE: u16 = 0x0400;
const EN_VSCROLL: u16 = 0x0602;

const MF_STRING: Uint = 0x0000;
const MF_POPUP: Uint = 0x0010;
const MF_CHECKED: Uint = 0x0008;
const MF_GRAYED: Uint = 0x0001;
const MF_UNCHECKED: Uint = 0x0000;
const MF_BYCOMMAND: Uint = 0x0000;
const MF_BYPOSITION: Uint = 0x0400;
const MF_SEPARATOR: Uint = 0x0800;
const TPM_RIGHTBUTTON: Uint = 0x0002;
const TPM_RETURNCMD: Uint = 0x0100;

const MB_OK: Uint = 0x00000000;
const MB_ICONERROR: Uint = 0x00000010;
const MB_ICONINFORMATION: Uint = 0x00000040;

const MONITOR_DEFAULTTOPRIMARY: Dword = 1;
const MONITOR_DEFAULTTONEAREST: Dword = 2;

const OFN_OVERWRITEPROMPT: Dword = 0x00000002;
const OFN_HIDEREADONLY: Dword = 0x00000004;
const OFN_NOCHANGEDIR: Dword = 0x00000008;
const OFN_PATHMUSTEXIST: Dword = 0x00000800;
const OFN_FILEMUSTEXIST: Dword = 0x00001000;
const OFN_EXPLORER: Dword = 0x00080000;

const FR_DOWN: Dword = 0x00000001;
const FR_WHOLEWORD: Dword = 0x00000002;
const FR_MATCHCASE: Dword = 0x00000004;
const FR_FINDNEXT: Dword = 0x00000008;
const FR_REPLACE: Dword = 0x00000010;
const FR_REPLACEALL: Dword = 0x00000020;
const FR_DIALOGTERM: Dword = 0x00000040;

const CF_SCREENFONTS: Dword = 0x00000001;
const CF_INITTOLOGFONTSTRUCT: Dword = 0x00000040;
const CF_FORCEFONTEXIST: Dword = 0x00010000;

const DT_CENTER: Uint = 0x0001;
const DT_RIGHT: Uint = 0x0002;
const DT_VCENTER: Uint = 0x0004;
const DT_SINGLELINE: Uint = 0x0020;
const DT_NOPREFIX: Uint = 0x0800;
const DT_END_ELLIPSIS: Uint = 0x8000;
const TRANSPARENT: Int = 1;

const MK_CONTROL: u16 = 0x0008;
const MK_LBUTTON: u16 = 0x0001;
const CF_UNICODETEXT: Uint = 13;
const GMEM_MOVEABLE: Uint = 0x0002;
const GMEM_ZEROINIT: Uint = 0x0040;

const FVIRTKEY: u8 = 0x01;
const FSHIFT: u8 = 0x04;
const FCONTROL: u8 = 0x08;
const FALT: u8 = 0x10;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0D;
const VK_SHIFT: u16 = 0x10;
const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12;
const VK_ESCAPE: u16 = 0x1B;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_F3: u16 = 0x72;
const VK_F1: u16 = 0x70;
const VK_F11: u16 = 0x7A;
const VK_ADD: u16 = 0x6B;
const VK_SUBTRACT: u16 = 0x6D;
const VK_OEM_PLUS: u16 = 0xBB;
const VK_OEM_MINUS: u16 = 0xBD;

const SCF_SELECTION: Wparam = 0x0001;
const SCF_ALL: Wparam = 0x0004;
const CFM_HIDDEN: Dword = 0x00000100;
const CFM_ITALIC: Dword = 0x00000002;
const CFM_WEIGHT: Dword = 0x00400000;
const CFM_BACKCOLOR: Dword = 0x04000000;
const CFM_FACE: Dword = 0x20000000;
const CFM_COLOR: Dword = 0x40000000;
const CFM_SIZE: Dword = 0x80000000;
const CFE_HIDDEN: Dword = 0x00000100;
const CFE_ITALIC: Dword = 0x00000002;

#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: Long,
    y: Long,
}

#[repr(C)]
struct Size {
    cx: Long,
    cy: Long,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Rect {
    left: Long,
    top: Long,
    right: Long,
    bottom: Long,
}

#[repr(C)]
struct PaintStruct {
    hdc: Hdc,
    fErase: Bool,
    rcPaint: Rect,
    fRestore: Bool,
    fIncUpdate: Bool,
    rgbReserved: [u8; 32],
}

#[repr(C)]
struct TextMetricW {
    tmHeight: Long,
    tmAscent: Long,
    tmDescent: Long,
    tmInternalLeading: Long,
    tmExternalLeading: Long,
    tmAveCharWidth: Long,
    tmMaxCharWidth: Long,
    tmWeight: Long,
    tmOverhang: Long,
    tmDigitizedAspectX: Long,
    tmDigitizedAspectY: Long,
    tmFirstChar: u16,
    tmLastChar: u16,
    tmDefaultChar: u16,
    tmBreakChar: u16,
    tmItalic: u8,
    tmUnderlined: u8,
    tmStruckOut: u8,
    tmPitchAndFamily: u8,
    tmCharSet: u8,
}

#[repr(C)]
struct SystemTime {
    wYear: Word,
    wMonth: Word,
    wDayOfWeek: Word,
    wDay: Word,
    wHour: Word,
    wMinute: Word,
    wSecond: Word,
    wMilliseconds: Word,
}

#[repr(C)]
struct Msg {
    hwnd: Hwnd,
    message: Uint,
    wParam: Wparam,
    lParam: Lparam,
    time: Dword,
    pt: Point,
}

#[repr(C)]
struct WndClassW {
    style: Uint,
    lpfnWndProc: WndProc,
    cbClsExtra: Int,
    cbWndExtra: Int,
    hInstance: Hinstance,
    hIcon: Hicon,
    hCursor: Cursor,
    hbrBackground: Brush,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
}

#[repr(C)]
struct CreateStructW {
    lpCreateParams: *mut c_void,
    hInstance: Hinstance,
    hMenu: Hmenu,
    hwndParent: Hwnd,
    cy: Int,
    cx: Int,
    y: Int,
    x: Int,
    style: Long,
    lpszName: *const u16,
    lpszClass: *const u16,
    dwExStyle: Dword,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MonitorInfo {
    cbSize: Dword,
    rcMonitor: Rect,
    rcWork: Rect,
    dwFlags: Dword,
}

#[repr(C)]
struct OpenFileNameW {
    lStructSize: Dword,
    hwndOwner: Hwnd,
    hInstance: Hinstance,
    lpstrFilter: *const u16,
    lpstrCustomFilter: *mut u16,
    nMaxCustFilter: Dword,
    nFilterIndex: Dword,
    lpstrFile: *mut u16,
    nMaxFile: Dword,
    lpstrFileTitle: *mut u16,
    nMaxFileTitle: Dword,
    lpstrInitialDir: *const u16,
    lpstrTitle: *const u16,
    Flags: Dword,
    nFileOffset: Word,
    nFileExtension: Word,
    lpstrDefExt: *const u16,
    lCustData: Lparam,
    lpfnHook: *mut c_void,
    lpTemplateName: *const u16,
    pvReserved: *mut c_void,
    dwReserved: Dword,
    FlagsEx: Dword,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LogFontW {
    lfHeight: Long,
    lfWidth: Long,
    lfEscapement: Long,
    lfOrientation: Long,
    lfWeight: Long,
    lfItalic: u8,
    lfUnderline: u8,
    lfStrikeOut: u8,
    lfCharSet: u8,
    lfOutPrecision: u8,
    lfClipPrecision: u8,
    lfQuality: u8,
    lfPitchAndFamily: u8,
    lfFaceName: [u16; LF_FACESIZE],
}

#[repr(C)]
struct FindReplaceW {
    lStructSize: Dword,
    hwndOwner: Hwnd,
    hInstance: Hinstance,
    Flags: Dword,
    lpstrFindWhat: *mut u16,
    lpstrReplaceWith: *mut u16,
    wFindWhatLen: Word,
    wReplaceWithLen: Word,
    lCustData: Lparam,
    lpfnHook: *mut c_void,
    lpTemplateName: *const u16,
}

#[repr(C)]
struct ChooseFontW {
    lStructSize: Dword,
    hwndOwner: Hwnd,
    hDC: Hdc,
    lpLogFont: *mut LogFontW,
    iPointSize: Int,
    Flags: Dword,
    rgbColors: Dword,
    lCustData: Lparam,
    lpfnHook: *mut c_void,
    lpTemplateName: *const u16,
    hInstance: Hinstance,
    lpszStyle: *mut u16,
    nFontType: Word,
    ___MISSING_ALIGNMENT__: Word,
    nSizeMin: Int,
    nSizeMax: Int,
}

#[repr(C)]
struct CharFormat2W {
    cbSize: Uint,
    dwMask: Dword,
    dwEffects: Dword,
    yHeight: Long,
    yOffset: Long,
    crTextColor: Dword,
    bCharSet: u8,
    bPitchAndFamily: u8,
    szFaceName: [u16; LF_FACESIZE],
    wWeight: Word,
    sSpacing: i16,
    crBackColor: Dword,
    lcid: Dword,
    dwReserved: Dword,
    sStyle: i16,
    wKerning: Word,
    bUnderlineType: u8,
    bAnimation: u8,
    bRevAuthor: u8,
    bUnderlineColor: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Accel {
    fVirt: u8,
    key: Word,
    cmd: Word,
}

#[derive(Debug)]
struct FullscreenSnapshot {
    rect: Rect,
    style: isize,
}

struct GotoLineState {
    parent: Hwnd,
    edit: Hwnd,
    max_line: usize,
    result: Option<usize>,
    done: bool,
}

enum SessionTab {
    File {
        path: PathBuf,
        text: Option<String>,
        line_ending: Option<LineEnding>,
    },
    Untitled {
        name: String,
        text: String,
        line_ending: LineEnding,
    },
}

struct SessionState {
    recent_files: Vec<PathBuf>,
    tabs: Vec<SessionTab>,
    active_tab: usize,
    compare_page_sync: bool,
    word_wrap_enabled: bool,
}

#[derive(Clone, Copy)]
struct FindOptions {
    down: bool,
    match_case: bool,
    whole_word: bool,
}

#[derive(Clone, Copy)]
struct TextMatch {
    start_utf16: i32,
    end_utf16: i32,
}

#[repr(C)]
struct CharRange {
    cpMin: Long,
    cpMax: Long,
}

#[repr(C)]
struct FindTextExW {
    chrg: CharRange,
    lpstrText: *const u16,
    chrgText: CharRange,
}

struct SelectionStatus {
    chars: usize,
    bytes: Option<usize>,
    lines: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineEnding {
    Lf,
    Crlf,
    Cr,
    Mixed,
}

impl LineEnding {
    fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
            Self::Cr => "CR",
            Self::Mixed => "Mixed",
        }
    }

    fn save_sequence(self) -> &'static str {
        match self {
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
            Self::Lf | Self::Mixed => "\n",
        }
    }

    fn session_token(self) -> &'static str {
        match self {
            Self::Lf => "lf",
            Self::Crlf => "crlf",
            Self::Cr => "cr",
            Self::Mixed => "mixed",
        }
    }

    fn from_session_token(value: &str) -> Option<Self> {
        match value {
            "lf" => Some(Self::Lf),
            "crlf" => Some(Self::Crlf),
            "cr" => Some(Self::Cr),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct FoldRange {
    start_utf16: i32,
    end_utf16: i32,
    hidden_start_utf16: i32,
    hidden_end_utf16: i32,
    start_line: i32,
    end_line: i32,
    collapsed: bool,
}

#[derive(Clone, Copy)]
struct GutterRow {
    line_index: i32,
    top: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug)]
struct LineProbeMetrics {
    line_index: i32,
    top: i32,
    height: i32,
    bottom: i32,
}

struct StartupRequest {
    paths: Vec<PathBuf>,
    persist_session: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

#[derive(Clone, Copy)]
struct ThemePalette {
    path_background: Dword,
    path_text: Dword,
    path_muted_text: Dword,
    tab_strip_background: Dword,
    tab_active_background: Dword,
    tab_inactive_background: Dword,
    tab_border: Dword,
    tab_text: Dword,
    tab_active_text: Dword,
    editor_background: Dword,
    editor_text: Dword,
    gutter_background: Dword,
    gutter_separator: Dword,
    gutter_text: Dword,
    status_background: Dword,
    status_separator: Dword,
    status_text: Dword,
}

impl Theme {
    fn palette(self) -> ThemePalette {
        match self {
            Self::Light => ThemePalette {
                path_background: 0x00F4F4F4,
                path_text: 0x00202020,
                path_muted_text: 0x006A6A6A,
                tab_strip_background: 0x00E8E8E8,
                tab_active_background: 0x00FFFFFF,
                tab_inactive_background: 0x00DCDCDC,
                tab_border: 0x00BDBDBD,
                tab_text: 0x00303030,
                tab_active_text: 0x00000000,
                editor_background: 0x00FFFFFF,
                editor_text: 0x00202020,
                gutter_background: 0x00F2F2F2,
                gutter_separator: 0x00C8C8C8,
                gutter_text: 0x00666666,
                status_background: 0x00F2F2F2,
                status_separator: 0x00D0D0D0,
                status_text: 0x00252525,
            },
            Self::Dark => ThemePalette {
                path_background: 0x001A1D23,
                path_text: 0x00E6E6E6,
                path_muted_text: 0x00A8B0BC,
                tab_strip_background: 0x00111111,
                tab_active_background: 0x000B0E14,
                tab_inactive_background: 0x002B2B2B,
                tab_border: 0x003C414A,
                tab_text: 0x00D6D6D6,
                tab_active_text: 0x00FFFFFF,
                editor_background: 0x001E1E1E,
                editor_text: 0x00F0F0F0,
                gutter_background: 0x00252525,
                gutter_separator: 0x00404040,
                gutter_text: 0x009A9A9A,
                status_background: 0x002B2B2B,
                status_separator: 0x00404040,
                status_text: 0x00ECECEC,
            },
        }
    }
}

struct Document {
    path: Option<PathBuf>,
    text: String,
    line_count: usize,
    line_starts: Vec<i32>,
    line_ending: LineEnding,
    untitled_name: String,
}

impl Document {
    fn untitled(index: usize) -> Self {
        Self {
            path: None,
            text: String::new(),
            line_count: 1,
            line_starts: vec![0],
            line_ending: LineEnding::Lf,
            untitled_name: format!("Untitled {index}"),
        }
    }

    fn from_file(path: PathBuf, text: String, line_ending: LineEnding) -> Self {
        let line_count = logical_line_count(&text);
        let line_starts = line_starts_utf16(&text);
        Self {
            path: Some(path),
            text,
            line_count,
            line_starts,
            line_ending,
            untitled_name: String::new(),
        }
    }

    fn from_open_file(path: PathBuf, raw_text: String, _use_rich_edit: bool) -> Self {
        let line_ending = detect_line_ending(&raw_text);
        Self::from_file(path, to_edit_line_endings(&raw_text), line_ending)
    }

    fn untitled_with_text(name: String, text: String, line_ending: LineEnding) -> Self {
        let line_count = logical_line_count(&text);
        let line_starts = line_starts_utf16(&text);
        Self {
            path: None,
            text,
            line_count,
            line_starts,
            line_ending,
            untitled_name: if name.is_empty() {
                "Untitled 1".to_string()
            } else {
                name
            },
        }
    }

    fn set_text(&mut self, text: String) {
        self.line_count = logical_line_count(&text);
        self.line_starts = line_starts_utf16(&text);
        self.text = text;
    }

    fn display_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .map(str::to_owned)
            .unwrap_or_else(|| self.untitled_name.clone())
    }

    fn location(&self) -> String {
        self.path
            .as_ref()
            .map(|path| display_path(path))
            .unwrap_or_else(|| "Unsaved".to_string())
    }

    fn is_empty_untitled(&self) -> bool {
        self.path.is_none() && self.text.is_empty()
    }

    fn is_empty_initial_untitled(&self) -> bool {
        self.is_empty_untitled() && self.untitled_name == "Untitled 1"
    }

    fn is_initial_untitled(&self) -> bool {
        self.path.is_none() && self.untitled_name == "Untitled 1"
    }
}

struct AppData {
    hwnd: Hwnd,
    menu_bar: Hwnd,
    path_bar: Hwnd,
    tab_bar: Hwnd,
    edit: Hwnd,
    compare_edit: Hwnd,
    compare_splitter: Hwnd,
    line_probe: Hwnd,
    gutter: Hwnd,
    compare_gutter: Hwnd,
    status: Hwnd,
    _menu: Hmenu,
    file_menu: Hmenu,
    edit_menu: Hmenu,
    view_menu: Hmenu,
    help_menu: Hmenu,
    recent_menu: Hmenu,
    font: Hfont,
    font_face: String,
    font_size_pt: i32,
    font_weight: Int,
    font_italic: bool,
    zoom_percent: i32,
    theme: Theme,
    editor_background_brush: Brush,
    line_numbers_visible: bool,
    word_wrap_enabled: bool,
    documents: Vec<Document>,
    recent_files: Vec<PathBuf>,
    active_tab: usize,
    tab_hover_index: Option<usize>,
    tab_hover_close_index: Option<usize>,
    gutter_hover_line: Option<i32>,
    next_untitled_index: usize,
    status_left_cache: String,
    status_right_cache: String,
    fullscreen: Option<FullscreenSnapshot>,
    compare_tab: Option<usize>,
    compare_split_ratio: i32,
    compare_dragging: bool,
    compare_page_sync: bool,
    running_as_administrator: bool,
    find_dialog: Hwnd,
    replace_dialog: Hwnd,
    find_buffer: Vec<u16>,
    replace_buffer: Vec<u16>,
    find_state: FindReplaceW,
    replace_state: FindReplaceW,
    find_message: Uint,
    last_find_options: FindOptions,
    use_rich_edit: bool,
    persist_session: bool,
    programmatic_text_update: bool,
    active_menu_index: Option<usize>,
    pending_menu_index: Option<usize>,
    menu_popup_active: bool,
    last_gutter_first_visible_line: i32,
    line_probe_metrics: Option<LineProbeMetrics>,
    line_probe_cache: String,
    fold_ranges: Vec<FoldRange>,
    fold_formats_active: bool,
    fold_refresh_pending: bool,
    fold_refresh_timer_active: bool,
}

impl AppData {
    fn new(menus: MainMenus, use_rich_edit: bool, startup: StartupRequest) -> io::Result<Self> {
        let theme = Theme::Dark;
        let find_message =
            unsafe { RegisterWindowMessageW(to_wide(FIND_REPLACE_MESSAGE_NAME).as_ptr()) };
        let session = if startup.persist_session {
            load_session_state()
        } else {
            empty_session_state()
        };
        let (documents, active_tab) = if startup.paths.is_empty() {
            restored_session_documents(&session, use_rich_edit)
        } else {
            startup_documents(&startup.paths, use_rich_edit)
        };
        let next_untitled_index = next_available_untitled_index(&documents);
        let recent_files = if startup.persist_session {
            restored_recent_files(&session)
        } else {
            Vec::new()
        };

        Ok(Self {
            hwnd: null_mut(),
            menu_bar: null_mut(),
            path_bar: null_mut(),
            tab_bar: null_mut(),
            edit: null_mut(),
            compare_edit: null_mut(),
            compare_splitter: null_mut(),
            line_probe: null_mut(),
            gutter: null_mut(),
            compare_gutter: null_mut(),
            status: null_mut(),
            _menu: menus.menu,
            file_menu: menus.file_menu,
            edit_menu: menus.edit_menu,
            view_menu: menus.view_menu,
            help_menu: menus.help_menu,
            recent_menu: menus.recent_menu,
            font: null_mut(),
            font_face: DEFAULT_FONT_FACE.to_string(),
            font_size_pt: BASE_TEXT_SIZE_PT,
            font_weight: FW_NORMAL,
            font_italic: false,
            zoom_percent: DEFAULT_ZOOM_PERCENT,
            theme,
            editor_background_brush: create_theme_editor_brush(theme)?,
            line_numbers_visible: true,
            word_wrap_enabled: if startup.persist_session {
                session.word_wrap_enabled
            } else {
                true
            },
            documents,
            recent_files,
            active_tab,
            tab_hover_index: None,
            tab_hover_close_index: None,
            gutter_hover_line: None,
            next_untitled_index,
            status_left_cache: String::new(),
            status_right_cache: String::new(),
            fullscreen: None,
            compare_tab: None,
            compare_split_ratio: COMPARE_SPLIT_CENTER,
            compare_dragging: false,
            compare_page_sync: startup.persist_session && session.compare_page_sync,
            running_as_administrator: is_process_running_as_administrator(),
            find_dialog: null_mut(),
            replace_dialog: null_mut(),
            find_buffer: vec![0; FIND_TEXT_BUFFER_LEN],
            replace_buffer: vec![0; REPLACE_TEXT_BUFFER_LEN],
            find_state: unsafe { zeroed() },
            replace_state: unsafe { zeroed() },
            find_message,
            last_find_options: FindOptions {
                down: true,
                match_case: false,
                whole_word: false,
            },
            use_rich_edit,
            persist_session: startup.persist_session,
            programmatic_text_update: false,
            active_menu_index: None,
            pending_menu_index: None,
            menu_popup_active: false,
            last_gutter_first_visible_line: 0,
            line_probe_metrics: None,
            line_probe_cache: String::new(),
            fold_ranges: Vec::new(),
            fold_formats_active: false,
            fold_refresh_pending: false,
            fold_refresh_timer_active: false,
        })
    }
}

impl Drop for AppData {
    fn drop(&mut self) {
        if !self.find_dialog.is_null() {
            unsafe {
                DestroyWindow(self.find_dialog);
            }
        }
        if !self.replace_dialog.is_null() {
            unsafe {
                DestroyWindow(self.replace_dialog);
            }
        }
        if !self.font.is_null() {
            unsafe {
                DeleteObject(self.font as Hgdiobj);
            }
        }
        if !self.editor_background_brush.is_null() {
            unsafe {
                DeleteObject(self.editor_background_brush as Hgdiobj);
            }
        }
    }
}

include!("session.rs");

include!("app.rs");

include!("editor.rs");

include!("chrome.rs");

include!("commands.rs");

include!("search.rs");

include!("support.rs");

include!("ffi.rs");
