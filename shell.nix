with import <nixpkgs> {};

stdenv.mkDerivation {
	name = "neo-shell"; # unused
	buildInputs = [
		gcc
		openssl
		pkgconfig
	];
}
