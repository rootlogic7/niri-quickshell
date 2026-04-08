{
  description = "Niri Quickshell - High Performance Wayland Shell";

  inputs = {
    # Wir zielen auf den aktuellen Channel (für NixOS 26.05 passend)
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; 
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      # =========================================================
      # DAS ECHTE SYSTEM-PAKET FÜR NIXOS
      # =========================================================
      packages.${system}.default = pkgs.stdenv.mkDerivation {
        pname = "niri-quickshell";
        version = "0.1.0";

        # Nimm das komplette aktuelle Git-Verzeichnis als Quelle
        src = ./.;

        cmakeDir = "../frontend";

        cargoRoot = "backend";

        # RUST OFFLINE-ZUGRIFF: Nix liest die Lock-Datei und lädt alle Crates vorab!
        cargoDeps = pkgs.rustPlatform.importCargoLock {
          lockFile = ./backend/Cargo.lock;
        };

        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          ninja
          flatbuffers
          corrosion                   # Nix bringt Corrosion direkt mit!
          rustPlatform.cargoSetupHook # Sagt CMake, wo die Offline-Crates liegen
          cargo
          rustc
          qt6.wrapQtAppsHook          # MAGIE: Setzt automatisch alle QT_WAYLAND Pfade für das Backend/Skript
        ];

        buildInputs = with pkgs; [
          qt6.qtbase
          qt6.qtdeclarative
          qt6.qtwayland
          wayland
          quickshell
        ];
      };

      # =========================================================
      # DEINE ENTWICKLUNGSUMGEBUNG (Bleibt wie sie ist)
      # =========================================================
      devShells.${system}.default = pkgs.mkShell {
        # Build-Tools und Compiler
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          ninja
          flatbuffers
          git
          
          # Rust Toolchain
          cargo
          rustc
          rustfmt
          clippy
        ];

        # Bibliotheken, gegen die wir linken müssen
        buildInputs = with pkgs; [
          qt6.qtbase
          qt6.qtdeclarative
          qt6.qtwayland
          wayland
          quickshell
        ];

        # Umgebungsvariablen, damit Qt und pkg-config sich in der devShell finden
        QT_QPA_PLATFORM = "wayland";
        PKG_CONFIG_PATH = "${pkgs.qt6.qtbase.dev}/lib/pkgconfig";

        shellHook = ''
          echo "================================================="
          echo "🚀 Niri-Quickshell Dev Environment (April 2026)"
          echo "================================================="
          echo "Rust: $(cargo --version)"
          echo "FlatBuffers: $(flatc --version)"
          echo "Qt6 bereit für Quickshell-C++-Plugins."
          echo "================================================="
        '';
      };
    };
}
