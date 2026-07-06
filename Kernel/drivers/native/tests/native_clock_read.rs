use arcrtc_driver_native::native_clock_unix_epoch_millis_now;

#[test]
fn native_clock_reads_epoch_milliseconds() {
    let first = native_clock_unix_epoch_millis_now();
    let second = native_clock_unix_epoch_millis_now();
    println!("first={first:?} second={second:?}");
    assert!(first.expect("first clock read must succeed") > 0);
    assert!(second.expect("second clock read must succeed") > 0);
}
