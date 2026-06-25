// src/main.rs — EdgeDefender Cleaner (Rust Edition) v13.0
// Unified Windows Defender + Microsoft Edge removal tool

mod ui;
mod ops;
mod utils;
mod crypto;
mod updater;

use anyhow::Result;
use ops::Operation;
use utils::admin;

fn main() -> Result<()> {
    // Force UTF-8 output on Windows terminal
    let _ = std::process::Command::new("cmd")
        .args(["/c", "chcp", "65001"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Check for administrator privileges
    if !admin::is_elevated() {
        eprintln!();
        eprintln!("  [!] Bu uygulama Yönetici (Administrator) yetkileri gerektiriyor.");
        eprintln!("  [!] Lütfen uygulamayı 'Yönetici olarak çalıştır' ile başlatın.");
        eprintln!();
        admin::relaunch_as_admin()?;
        return Ok(());
    }

    // Ctrl+C graceful exit
    ctrlc::set_handler(|| {
        println!();
        println!("  [!] İşlem iptal edildi. Çıkılıyor...");
        std::process::exit(0);
    })
    .ok();

    ui::print_banner();

    // Run GitHub Update Check
    updater::check_for_updates();

    // CLI argument mode (automation)
    let args: Vec<String> = std::env::args().collect();
    let operation = if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "/all" | "--all"   => { println!("  → Otomasyon: Tüm bileşenler kaldırılıyor"); Operation::RemoveAll }
            "/r"  | "--r"  | "y" => { println!("  → Otomasyon: Defender tam kaldırma"); Operation::RemoveFull }
            "/a"  | "--a"  | "a" => { println!("  → Otomasyon: Sadece Antivirus"); Operation::RemoveAntivirusOnly }
            "/e"  | "--e"  | "e" => { println!("  → Otomasyon: Edge kaldırma"); Operation::RemoveEdge }
            "/s"  | "--s"  | "s" => { println!("  → Otomasyon: Dosya temizliği"); Operation::RemoveFiles }
            _ => ui::show_menu()?,
        }
    } else {
        ui::show_menu()?
    };

    ops::execute(operation)?;

    println!();
    println!("  {} İşlem tamamlandı. Log dosyası: defender_remover.log",
        colored::Colorize::bright_cyan("ℹ"));
    println!();

    Ok(())
}
