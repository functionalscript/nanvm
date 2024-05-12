use io_impl::RealIo;
use nanvm_lib::app::run;

fn main() {
    let _ = run(&RealIo::default());
}
