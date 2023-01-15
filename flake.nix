{
  inputs = {
    nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration";
    dev.url = "git+https://git.sr.ht/~megamanmalte/dev";
  };

  outputs = inputs: let
    clingoFixed = pkgs:
      pkgs.clingo.overrideAttrs (old: {
        version = "5.5.0";
        src = pkgs.fetchFromGitHub {
          owner = "potassco";
          repo = "clingo";
          rev = "v5.5.0";
          sha256 = "sha256-6xKtNi5IprjaFNadfk8kKjKzuPRanUjycLWCytnk0mU=";
        };
      });
    bench = pkgs:
      pkgs.writeShellApplication {
        name = "bench";
        runtimeInputs = [pkgs.gnuplot pkgs.cargo];
        text = ''
          cargo bench -- --plotting-backend gnuplot --sample-size 10 "$@"
        '';
      };
    # TODO: Implement
    generate-test-frameworks = pkgs:
      pkgs.writeShellApplication {
        name = "generate-test-frameworks";
        runtimeInputs = [inputs.self.x86_64-linux.af-generator];
        text = ''

        '';
      };
  in
    (inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      # TODO: Potentially support others?
      systems = ["x86_64-linux" "i686-linux"];
      config = common: {
        shell = {
          packages = with common.pkgs; [
            rust-analyzer
            lldb
            treefmt
            cargo-watch
            cargo-workspaces
            cargo-flamegraph
            hyperfine
            nil
            inputs.dev.packages.x86_64-linux.mdpls
            (clingoFixed common.pkgs)
            (bench common.pkgs)
          ];
          env = [
            {
              name = "CLINGO_LIBRARY_PATH";
              value = "${clingoFixed common.pkgs}/lib";
            }
          ];
        };
        runtimeLibs = [
          (clingoFixed common.pkgs)
        ];
      };
      pkgConfig = common: {
        cli.depsOverrides.fixClingoSysBuild = {
          CLINGO_LIBRARY_PATH = "${clingoFixed common.pkgs}/lib";
        };
        cli.overrides.fixClingoLinking = {
          buildInputs = [(clingoFixed common.pkgs)];
        };
        lib.depsOverrides.fixClingoSysBuild = {
          CLINGO_LIBRARY_PATH = "${clingoFixed common.pkgs}/lib";
        };
        lib.overrides.fixClingoLinking = {
          buildInputs = [(clingoFixed common.pkgs)];
        };
      };
    })
    // {
      # Let hydra build a few of these
      hydraJobs = let
        shells = builtins.mapAttrs (name: value: value.default) inputs.self.devShells;
      in {
        inherit (inputs.self) packages;
        inherit shells;
      };
    };
}
