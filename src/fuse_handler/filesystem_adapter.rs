pub trait FileSystemStrategy {
    fn init(&self) -> Result<(), libc::c_int>;
}
