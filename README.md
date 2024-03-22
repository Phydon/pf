# pf

**P**eak **F**ile

Show the first/last few lines of a filepath:
- given as an argument, or
- given via pipe as the output of a previous command

Only works with one filepath at a time.


## Examples

todo!()

## Usage

### Short Usage

```
Usage: pf [OPTIONS] [PATH] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]  The filepath to work with

Options:
  -l, --last          Show last n lines
  -n, --num <NUMBER>  Number of lines to show
  -h, --help          Print help (see more with '--help')
  -V, --version       Print version
```

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/pf/releases)
