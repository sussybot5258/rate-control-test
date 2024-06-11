use crate::frame_count;
use futures_time::channel;
use std::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread,
};

const DRAW_SIGNAL: u8 = 0;

const DRAW_RATE: f64 = 1. / 144.;
// const DRAW_RATE: f64 = 1. / 60.;

// const FRAME_LIMIT: f64 = 1. / 60.;
const FRAME_LIMIT: f64 = 1. / 144.;

struct TaskCounter {
    count: u8,
    threshold: u8,
    task: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl TaskCounter {
    fn new(threshold: u8) -> Self {
        TaskCounter {
            count: 0,
            task: None,
            threshold: threshold - 1,
        }
    }
    fn done(&mut self) {
        if self.count == self.threshold {
            self.count = 0;
            if let Some(task) = self.task.take() {
                task();
            } else {
                panic!("FnOnce() task has been taken or never existed");
            }
        } else {
            self.count += 1
        }
    }
}

// this is a recursive method
fn frame_control(
    sender: channel::Sender<u8>,
    receiver: channel::Receiver<u8>,
    frame_count: Arc<Mutex<frame_count::FrameCounter>>,
) {
    // task counter for both `timer thread` and `draw task thread`
    let taskc = Arc::new(Mutex::new(TaskCounter::new(2)));
    let taskc_timer = taskc.clone();

    // clone and move for next recursion
    let sender_clone = sender.clone();
    let receiver_clone = receiver.clone();
    let frame_count_clone = frame_count.clone();
    taskc.lock().unwrap().task = Some(Box::new(move || {
        frame_control(sender_clone, receiver_clone, frame_count_clone)
    }));

    // timer thread
    thread::spawn(move || {
        thread::sleep(Duration::from_secs_f64(FRAME_LIMIT));
        taskc_timer.lock().unwrap().done();
    });

    // draw task thread
    thread::spawn(move || {
        if let Err(e) = sender.send_blocking(DRAW_SIGNAL) {
            panic!("[SUB:FATAL] error in sending start draw signal: {}", e)
        }

        if let Err(e) = receiver.recv_blocking() {
            panic!("[SUB:FATAL] error in receiving draw done signal: {}", e)
        };
        frame_count.lock().unwrap().count();
        taskc.lock().unwrap().done();
    });
}

pub fn frame_control_2_main() {
    let (sub_sender, main_receiver) = channel::bounded::<u8>(1);
    let (main_sender, sub_receiver) = channel::bounded::<u8>(1);

    // frame control
    let frame_counter = Arc::new(Mutex::new(frame_count::FrameCounter::new()));
    frame_control(sub_sender, sub_receiver, frame_counter);

    futures::executor::block_on(async {
        // main thread
        println!("[MAIN] waiting for msg...");

        // await until frame controler signaled
        while main_receiver.recv().await.is_ok() {
            // use `sleep` to simulate drawing
            thread::sleep(Duration::from_secs_f64(DRAW_RATE));

            // send draw done signal
            if let Err(t) = main_sender.send(DRAW_SIGNAL).await {
                panic!("[MAIN:FATAL] send done signal failed: {:?}", t);
            }
        }

        panic!("[MAIN:FATAL] channel error");
    })
}
