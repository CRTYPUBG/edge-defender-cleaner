// src/ops/mod.rs — EdgeDefender Cleaner — unified operation dispatcher

pub mod registry;
pub mod services;
pub mod files;
pub mod appx;
pub mod edge;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::ui;
use crate::utils::logger;

/// All available operations in the unified cleaner
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Full combo: Defender + Security App + Edge
    RemoveAll,
    /// Defender antivirus + Windows Security App (no Edge)
    RemoveFull,
    /// Defender antivirus engine only (keeps Security App)
    RemoveAntivirusOnly,
    /// Microsoft Edge + WebView2 only
    RemoveEdge,
    /// Remove leftover Defender files from disk
    RemoveFiles,
}

/// Execute the selected operation
pub fn execute(op: Operation) -> Result<()> {
    match op {
        Operation::RemoveAll          => execute_all(),
        Operation::RemoveFull         => execute_full(),
        Operation::RemoveAntivirusOnly => execute_antivirus_only(),
        Operation::RemoveEdge         => execute_edge_only(),
        Operation::RemoveFiles        => execute_file_removal(),
    }
}

// ─── Spinner helpers ──────────────────────────────────────────────────────────

fn make_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn finish_ok(pb: &ProgressBar, msg: &str) {
    pb.set_style(
        ProgressStyle::with_template("  {prefix} {msg}")
            .unwrap(),
    );
    pb.set_prefix("✓");
    pb.finish_with_message(
        colored::Colorize::green(msg).to_string()
    );
}

fn finish_warn(pb: &ProgressBar, msg: &str) {
    pb.set_style(
        ProgressStyle::with_template("  {prefix} {msg}")
            .unwrap(),
    );
    pb.set_prefix("⚠");
    pb.finish_with_message(
        colored::Colorize::yellow(msg).to_string()
    );
}

// ─── All-in-One ───────────────────────────────────────────────────────────────

