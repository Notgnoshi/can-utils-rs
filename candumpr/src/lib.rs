pub mod can;
pub mod format;
pub mod frame;
pub mod recv;
pub mod write;

#[cfg(test)]
#[ctor::ctor]
fn test_setup() {
    tracing_subscriber::fmt().with_test_writer().init();
    vcan_fixture::enter_namespace();
}
