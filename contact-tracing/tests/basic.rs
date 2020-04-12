use contact_tracing::{Rpi, TracingKey};

#[test]
fn test_simple_broadcast() {
    let tkey = TracingKey::unique();
    let _rpi = Rpi::for_now(&tkey);
}
