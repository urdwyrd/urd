/// Urd Language Server — binary entry point.
///
/// Launches the LSP server on stdin/stdout.

fn main() {
    let (connection, io_threads) = lsp_server::Connection::stdio();
    urd_lsp::run_server(connection);
    io_threads.join().unwrap();
}
