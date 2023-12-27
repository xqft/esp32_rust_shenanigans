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
            in with pkgs;
            {
                devShells.default = mkShell {
                    buildInputs = [
                        rust-bin.stable.latest.default
                        pkgs.espup
                    ];
                };
            }
        );
}
