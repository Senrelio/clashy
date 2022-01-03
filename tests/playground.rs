use sysinfo::{SystemExt, ProcessExt};
fn main() {
    let mut system = sysinfo::System::default();
    system.refresh_processes();
    for p in system.process_by_name("clash") {
        println!("{}: {}", &p.pid(), &p.name());
    }
}
