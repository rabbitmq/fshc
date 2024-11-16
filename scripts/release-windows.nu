#!/usr/bin/env nu

let binary = 'fshc'
let src = $env.SRC | path expand
let target = $env.TARGET

let version = (open Cargo.toml | get package.version)
let release_dir = $'($env.SRC)/target/($target)/release' | path expand
let executable = $'($release_dir)/($binary).exe'

print $'Packaging ($binary) v($version) for ($target) in ($src)...'
print $'Executable path is ($executable)...'

if not ('Cargo.lock' | path exists) {
  cargo generate-lockfile
}

rm -rf $release_dir
mkdir $release_dir

print $'Building on Windows in ($src)...'
build-with-cargo

#
# Windows
#

print "Building on Windows..."
cargo rustc --bin $binary --target $target --target-dir $env.GITHUB_WORKSPACE --release

#
# Release packaging
#

cd $src

print $"Release directory: ($release_dir)"
ls $release_dir | print

cp -r LICENSE* $release_dir
cp -r README* $release_dir

cd $release_dir
ls $release_dir

let archive_filename = $'($binary)-($version)-($target).zip'
print $'Release archive name: ($archive_filename)'
7z a $archive_filename $binary
print $'Release archive at ($archive_filename)';
let pkg = (ls -f $archive_filename | get name)
if not ($pkg | empty?) {
  echo $'archive=($pkg | get 0)' | save --append $env.GITHUB_OUTPUT
}

def 'build-with-cargo' [] {
  cargo rustc --bin $binary --target $target --release
}