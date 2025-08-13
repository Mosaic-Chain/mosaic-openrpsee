{
  description = "Rust Project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      pname = manifest.name;
      version = manifest.version;

      nativeBuildInputs = [ rustToolchain ];
      buildInputs = [ ];
    in
    {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        inherit pname version nativeBuildInputs buildInputs;

        meta = {
          name = pname;
          version = version;
          description = manifest.description;
          licence = manifest.license;
          authors = manifest.authors;
        };

        src = pkgs.nix-gitignore.gitignoreSource [] ( pkgs.lib.cleanSource ./. );
        cargoLock.lockFile = ./Cargo.lock;

        cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

        doCheck = true;
      };

      devShell = pkgs.mkShell {
        inherit buildInputs nativeBuildInputs;
        RUST_BACKTRACE=1;
        LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH";
      };
    });
}
