#!/usr/bin/env nu

let binary = 'fshc'
let src = $env.SRC | path expand
let os = $env.OS
let target = $env.TARGET

let version = (open Cargo.toml | get package.version)
let release_dir = $'($env.SRC)/target/($target)/release' | path expand
let executable = $'($release_dir)/($binary)'

print $'Packaging ($binary) v($version) for ($target) in ($src)...'
print $'Executable path is ($executable)...'

if not ('Cargo.lock' | path exists) {
  cargo generate-lockfile
}

rm -rf $release_dir
mkdir $release_dir

#
# Linux
#

def is-musl-target [] {
  $target | str ends-with '-musl'
}

if ($os | str starts-with 'ubuntu') {
  print $"Building on Ubuntu for ($target)..."

  if $target == 'aarch64-unknown-linux-gnu' {
    sudo apt-get install -y gcc-aarch64-linux-gnu
    $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    build-with-cargo
  } else if $target == 'armv7-unknown-linux-gnueabihf' {
    sudo apt-get install pkg-config gcc-arm-linux-gnueabihf -y
    $env.CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = 'arm-linux-gnueabihf-gcc'
    build-with-cargo
  } else if (is-musl-target) {
    # musl-tools to fix 'Failed to find tool. Is `musl-gcc` installed?'
    sudo apt-get install musl-tools -y
    build-static-with-cargo
  } else {
    build-with-cargo
  }
}

if $os in ['fedora', 'fedora-latest'] {
  print "Building on Fedora..."
  if $target == 'aarch64-unknown-linux-gnu' {
    sudo dnf install -y gcc-aarch64-linux-gnu
    $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    build-with-cargo
  } else if $target == 'armv7-unknown-linux-gnueabihf' {
    sudo dnf install pkg-config gcc-arm-linux-gnueabihf -y
    $env.CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = 'arm-linux-gnueabihf-gcc'
    build-with-cargo
  }
}


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

print "Compiling a release archive..."

let archive_filename = $'($binary)-($version)-($target).tar.gz'
print $'Release archive name: ($archive_filename)'
tar --verbose -C $release_dir -czf $archive_filename $binary
print $'Release archive at ($archive_filename) is ready'
echo $'archive=($archive_filename)' | save --append $env.GITHUB_OUTPUT

def 'build-with-cargo' [] {
  cargo rustc --bin $binary --target $target --release
}

def 'build-static-with-cargo' [] {
  $env.RUSTFLAGS = '-C target-feature=+crt-static'
  cargo rustc --bin $binary --target $target --release
}