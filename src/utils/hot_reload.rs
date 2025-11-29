use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::os::raw::c_char;

use inotify::{Inotify, WatchMask};
use libc::{O_RDONLY, O_WRONLY, c_void, mkfifo};

use crate::launcher::Launcher;

fn pipe_writer_thread(watch_path: &Path, pipe_box: Box<dyn Fn(i32) -> (i32, [c_char; 20])>) {
    println!("Watch path: {:?}", watch_path);
    let mut inotify = Inotify::init().expect("Error while initializing inotify instance");
    inotify
        .watches()
        .add(watch_path, WatchMask::MODIFY | WatchMask::CLOSE)
        .expect("Failed to add file watch");
    let (fd, buffer) = pipe_box(O_WRONLY);
    let mut buffer2 = [0; 1024];
    loop {
        'outer: {
            match inotify.read_events_blocking(&mut buffer2) {
                Ok(events) => {
                    println!("inotify event");
                    for event in events {
                        println!("{:?}", event.mask);
                        if !matches!(event.mask, inotify::EventMask::CLOSE_NOWRITE) {
                            unsafe {
                                libc::write(fd, buffer.as_ptr() as *mut c_void, 1);
                            }
                            break 'outer;
                        }
                    }
                }
                Err(error) => {
                    println!("inotify err: {:?}", error);
                }
            }
        }
    }
}

pub fn attach(css_path: &Path, launcher_cell: Rc<RefCell<Launcher>>) {
    use gtk::prelude::FileExt;
    let launcher = launcher_cell.borrow_mut();
    let css_path = launcher.css_provider.clone().unwrap().0;
    let css_path = css_path
        .path()
        .expect("Error getting pathbuf for css provider");
    let mut pipe_path = css_path.to_path_buf().clone();
    pipe_path.add_extension(&"pipe");
        let mut j = 0;
        let mut fifo_path = ['\0' as i8; 2000];
        for (i, c) in pipe_path.to_str().unwrap().chars().enumerate() {
            fifo_path[i] = c as i8;
            j = i + 1;
        }
        fifo_path[j] = '\0' as i8;
        let path = fifo_path.clone();
        let p = fifo_path.as_ptr() as *const i8;
        unsafe {
            mkfifo(p, 0o666);
        }
        let launcher_cell_pipe = launcher_cell.clone();
        let open_pipe = move |flags| {
            let fd = unsafe { libc::open(path.as_ptr() as *const i8, flags) };
            if fd < 0 {
                panic!("failed to open pipe {:?}", &std::io::Error::last_os_error());
            }
            let buffer: [c_char; 20] = [0; 20];
            (fd, buffer)
        };
        let pipe_box = Box::new(open_pipe.clone());

        let thread_writer_wrapper = move || pipe_writer_thread(&css_path, pipe_box);

        let threadpool_result = match glib::ThreadPool::shared(Some(1)) {
            Err(..) => todo!("fix app crashing when unable to detect modifying css file"),
            Ok(threadpool) => threadpool.push(move || std::thread::spawn(thread_writer_wrapper)),
        };

        if !threadpool_result.is_ok() {
            println!("Failed to create thread_writer_wrapper");
            return;
        }

        let (fd, buffer) = open_pipe(O_RDONLY);
        glib::source::unix_fd_add_local(fd, glib::IOCondition::IN, move |_, _d| {
            let mut launcher = launcher_cell_pipe.borrow_mut();
            let bytes_read = unsafe { libc::read(fd, buffer.as_ptr() as *mut c_void, 20) };
            println!("bytes_read {:?}", bytes_read);
            if bytes_read == 0 {
                panic!("{:?}", &std::io::Error::last_os_error());
            }
            let contents = format!(
                "{:?}",
                String::from_utf8(buffer.to_vec().iter().map(|i| *i as u8).collect())
            );
            println!("contents: {}", contents);
            launcher.reload_css();
            glib::ControlFlow::Continue
        });
    
}
