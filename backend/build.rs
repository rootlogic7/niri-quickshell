use std::process::Command;

fn main() {
    let proto_file = "../protocol/shell_state.fbs";

    // Sage Cargo: "Führe dieses Skript nur neu aus, wenn sich die .fbs Datei ändert!"
    println!("cargo:rerun-if-changed={}", proto_file);

    // Rufe flatc auf
    let status = Command::new("flatc")
        .args(["--rust", "-o", "src/", proto_file])
        .status()
        .expect("Konnte flatc nicht ausführen. Ist es installiert?");

    if !status.success() {
        panic!("flatc ist fehlgeschlagen!");
    }
}
