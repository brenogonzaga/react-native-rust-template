// C interface exported by librust_bridge.a (see rust-core/src/lib.rs).
#ifndef RUST_BRIDGE_H
#define RUST_BRIDGE_H

/// Runs a bridge command. `envelope` is a JSON document: {"cmd":...,"args":...}.
/// The returned string is owned by the caller and must be released with
/// free_rust_string(). Returns NULL if the response could not be encoded.
char *call_rust(const char *envelope);

/// Releases a string returned by call_rust(). NULL is a no-op.
void free_rust_string(char *s);

#endif /* RUST_BRIDGE_H */
