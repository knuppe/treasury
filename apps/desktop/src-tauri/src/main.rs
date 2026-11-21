// Sem janela de console no Windows em release (o Treasury mira macOS e Linux, mas não custa).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Release: sem core dump e, no Linux, memória fora do alcance de outros processos.
    treasury_core::vault::harden_process();
    // A versão do app é a deste crate; o núcleo tem a dele fixa (ver config::app_version).
    treasury_core::config::set_app_version(env!("CARGO_PKG_VERSION"));
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        // `treasury mcp`: servidor MCP via stdio para clientes externos (Claude Desktop/Code).
        Some("mcp") => std::process::exit(treasury_desktop::run_mcp()),
        Some("--version") | Some("-V") => println!("treasury {}", env!("CARGO_PKG_VERSION")),
        Some("--help") | Some("-h") => {
            println!(
                "treasury {}\n\nUso:\n  treasury          abre o app\n  treasury mcp      servidor MCP via stdio (somente leitura)",
                env!("CARGO_PKG_VERSION")
            )
        }
        _ => treasury_desktop::run(),
    }
}
