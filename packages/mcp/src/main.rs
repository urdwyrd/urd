/// Urd MCP Server — binary entry point.
///
/// Usage: urd-mcp <file.urd.md>
///
/// Compiles the given file, then serves read-only MCP tools on stdin/stdout.

use std::env;

use rmcp::ServiceExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: urd-mcp <file.urd.md>");
        std::process::exit(1);
    }
    let file_path = &args[1];

    // Compile the world
    let result = urd_compiler::compile(file_path);

    // Build immutable query state
    let world_data = urd_mcp::world_data::WorldData::from_result(result);

    // Create service and serve on stdio
    let service = urd_mcp::service::UrdMcpService::new(world_data);
    let server = service.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;

    Ok(())
}
