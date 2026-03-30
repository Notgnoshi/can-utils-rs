pub mod can;
pub mod frame;
pub mod recv;

#[cfg(test)]
#[ctor::ctor]
fn test_setup() {
    tracing_subscriber::fmt().with_test_writer().init();
    vcan_fixture::enter_namespace();
}
