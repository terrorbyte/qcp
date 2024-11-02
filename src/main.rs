//! qcp utility - main entrypoint
// (c) 2024 Ross Younger

#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> anyhow::Result<std::process::ExitCode> {
    qcp::cli()
}
