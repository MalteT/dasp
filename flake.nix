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
    runBin = pkgs: name: bin: mode:
      pkgs.writeShellApplication {
        inherit name;
        text = ''
          ${pkgs.cargo}/bin/cargo run --profile ${mode} --bin ${bin} -- "$@"
        '';
      };

    pythonForTesting = pkgs:
      pkgs.python3.withPackages (ps: [
        ps.pip
        (
          ps.buildPythonPackage rec {
            pname = "clingo";
            version = "5.5.2";
            src = ps.fetchPypi {
              inherit pname version;
              sha256 = "sha256-ImxCPEUlO/D6GShF3R/EuvCwsfgqjou0uEgP48+Efkc=";
            };
            propagatedBuildInputs = [
              ps.setuptools
              ps.wheel
              ps.scikit-build
              pkgs.cmake
              pkgs.ninja
            ];
          }
        )
      ]);
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
            diff-so-fancy
            python3.pkgs.pylsp-mypy
            python3.pkgs.python-lsp-server
            inputs.dev.packages.x86_64-linux.mdpls
            (clingoFixed common.pkgs)
            # (pythonForTesting common.pkgs)
            (bench common.pkgs)
            (runBin common.pkgs "dasp-" "cli" "release")
            (runBin common.pkgs "dasp" "cli" "dev")
            (runBin common.pkgs "gen-" "af-generator" "release")
            (runBin common.pkgs "gen" "af-generator" "dev")
          ];
          env = [
            {
              name = "CLINGO_LIBRARY_PATH";
              value = "${clingoFixed common.pkgs}/lib";
            }
            {
              name = "LD_LIBRARY_PATH";
              value = "${common.pkgs.stdenv.cc.cc.lib}/lib";
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
