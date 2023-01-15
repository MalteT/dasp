# Argumentation Framework Generator

Generate argumentation frameworks for testing purposes. This is a simple generator,
selecting a subset of the complete graph configured by command line parameters. If requested,
any number of additional updates to the initial graph can be generated aswell.

## Usage

```text
Generate AFs and optional updates for the dynamic context

Usage: af-generator [OPTIONS] --output <PATH> --format <EXT>

Options:
  -n, --size <NUM>
          Size of the initial AF [default: 1000]
  -u, --updates <NUM>
          Number of updates to generate [default: 0]
  -o, --output <PATH>
          Output path to write to. The main file will be
          written to PATH-initial.EXT. The update file will be
          written to PATH-updates.EXTm
  -f, --format <EXT>
          Format for written files [possible values: apx, tgf]
  -p, --edge <FLOAT>
          Edge propability [default: 0.05]
      --update-edge <FLOAT>
          Probability by which attacks from and to every other
          argument should be selected when an argument-add
          update is created. If the argument `3` is added,
          consider every possible new attack and add it with
          this probability [default: 0.0025]
  -h, --help
          Print help information
```
