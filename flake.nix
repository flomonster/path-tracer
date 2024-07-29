{
  description = "A Nix flake for Path Tracer";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    alejandra = {
      url = "github:kamadorueda/alejandra/3.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
    alejandra,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };

        rustVer = fenix.packages.${system}.stable;
        rustChan = rustVer.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];
      in
        with pkgs; {
          devShells.default = mkShell {
            nativeBuildInputs = [
              rustChan
              mold-wrapped
          ];
            buildInputs =
              [
                # Tools
                cargo-watch
                taplo

                # Libs
                sfml
                csfml

                # Nix formatter
                alejandra.defaultPackage.${system}
              ];

            RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          };
        }
    );
}