fn execute_all() -> Result<()> {
    ui::print_section("TÜM BİLEŞENLER KALDIRILIYOR (Defender + Edge)");

    let pb = make_spinner("Defender servisleri durduruluyor...");
    match services::stop_defender_services() {
        Ok(_)  => finish_ok(&pb, "Defender servisleri durduruldu"),
        Err(e) => finish_warn(&pb, &format!("Devam ediliyor: {e}")),
    }

    let pb = make_spinner("Defender Antivirus kayıt defteri uygulanıyor...");
    let crty_path = find_file("res.crty");
    match registry::apply_reg_archive(&crty_path, "Remove_Defender") {
        Ok(n)  => finish_ok(&pb, &format!("{n} REG dosyası uygulandı")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Güvenlik bileşenleri kayıt defteri uygulanıyor...");
    match registry::apply_reg_archive(&crty_path, "Remove_SecurityComp") {
        Ok(n)  => finish_ok(&pb, &format!("{n} REG dosyası uygulandı")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Windows Security UWP App (SecHealthUI) kaldırılıyor...");
    match appx::remove_sec_health_ui() {
        Ok(_)  => finish_ok(&pb, "SecHealthUI kaldırıldı"),
        Err(e) => finish_warn(&pb, &format!("Devam ediliyor: {e}")),
    }

    let pb = make_spinner("Defender zamanlanmış görevleri devre dışı bırakılıyor...");
    match services::disable_defender_tasks() {
        Ok(_)  => finish_ok(&pb, "Zamanlanmış görevler devre dışı"),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Microsoft Edge kaldırılıyor...");
    match edge::remove_edge() {
        Ok(_)  => finish_ok(&pb, "Microsoft Edge kaldırıldı"),
        Err(e) => finish_warn(&pb, &format!("Edge kısmen kaldırıldı: {e}")),
    }

    let pb = make_spinner("Defender dosyaları temizleniyor...");
    match files::remove_defender_files() {
        Ok(n)  => finish_ok(&pb, &format!("{n} konum temizlendi")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Yeniden başlatma planlanıyor...");
    pb.finish_and_clear();
    ui::print_reboot_warning(15);
    logger::log_ok("Tüm bileşenler kaldırıldı — reboot zamanlandı");
    schedule_reboot(15)
}

// ─── Full Defender Removal ────────────────────────────────────────────────────

fn execute_full() -> Result<()> {
    ui::print_section("DEFENDER TAM KALDIRMA");

    let pb = make_spinner("Defender servisleri durduruluyor...");
    match services::stop_defender_services() {
        Ok(_)  => finish_ok(&pb, "Defender servisleri durduruldu"),
        Err(e) => finish_warn(&pb, &format!("Devam ediliyor: {e}")),
    }

    let pb = make_spinner("Defender Antivirus kayıt defteri uygulanıyor...");
    let crty_path = find_file("res.crty");
    match registry::apply_reg_archive(&crty_path, "Remove_Defender") {
        Ok(n)  => finish_ok(&pb, &format!("{n} REG dosyası uygulandı")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Güvenlik bileşenleri kayıt defteri uygulanıyor...");
    match registry::apply_reg_archive(&crty_path, "Remove_SecurityComp") {
        Ok(n)  => finish_ok(&pb, &format!("{n} REG dosyası uygulandı")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Windows Security UWP App kaldırılıyor...");
    match appx::remove_sec_health_ui() {
        Ok(_)  => finish_ok(&pb, "SecHealthUI kaldırıldı"),
        Err(e) => finish_warn(&pb, &format!("Devam ediliyor: {e}")),
    }

    let pb = make_spinner("Zamanlanmış görevler devre dışı bırakılıyor...");
    match services::disable_defender_tasks() {
        Ok(_)  => finish_ok(&pb, "Görevler devre dışı"),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Yeniden başlatma planlanıyor...");
    pb.finish_and_clear();
    ui::print_reboot_warning(10);
    logger::log_ok("Defender tam kaldırma tamamlandı");
    schedule_reboot(10)
}

// ─── Antivirus Only ────────────────────────────────────────────────────────────

fn execute_antivirus_only() -> Result<()> {
    ui::print_section("DEFENDER ANTİVİRÜS KALDIRMA");

    let pb = make_spinner("Defender servisleri durduruluyor...");
    match services::stop_defender_services() {
        Ok(_)  => finish_ok(&pb, "Defender servisleri durduruldu"),
        Err(e) => finish_warn(&pb, &format!("Devam ediliyor: {e}")),
    }

    let pb = make_spinner("Defender Antivirus kayıt defteri uygulanıyor...");
    let crty_path = find_file("res.crty");
    match registry::apply_reg_archive(&crty_path, "Remove_Defender") {
        Ok(n)  => finish_ok(&pb, &format!("{n} REG dosyası uygulandı")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }

    let pb = make_spinner("Yeniden başlatma planlanıyor...");
    pb.finish_and_clear();
    ui::print_reboot_warning(10);
    logger::log_ok("Antivirus kaldırma tamamlandı");
    schedule_reboot(10)
}

// ─── Edge Only ─────────────────────────────────────────────────────────────────

fn execute_edge_only() -> Result<()> {
    ui::print_section("MICROSOFT EDGE KALDIRMA");

    let pb = make_spinner("Microsoft Edge ve tüm bileşenleri kaldırılıyor...");
    match edge::remove_edge() {
        Ok(_)  => finish_ok(&pb, "Microsoft Edge tamamen kaldırıldı"),
        Err(e) => {
            finish_warn(&pb, &format!("Hata: {e}"));
        }
    }
    logger::log_ok("Edge kaldırma tamamlandı");

    println!();
    println!(
        "  {} Edge kaldırma tamamlandı. Değişiklikler için yeniden başlatın.",
        colored::Colorize::bright_green("✓")
    );
    Ok(())
}

// ─── File Removal ──────────────────────────────────────────────────────────────

fn execute_file_removal() -> Result<()> {
    ui::print_section("DEFENDER DOSYA TEMİZLİĞİ");

    let pb = make_spinner("Defender klasörleri siliniyor...");
    match files::remove_defender_files() {
        Ok(n)  => finish_ok(&pb, &format!("{n} konum temizlendi")),
        Err(e) => finish_warn(&pb, &format!("{e}")),
    }
    logger::log_ok("Defender dosyaları temizlendi");

    println!();
    println!("  {} Dosya temizliği tamamlandı.", colored::Colorize::bright_green("✓"));
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Find a project sub-directory or file relative to the binary
pub fn find_file(name: &str) -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().unwrap_or(&exe).to_path_buf();
        let candidate = dir.join(name);
        if candidate.exists() {
            return candidate;
        }
        for _ in 0..5 {
            dir = dir.parent().unwrap_or(&dir).to_path_buf();
            let candidate = dir.join(name);
            if candidate.exists() {
                return candidate;
            }
        }
    }
    std::path::PathBuf::from(name)
}

/// Schedule Windows reboot
fn schedule_reboot(seconds: u32) -> Result<()> {
    let status = std::process::Command::new("shutdown")
        .args(["/r", "/f", "/t", &seconds.to_string()])
        .status()?;
    if !status.success() {
        anyhow::bail!("shutdown komutu başarısız");
    }
    Ok(())
}
