{
    description = "xqft's Rust & ESP32 shenanigans";

    inputs = {
        rust-overlay.url = "github:oxalica/rust-overlay";
        flake-utils.url = "github:numtide/flake-utils";
        nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    };

    outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ (import rust-overlay )];
                pkgs = import nixpkgs {
                    inherit system overlays;
                };
                rustToolchain = (
                    pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
                );
            in with pkgs;
            {
                devShells.default = mkShell {
                    nativeBuildInputs = [
                        #rustToolchain
                    ];

                    buildInputs = with pkgs; [
                        espup
                    ];

                    shellHook = ''
                        echo "Please run -espup install- if the esp-rs toolchain is not installed yet."
                        source $HOME/export-esp.sh
                    '';
                };
            }
        );
}
