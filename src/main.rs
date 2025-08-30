use chrono::{DateTime, Local, Datelike, Timelike, Duration, TimeZone};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::Threading::*;
//use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use rand::{random, Rng};
use windows::Win32::UI::WindowsAndMessaging::BS_PUSHBUTTON;
use windows::Win32::UI::WindowsAndMessaging::WS_CHILD;
use windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE;
//use windows::Win32::Foundation::HMENU;
use windows::Win32::Foundation::HINSTANCE;

// 全局变量
static mut TARGET_TIME: Option<DateTime<Local>> = None;
static mut COUNTDOWN_TEXT: String = String::new();
const TIMER_ID: usize = 1;
static mut BUTTON_HWND: HWND = HWND(std::ptr::null_mut());
const BUTTON_ID: i32 = 1001;

fn main() -> Result<()> {
    //unsafe {
    //    let console_window = GetConsoleWindow();
    //    if !console_window.is_null() {
    //        ShowWindow(console_window, SW_HIDE);
    //    }
    //}

    let now = Local::now();
    let year = now.year();
    let target_time = Local
        .with_ymd_and_hms(year, 9, 1, 8, 0, 0)
        .single()
        .unwrap_or_else(|| Local.with_ymd_and_hms(year + 1, 9, 1, 8, 0, 0).unwrap());

    unsafe {
        TARGET_TIME = Some(target_time);
        update_countdown_text();
    }

    create_countdown_window()
}

fn string_to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect()
}

//  let hwnd =
unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let cmd_id = (wparam.0 as u32) & 0xFFFF;
            if cmd_id == BUTTON_ID as u32 {
                btn();
                return LRESULT(0);
            }
        }
        WM_TIMER => {
            if wparam.0 as usize == TIMER_ID {
                update_countdown_text();
                InvalidateRect(Some(hwnd), None, true);

                if let Some(target_time) = TARGET_TIME {
                    if Local::now() >= target_time {
                        kaixue();
                        PostQuitMessage(0);
                    }
                }

                return LRESULT(0);
            }
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let background_brush = GetSysColorBrush(COLOR_3DFACE);
            FillRect(hdc, &ps.rcPaint, background_brush);

            let text = unsafe { (*std::ptr::addr_of_mut!(COUNTDOWN_TEXT)).clone() };
            let lines: Vec<&str> = text.split('\n').collect();

            //SetTextColor(hdc, COLORREF(RGB(0, 0, 0))); 

            let mut y_pos = 30;
            for line in lines {
                if !line.trim().is_empty() {
                    let line_wide = string_to_wstring(line);
                    TextOutW(
                        hdc,
                        30,
                        y_pos,
                        &line_wide
                    );
                    y_pos += 25; // 增加行间距
                }
            }

            EndPaint(hwnd, &ps);
            return LRESULT(0);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            return LRESULT(0);
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn create_countdown_window() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let hinstance = HINSTANCE(instance.0);
        let class_name = w!("KaixueWindowClass");

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_procedure),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE::from(instance),
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(((COLOR_WINDOW.0 as isize) + 1) as *mut core::ffi::c_void),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: LoadIconW(None, IDI_APPLICATION)?,
        };

        RegisterClassExW(&wc);

        let width = 400;
        let height = 250;
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("⚠️开学倒计时⚠️"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            x,
            y,
            width,
            height,
            None,
            None,
            Some(hinstance),
            None,
        )?;

        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect);
        let window_width = rect.right - rect.left;
        let window_height = rect.bottom - rect.top;

        let button_width = 120;
        let button_height = 30;
        let button_x = (window_width - button_width) / 2;
        let button_y = window_height - button_height - 20; // 底部留20像素边距
        //let button_style = WS_CHILD.0 | WS_VISIBLE.0;
        BUTTON_HWND = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            w!("我不想开学！"),
            WS_CHILD | WS_VISIBLE,
            button_x,
            button_y,
            button_width,
            button_height,
            Some(hwnd),
            Some(HMENU(BUTTON_ID as isize as *mut core::ffi::c_void)), // HMENU 需要包装为 Option<HMENU>
            Some(HINSTANCE(instance.0)), // HINSTANCE 需要包装为 Option<HINSTANCE>
            None,
        )?;

        SetWindowPos(
            hwnd,
            Option::from(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );

        SetTimer(Option::from(hwnd), TIMER_ID, 1000, None);

        update_countdown_text();
        InvalidateRect(Option::from(hwnd), None, true);

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        KillTimer(Option::from(hwnd), TIMER_ID);

        Ok(())
    }
}

