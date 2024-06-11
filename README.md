This is a test for rate control.

main thread do draw tasks, create another thread to control draw rate.  
2 threads using 2 channels to communicate with each other.

main loop wait for `draw start` signal send by `frame_control` and send back `draw done` signal to `frame_control`,
`frame_control` awaits for `rate limit(sleep)` and `draw done` signal then start the next loop.

## Something idk

seems that there's some problem with my async design or frame count logic:

```
# set rate to 60 per second
1s ◉ cargo r
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/future_test`
[MAIN] waiting for msg...
frames in one sec: 58
frames in one sec: 58
frames in one sec: 58
frames in one sec: 58
frames in one sec: 58
frames in one sec: 58
```

```
# set rate to 144 per second
45s ◉ cargo r
   Compiling future_test v0.1.0 (/home/ogios/work/rust/study/future_test)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
     Running `target/debug/future_test`
[MAIN] waiting for msg...
frames in one sec: 135
frames in one sec: 136
frames in one sec: 135
frames in one sec: 136
frames in one sec: 136
frames in one sec: 135
```

frames lost too much

## Another way

see [frame_control_v2](./src/frame_control_v2/mod.rs)  
and change the call in `fn main` in `main.rs`.
