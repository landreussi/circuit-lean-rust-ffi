{
  description = "Rust development environment with stable toolchain via rustup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      demoScript = pkgs.writeShellScriptBin "demo" ''
        (
          cd lean
          lake build ArithCircuit:static
        )

        (
          cd arith-circuit-rs
          cargo test -- --test-threads=1
          cargo run --example demo
        )
      '';
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          clang
          pkg-config
          elan
          lean4
          libuv
          cargo
          rustfmt
        ];
      };

      apps.demo = {
        type = "app";
        program = "${demoScript}/bin/demo";
      };

      apps.default = self.apps.${system}.demo;
    });
}