fn kaixue(){
    unsafe {
        MessageBoxW(
            None,
            w!("日历撕至末页，暑假终于像捧不住的流水，从指缝间漏尽了。站在校门口的那一瞬，胸中涌起的情感竟比书包更沉几分——那是未读完的小说还夹着暑气的书签，是深夜冰西瓜与好友的傻笑，是计划表上未打完勾的旅行目的地，统统被上课铃碾成碎片。/n分明听见两个自己在争吵：一个雀跃于新课本的墨香，另一个却拼命拽住暑假的衣角。古人叹“逝者如斯夫”，原来两千年前的孔子也曾这样目送时间离去。我们总在告别后才学会珍惜，在结束时才开始怀念。/n可成长本就是一场温柔的告别式。那些未完成的遗憾，终会成为照亮前路的光——毕竟，所有盛夏的欢愉，都是为了沉淀成秋天厚积薄发的勇气。"),
            w!("夏逝秋启，惜憾前行。"),
            MB_OK | MB_SETFOREGROUND | MB_TOPMOST,
        );
    }
    println!("夏逝秋启，惜憾前行。")
}
fn btn(){
    let mut rng = rand::thread_rng();
    let r: usize = rng.gen_range(0..8);
    let mingyan:[&str;8] = ["“逝者如斯夫，不舍昼夜。” ——孔子《论语》\n【释义】时间如流水奔涌，日夜不停，警示世人莫虚度。","“一寸光阴一寸金，寸金难买寸光阴。” ——《增广贤文》\n【释义】光阴比黄金更珍贵，因其不可复得。","“盛年不重来，一日难再晨。” ——陶渊明《杂诗》\n【释义】青春与良辰皆无二次，当下即是最贵。","“不贵尺之璧，而重寸之阴。” ——《淮南子》\n【释义】宝玉易得，光阴难买，轻物重时方为智者。","“时间就是性命。” ——鲁迅\n【释义】浪费他人时间如谋财害命，浪费自己时间如慢性自杀。","“抛弃时间的人，时间也抛弃他。” ——莎士比亚\n【释义】怠慢时间者，终被时间反噬。","“你热爱生命吗？那么别浪费时间，因为它是生命的材料。” ——富兰克林\n【释义】时间构成生命本质，虚度即浪费生命。","“未来姗姗来迟，现在像箭一样飞逝，过去永远静立不动。” ——席勒\n【释义】唯有抓住当下，方能不悔过去、不惧未来。"];
    let text = mingyan[r];
    let message_wide = string_to_wstring(&text);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(message_wide.as_ptr()),
            w!("时光匆匆，且玩且惜。"),
            MB_OK | MB_SETFOREGROUND | MB_TOPMOST,
        );
    }
    println!("{}", text)
}

unsafe fn update_countdown_text() {
    if let Some(target_time) = TARGET_TIME {
        let now = Local::now();

        if now >= target_time {
            COUNTDOWN_TEXT = "开学了".to_string();
            return;
        }

        let duration = target_time - now;
        let seconds_remaining = duration.num_seconds();

        let target_year = target_time.year();
        let target_month = target_time.month();
        let target_day = target_time.day();
        let target_hour = target_time.hour();
        let target_minute = target_time.minute();
        let target_second = target_time.second();

        let date_str = format!(
            "{:04}年{:02}月{:02}日{:02}时{:02}分{:02}秒",
            target_year, target_month, target_day, target_hour, target_minute, target_second
        );
        fn format_duration_detailed(duration: Duration) -> String {
            let total_seconds = duration.num_seconds();
            let days = total_seconds / 86400;
            let hours = (total_seconds % 86400) / 3600;
            let minutes = (total_seconds % 3600) / 60;
            let seconds = total_seconds % 60;

            if days > 0 {
                format!("{}天{}小时{}分{}秒", days, hours, minutes, seconds)
            } else if hours > 0 {
                format!("{}小时{}分{}秒", hours, minutes, seconds)
            } else if minutes > 0 {
                format!("{}分{}秒", minutes, seconds)
            } else {
                format!("{}秒", seconds)
            }
        }

        COUNTDOWN_TEXT = format!(
            "目标时间: {};\n剩余秒数: {} 秒;\n剩余时间：{}。",
            date_str, seconds_remaining, format_duration_detailed(duration)
        );
    }
}

