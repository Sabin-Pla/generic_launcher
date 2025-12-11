#!/bin/bash
set -e

dbg=false
while [[ $# -gt 0 ]]; do
	case "$1" in
		--debug ) dbg=true; echo "Performing debug build";;
		--install-dir ) shift; INSTALL_DIR="$1";;
		* ) echo "invalid arg $1" >&2
	esac
	shift
done

if [[ -z "$INSTALL_DIR" ]]; then 
	export INSTALL_DIR="$(dirname "$0")"
fi

if [[ ! -d "$INSTALL_DIR" ]]; then 
	echo "Error setting installing dir" >&2
	exit 1
fi

INSTALL_DIR=$(realpath "$INSTALL_DIR")
echo "installing to $INSTALL_DIR"
project_dir=$(dirname "$0")
cd "$project_dir"
echo "performing build in: $(realpath "$project_dir")"

if [[ -z "$XDG_CACHE_HOME" ]]; then
	build_dir="$HOME/.config"
else
	build_dir="$XDG_CACHE_HOME"
fi
build_dir="$build_dir/cargo-targets/generic_launcher"
export CARGO_TARGET_DIR="$build_dir"

if [[ "$dbg" = "true" ]]; then 
	rustup run nightly cargo -Zbuild-std -Zbuild-std-features=debug_refcell build
	out_file="$build_dir/debug/generic_launcher"
else
	cargo build --release
	out_file="$build_dir/release/generic_launcher"
fi

rm -f "$INSTALL_DIR/generic_launcher"
set -x
if ! install -v "$out_file" "$INSTALL_DIR"; then 
	echo $?
fi

if [[ $(realpath "$project_dir") !=  $(realpath "$INSTALL_DIR") ]]; then
	cp -r "$project_dir/assets" "$INSTALL_DIR"
	cp "$project_dir/launcher.css" "$INSTALL_DIR"
fi

"$INSTALL_DIR/generic_launcher"