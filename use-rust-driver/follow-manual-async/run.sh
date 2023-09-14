export MONGOCRYPT_LIB_DIR=/Users/kevin.albertson/code/c-bootstrap/install/libmongocrypt-1.8.0/lib
export DYLD_LIBRARY_PATH=$MONGOCRYPT_LIB_DIR
export PATH=$PATH:/Users/q/bin/mongodb-macos-aarch64-enterprise-6.0.8/bin
cargo rustc -- -C panic=abort
./target/debug/follow-readme