/*
                                                                                                                                                      
                                                                                                                                                      
                                                                                                                        $$      $$$$         $$       
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$        $$$$$     $$$$$       $$$$$     
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$         $$$$$$    $$$$$     $$$$$      
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$              $$$$    $$$$$                      $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
                $$$$$$$                  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$             $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
               $$$$$$                    $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                         $$$$  
             $$$$$$$$$$$$                $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                    $    $$$$  
            $$$$$$$$$$$$$$$$             $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$ $$$$$$$$$$$$$$$$$$$$$$$ $$$$  
         $$$$$$$$$$$$  $$$$$$$$          $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                      $$$$$$$$         
       $$$$$$$  $$$$$    $$$$$$$$                  $$$$$                    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                  $$$$$$$$$            
   $$$$$$$$$    $$$$$       $$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$$         $$$$                          $$$$$                
 $$$$$$$$       $$$$$         $$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$          $$$$           $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ 
   $$$          $$$$$            $$           $$$$$$        $$$$$$                  $$$$$          $$$$                          $$$$                 
                $$$$$                        $$$$$$$       $$$$$$                 $$$$$$           $$$$                          $$$$                 
                $$$$$                       $$$$$$$$$$$$$$$$$$$                  $$$$$$            $$$$                          $$$$                 
                $$$$$                          $$$$$$$$$$$$$$$$$$$$$$         $$$$$$$$             $$$$                          $$$$                 
                $$$$$                  $$$$$$$$$$$$$$$$       $$$$$$$$$$$   $$$$$$$$               $$$$                   $$$$$$$$$$$                 
                                                                                                                                                      
                                                                                                                                                      
                                                                                                                        $$      $$$$         $$       
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$        $$$$$     $$$$$       $$$$$     
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$         $$$$$$    $$$$$     $$$$$      
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$              $$$$    $$$$$                      $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
                $$$$$$$                  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$             $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
               $$$$$$                    $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                         $$$$  
             $$$$$$$$$$$$                $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                    $    $$$$  
            $$$$$$$$$$$$$$$$             $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$ $$$$$$$$$$$$$$$$$$$$$$$ $$$$  
         $$$$$$$$$$$$  $$$$$$$$          $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                      $$$$$$$$         
       $$$$$$$  $$$$$    $$$$$$$$                  $$$$$                    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                  $$$$$$$$$            
   $$$$$$$$$    $$$$$       $$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$$         $$$$                          $$$$$                
 $$$$$$$$       $$$$$         $$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$          $$$$           $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ 
   $$$          $$$$$            $$           $$$$$$        $$$$$$                  $$$$$          $$$$                          $$$$                 
                $$$$$                        $$$$$$$       $$$$$$                 $$$$$$           $$$$                          $$$$                 
                $$$$$                       $$$$$$$$$$$$$$$$$$$                  $$$$$$            $$$$                          $$$$                 
                $$$$$                          $$$$$$$$$$$$$$$$$$$$$$         $$$$$$$$             $$$$                          $$$$                 
                $$$$$                  $$$$$$$$$$$$$$$$       $$$$$$$$$$$   $$$$$$$$               $$$$                   $$$$$$$$$$$                 
                                                                                                                                                      
                                                                                                                                                      
                                                                                                                        $$      $$$$         $$       
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$        $$$$$     $$$$$       $$$$$     
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$         $$$$$$    $$$$$     $$$$$      
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$              $$$$    $$$$$                      $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
                $$$$$$$                  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$             $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
               $$$$$$                    $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                         $$$$  
             $$$$$$$$$$$$                $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                    $    $$$$  
            $$$$$$$$$$$$$$$$             $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$ $$$$$$$$$$$$$$$$$$$$$$$ $$$$  
         $$$$$$$$$$$$  $$$$$$$$          $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                      $$$$$$$$         
       $$$$$$$  $$$$$    $$$$$$$$                  $$$$$                    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                  $$$$$$$$$            
   $$$$$$$$$    $$$$$       $$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$$         $$$$                          $$$$$                
 $$$$$$$$       $$$$$         $$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$          $$$$           $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ 
   $$$          $$$$$            $$           $$$$$$        $$$$$$                  $$$$$          $$$$                          $$$$                 
                $$$$$                        $$$$$$$       $$$$$$                 $$$$$$           $$$$                          $$$$                 
                $$$$$                       $$$$$$$$$$$$$$$$$$$                  $$$$$$            $$$$                          $$$$                 
                $$$$$                          $$$$$$$$$$$$$$$$$$$$$$         $$$$$$$$             $$$$                          $$$$                 
                $$$$$                  $$$$$$$$$$$$$$$$       $$$$$$$$$$$   $$$$$$$$               $$$$                   $$$$$$$$$$$                 
                                                                                                                                                      
                                                                                                                                                      
                                                                                                                        $$      $$$$         $$       
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$        $$$$$     $$$$$       $$$$$     
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$         $$$$$$    $$$$$     $$$$$      
  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$              $$$$    $$$$$                      $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
                $$$$$$$                  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$             $$$$$         $$$$            $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$  
               $$$$$$                    $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                         $$$$  
             $$$$$$$$$$$$                $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$                    $    $$$$  
            $$$$$$$$$$$$$$$$             $$$$     $$$$    $$$$$    $$$$$             $$$$$         $$$$            $$$$ $$$$$$$$$$$$$$$$$$$$$$$ $$$$  
         $$$$$$$$$$$$  $$$$$$$$          $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                      $$$$$$$$         
       $$$$$$$  $$$$$    $$$$$$$$                  $$$$$                    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$                  $$$$$$$$$            
   $$$$$$$$$    $$$$$       $$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$$         $$$$                          $$$$$                
 $$$$$$$$       $$$$$         $$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$          $$$$$          $$$$           $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ 
   $$$          $$$$$            $$           $$$$$$        $$$$$$                  $$$$$          $$$$                          $$$$                 
                $$$$$                        $$$$$$$       $$$$$$                 $$$$$$           $$$$                          $$$$                 
                $$$$$                       $$$$$$$$$$$$$$$$$$$                  $$$$$$            $$$$                          $$$$                 
                $$$$$                          $$$$$$$$$$$$$$$$$$$$$$         $$$$$$$$             $$$$                          $$$$                 
                $$$$$                  $$$$$$$$$$$$$$$$       $$$$$$$$$$$   $$$$$$$$               $$$$                   $$$$$$$$$$$                 

 */