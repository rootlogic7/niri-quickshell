use std::process::Command;

fn main() {
    let proto_file = "../protocol/shell_state.fbs";

    // Sage Cargo: "Führe dieses Skript nur neu aus, wenn sich die .fbs Datei ändert!"
    println!("cargo:rerun-if-changed={}", proto_file);

    // Rufe flatc auf
    let status = Command::new("flatc")
        .args(&[
            "--rust",
            "-o",
            "src/",
            "../protocol/shell_state.fbs",
            "../protocol/client_command.fbs", // NEU hinzugefügt!
        ])
        .status()
        .expect("Fehler beim Ausführen von flatc");

    if !status.success() {
        panic!("flatc ist fehlgeschlagen!");
    }
}
