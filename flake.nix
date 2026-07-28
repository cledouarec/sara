{
  description = "SARA - Solution Architecture Requirement for Alignment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      # Systems this flake provides outputs for. Windows is not supported by
      # Nix, and x86_64-darwin is no longer part of nixpkgs' Darwin platforms.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forEachSystem (pkgs: rec {
        default = sara;

        sara = pkgs.rustPlatform.buildRustPackage {
          pname = "sara";
          version = (nixpkgs.lib.importTOML ./Cargo.toml).workspace.package.version;

          src = self;

          cargoLock.lockFile = ./Cargo.lock;

          nativeCheckInputs = [ pkgs.git ];

          # This test asserts that the working directory is a Git repository.
          # Nix builds from a source copy that carries no `.git` directory, so
          # the assumption cannot hold here. Every other test is self-contained
          # and still runs.
          checkFlags = [ "--skip=repository::git::tests::test_is_git_repo" ];

          meta = {
            description = "Manage architecture documents and requirements as a knowledge graph";
            homepage = "https://github.com/cledouarec/sara";
            license = nixpkgs.lib.licenses.asl20;
            mainProgram = "sara";
          };
        };
      });

      devShells = forEachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            git
          ];
        };
      });
    };
}
