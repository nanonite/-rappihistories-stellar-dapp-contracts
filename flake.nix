{
  description = "Medichain Soroban contract toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { nixpkgs
    , rust-overlay
    , ...
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsFor = system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

      contractShellFor = system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.stable.latest.minimal.override {
            extensions = [
              "clippy"
              "rust-src"
              "rustfmt"
            ];
            targets = [ "wasm32-unknown-unknown" ];
          };
        in
        pkgs.mkShell {
          name = "medichain-contracts";

          packages = with pkgs; [
            bashInteractive
            binaryen
            cacert
            clang
            git
            gnumake
            lld
            pkg-config
            rustToolchain
          ];

          RUST_BACKTRACE = "1";

          shellHook = ''
            echo "Medichain contracts: Cargo $(cargo --version | cut -d' ' -f2), wasm32-unknown-unknown target ready"
          '';
        };
    in
    {
      devShells = forAllSystems
        (system: {
          default = contractShellFor system;
          ci = contractShellFor system;
        });

      checks = forAllSystems
        (system:
          let
            pkgs = pkgsFor system;
            rustToolchain = pkgs.rust-bin.stable.latest.minimal.override {
              targets = [ "wasm32-unknown-unknown" ];
            };
          in
          {
            contract-toolchain-smoke = pkgs.runCommand "medichain-contract-toolchain-smoke"
              {
                nativeBuildInputs = [
                  pkgs.binaryen
                  rustToolchain
                ];
              } ''
              cargo --version
              rustc --print target-list | grep '^wasm32-unknown-unknown$'

              sysroot="$(rustc --print sysroot)"
              test -d "$sysroot/lib/rustlib/wasm32-unknown-unknown/lib"

              wasm-opt --version

              mkdir -p "$out"
            '';
          });

      formatter = forAllSystems (system: (pkgsFor system).nixpkgs-fmt);
    };
}
