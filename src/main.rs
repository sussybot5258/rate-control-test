mod frame_control_v2;
mod frame_count;
use crate::frame_control_v2::frame_controler_2::frame_control_2_main;
use futures_time::{channel, task, time};
use std::thread;

const DRAW_SIGNAL: u8 = 0;

const DRAW_RATE: f64 = 1. / 144.;
// const DRAW_RATE: f64 = 1. / 60.;

// const FRAME_LIMIT: f64 = 1. / 60.;
const FRAME_LIMIT: f64 = 1. / 144.;

fn main() {
    frame_control_1_main()
    // frame_control_2_main();
}

fn frame_control_1_main() {
    // 2 channels for signal communication between `main thread` and `frame control thread`
    // ( Is there better way? )
    let (sub_sender, main_receiver) = channel::bounded::<u8>(1);
    let (main_sender, sub_receiver) = channel::bounded::<u8>(1);

    // frame control thread
    thread::spawn(frame_control(sub_sender, sub_receiver));

    // main thread
    futures::executor::block_on(async {
        // send start message
        if let Err(e) = main_sender.send(0).await {
            panic!("[MAIN:FATAL] start signal failed: {}", e);
        }

        println!("[MAIN] waiting for msg...");

        // await until frame controler signaled
        while main_receiver.recv().await.is_ok() {
            // use `sleep` to simulate drawing
            task::sleep(time::Duration::from_secs_f64(DRAW_RATE)).await;

            // send draw done signal
            if let Err(t) = main_sender.send(DRAW_SIGNAL).await {
                panic!("[MAIN:FATAL] send done signal failed: {:?}", t);
            }
        }

        panic!("[MAIN:FATAL] channel error");
    });
}

fn frame_control(sender: channel::Sender<u8>, receiver: channel::Receiver<u8>) -> impl FnOnce() {
    move || {
        futures::executor::block_on(async move {
            // frame control using `sleep`
            let task_timer = || async {
                // task::sleep(time::Duration::from_secs_f64(DRAW_RATE)).await;
                task::sleep(time::Duration::from_secs_f64(FRAME_LIMIT)).await;
            };

            // send draw signal and wait until draw complete
            let task_drawer = || async {
                if let Err(e) = sender.send(DRAW_SIGNAL).await {
                    panic!("[SUB:FATAL] error in sending start draw signal: {}", e)
                }

                if let Err(e) = receiver.recv().await {
                    panic!("[SUB:FATAL] error in receiving draw done signal: {}", e)
                };
            };

            // frame count
            let mut frame_counter = frame_count::FrameCounter::new();

            // wait until main thread says start
            if let Err(e) = receiver.recv().await {
                panic!("[SUB:FATAL] error in receiving start signal: {}", e)
            };

            frame_counter.reset();
            loop {
                // wait until both task is done
                futures::join!(task_timer(), task_drawer());

                // count the frame
                frame_counter.count();
            }
        })
    }
}
