use gloo_worker::Registrable;
use rewwise_worker::ParseWorker;

fn main() {
    ParseWorker::registrar().register();
}
