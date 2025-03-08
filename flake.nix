{
  description = "my project description";

  inputs = {

    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    leptos.url = "github:leptos-rs/leptos";

  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, leptos }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          devShells.default = with pkgs; mkShell {
            buildInputs = [
              rustfmt
              rust-analyzer
              trunk
(
  cargo-leptos # .overrideAttrs  (oldAttrs: rec {
  #  src = leptos;
  #  cargoDeps = oldAttrs.cargoDeps.overrideAttrs (oldAttrs: {
  #    name = "cargo-leptos-vendor";
  #    inherit src;
  #    outputHash = pkgs.lib.fakeSha256;
  #  });
  #}))
)
sass

              (rust-bin.nightly.latest.default.override {
                extensions = [ "rust-src" ];
                targets = [
                  "wasm32-unknown-unknown"
                ];
              })
            ];
          };
        }
      );
}

