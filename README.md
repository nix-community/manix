# Manix

A fast CLI documentation searcher for Nix.

## Supported sources

- Nixpkgs Documentation
- Nixpkgs Comments
- Nixpkgs Tree (pkgs, pkgs.lib)
- NixOS Options
- Nix-Darwin Options
- Home-Manager Options

## Usage

```sh
manix --help
manix mergeattr
manix --strict mergeattr
manix --update-cache mergeattr
```

### rnix-lsp

If you want to use it in your editor, check [ElKowar's rnix-lsp fork](https://github.com/elkowar/rnix-lsp), which uses it to provide documentation on hover and autocompletion.

![manix](/manix.png)

### fzf

You can use manix with fzf via this command:

```sh
manix "" | sed -n 's/^# \(.*\) \?.*/\1/p' | fzf --preview="manix '{}'" | xargs manix
```

Or, alternatively, without the final output if preview is enough:

```sh
manix "" | sed -n 's/^# \(.*\) \?.*/\1/p' | fzf --preview="manix '{}'"
```
Alternatively, you can use the following script by adding it to your Home Manager configuration:
```nix
(pkgs.callPackage PATH/TO/SCRIPT/SCRIPT.nix {})
```

This script caches all options for 10 days (you can adjust this value inside the script) or forces a refresh using the `-r` flag. Compared to the previous manix command, it trades a little disk space for significant time savings during searches.

Its dependencies are `manix`, `ripgrep`, and `fzf`.

```nix
{ writeShellApplication, manix, ripgrep, fzf, ... }:

let
  cacheTime = "10";
in

writeShellApplication {
  name = "custom-manix";
  runtimeInputs = [
    manix
    ripgrep
    fzf
  ];
  text = ''
CACHE_DIR="''${XDG_CACHE_HOME:-$HOME/.cache}"
CACHE_FILE="$CACHE_DIR/custom-manix-options.txt"
mkdir -p "$CACHE_DIR"

REGENERATE=0

# Check if the -r flag was passed
if [ "''${1:-}" = "-r" ]; then
  REGENERATE=1
fi

# Regenerate cache if it doesn't exist or is older than cacheTime days
if [ ! -f "$CACHE_FILE" ] || [ "$(find "$CACHE_FILE" -mtime +${cacheTime} -print)" ]; then
  REGENERATE=1
fi

if [ $REGENERATE -eq 1 ]; then
  echo "(Re)building custom-manix cache..."
  mkdir -p "$(dirname "$CACHE_FILE")"
  manix "" \
    | rg '^(?:\x1b\[[0-9;]*m)*# ' \
    | sed 's/\x1b\[[0-9;]*m//g; s/^# //' \
    | rg -v '^(<|.*https?://)' \
    > "$CACHE_FILE"
fi

# Read from cache and pipe into fzf
cat "$CACHE_FILE" \
  | fzf --preview "manix {}" \
  | xargs manix
  '';
}
```

## Installation

### Update

Manix is now available in nixpkgs. If you use the unstable channel installing is as easy as adding `manix` to your `environment.systemPackages` or `home.packages`.

### Github Releases

Since it can take some time to compile Manix, you can download statically-built executables from Github Releases.

```sh
wget https://github.com/nix-community/manix/releases/latest/download/manix
chmod +x manix
mv manix ~/bin/ # or some other location in your $PATH
```

### nix-env

```sh
# If you have the unstable channel on your system
nix-env -f '<unstable>' -iA manix
# OR
nix-env -i -f https://github.com/nix-community/manix/archive/master.tar.gz
# OR
nix profile install github:nix-community/manix/latest
```

### Nix with flakes enabled

``` sh
nix run 'github:nix-community/manix' mapAttrs
```

## Kudos

The original [manix](https://github.com/mlvzk/manix). mlvzk has been inactive for over a year, we thank him for his hard work.
The inspiration for this project came from [nix-doc](https://github.com/lf-/nix-doc)
