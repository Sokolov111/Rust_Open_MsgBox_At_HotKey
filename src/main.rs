use winapi::um::winuser::{
    RegisterClassExW, CreateWindowExW, DefWindowProcW, DispatchMessageW,
    GetMessageW, ShowWindow, UpdateWindow, TranslateMessage, WM_DESTROY, WNDCLASSEXW, SW_SHOW, WS_OVERLAPPEDWINDOW,
    CW_USEDEFAULT
};

use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::windef::{HWND, HMENU};
use winapi::shared::minwindef::{HINSTANCE, LRESULT, LPARAM, WPARAM, UINT};

use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::execute;

use winapi::um::winuser::*;

use std::ptr::null_mut;
use std::io::{stdout, Write};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// Структура для окна
struct Window {
    hwnd: HWND,
}

// Реализация методов
impl Window {
    pub fn new(h_instance: HINSTANCE, class_name: &[u16], window_title: &[u16]) -> Option<Self> {
        unsafe {
            // Регистрация класса окна
            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: null_mut(),
                lpszMenuName: null_mut(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: null_mut(),
            };
            RegisterClassExW(&wnd_class);

            // Создание окна
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT, CW_USEDEFAULT,
                200, 200,
                null_mut(),
                null_mut(),
                h_instance,
                null_mut(),
            );

            if hwnd.is_null() {
                return None;
            }

            // Создание кнопки
            let button_hwnd = CreateWindowExW(
                0,
                encode_utf16("BUTTON\0").as_ptr(),
                encode_utf16("Click Me\0").as_ptr(),
                WS_VISIBLE | WS_CHILD,
                50, 50, 100, 30, // x, y, width, height
                hwnd,
                1001 as HMENU, // ID кнопки
                h_instance,
                null_mut(),
            );

            if button_hwnd.is_null() {
                return None;
            }

            Some(Window { hwnd })
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
            UpdateWindow(self.hwnd);
        }
    }

    pub fn run_message_loop(&self) {
        unsafe {
            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

// Функция обработки сообщений
unsafe extern "system" fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            if wparam == 1001 {
                // Обработка нажатия кнопки (ID = 1001)
                MessageBoxW(
                    hwnd,
                    encode_utf16("Button clicked!\0").as_ptr(),
                    encode_utf16("Action\0").as_ptr(),
                    MB_OK,
                );
            }
            0
        }
        WM_DESTROY => {
            winapi::um::winuser::PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// Вспомогательная функция для преобразования строки в UTF-16
fn encode_utf16(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn main() {
    let mut stdout = stdout();
    enable_raw_mode().unwrap();

    // Очистка экрана и печать сообщения
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0), Print(r#"ctrl + q to exit, ctrl + h to open new Window and after Msg Box"#))
            .unwrap();

    // Поиск нажатий
    loop {
        // Переход в верхний-левый угол
        execute!(stdout, cursor::MoveTo(0, 0)).unwrap();

        match read().unwrap() {
            Event::Key(KeyEvent { // при нажатии комбинации Ctrl+h
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
                
            }) => {
                unsafe{
                    // Получение дескриптора приложения
                    let h_instance = GetModuleHandleW(null_mut());

                    let class_name = encode_utf16("MyWindowClass\0");
                    let window_title = encode_utf16("My Window\0");

                    // Создание окна
                    let window = match Window::new(h_instance, &class_name, &window_title) {
                        Some(w) => w,
                        None => {
                            println!("Ошибка создания окна");
                            return;
                        }
                    };

                    // Отобразить окно
                    window.show();
                    window.run_message_loop();
                    
                };
            },
            Event::Key(KeyEvent { // при нажатии комбинации Ctrl+q
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }) => break, // Остановить код
            _ => (),
        }
    }

    // Отключение 
    disable_raw_mode().unwrap();
}
