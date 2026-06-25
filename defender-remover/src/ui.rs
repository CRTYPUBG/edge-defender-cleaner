// src/ui.rs — EdgeDefender Cleaner — Terminal UI

use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use crate::ops::Operation;



/// Print the ASCII art banner for EdgeDefender Cleaner
pub fn print_banner() {
    print!("\x1B[2J\x1B[1;1H");  // clear screen

    let top    = format!("  \u{256c}{}\u{256c}", "\u{2550}".repeat(62));
    let bottom = format!("  \u{255a}{}\u{255d}", "\u{2550}".repeat(62));
    let mid    = format!("  \u{2551}{}\u{2551}", " ".repeat(62));
    let divider = format!("  \u{2551}  {}  \u{2551}", "\u{2500}".repeat(58));

    println!("{}", top.bright_red().bold());
    println!("{}", mid.bright_red().bold());
    println!(
        "  {}  {}  {}",
        "\u{2551}".bright_red().bold(),
        " \u{26a1}  EDGEDEFENDER CLEANER  \u{2014}  Rust Edition  v13.0          "
            .truecolor(255, 80, 80)
            .bold(),
        "\u{2551}".bright_red().bold()
    );
    println!(
        "  {}  {}  {}",
        "\u{2551}".bright_red().bold(),
        "    Windows Defender + Microsoft Edge Kald\u{0131}rma Arac\u{0131}        "
            .truecolor(200, 200, 200),
        "\u{2551}".bright_red().bold()
    );
    println!("{}", divider.bright_red().dimmed());
    println!(
        "  {}  {}  {}",
        "\u{2551}".bright_red().bold(),
        "    Yap\u{0131}mc\u{0131} : CRTY                                           "
            .truecolor(255, 200, 80),
        "\u{2551}".bright_red().bold()
    );
    println!(
        "  {}  {}  {}",
        "\u{2551}".bright_red().bold(),
        "    Defender : ionuttbara/windows-defender-remover        "
            .truecolor(150, 200, 255),
        "\u{2551}".bright_red().bold()
    );
    println!(
        "  {}  {}  {}",
        "\u{2551}".bright_red().bold(),
        "    Edge     : ShadowWhisperer/Remove-MS-Edge             "
            .truecolor(150, 200, 255),
        "\u{2551}".bright_red().bold()
    );
    println!("{}", mid.bright_red().bold());
    println!("{}", bottom.bright_red().bold());
    println!();
    println!(
        "  {} {}",
        "\u{26a0}".yellow().bold(),
        "İşlem öncesi sistem geri yükleme noktası oluşturmanız ÖNERİLİR."
            .yellow()
    );
    println!();
}

/// Show interactive 5-option menu
pub fn show_menu() -> Result<Operation> {
    let options = vec![
        "  ★  [TÜMÜ]   Defender + Edge + Security App → Hepsini Kaldır (ÖNERİLEN)",
        "  ●  [Y]      Defender Antivirus + Windows Security App Kaldır",
        "  ●  [A]      Sadece Defender Antivirus Kaldır (Security App kalır)",
        "  ◆  [E]      Microsoft Edge Kaldır (WebView2 KORUNUR — Edge Yeniden Yükleme Engellenir)",
        "  ◇  [S]      Defender Dosyalarını Temizle (Antivirus önceden kaldırıldıysa)",
    ];

    println!("  {}", "Kaldırma seçeneğini belirleyin:".bright_white().bold());
    println!();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact()?;

    println!();

    Ok(match selection {
        0 => Operation::RemoveAll,
        1 => Operation::RemoveFull,
        2 => Operation::RemoveAntivirusOnly,
        3 => Operation::RemoveEdge,
        4 => Operation::RemoveFiles,
        _ => unreachable!(),
    })
}


/// Print a section header
pub fn print_section(title: &str) {
    println!();
    println!("  {}", "━".repeat(62).bright_red().dimmed());
    println!(
        "  {}  {}",
        "▶".bright_red().bold(),
        title.bright_white().bold()
    );
    println!("  {}", "━".repeat(62).bright_red().dimmed());
}

/// Print reboot countdown notice
pub fn print_reboot_warning(seconds: u32) {
    println!();
    println!(
        "  {} {} saniye içinde sistem yeniden başlatılacak...",
        "⚡".bright_yellow(),
        seconds.to_string().bright_yellow().bold()
    );
    println!(
        "  {} İptal etmek: {}",
        "ℹ".cyan(),
        "shutdown /a".bright_cyan().bold()
    );
}

