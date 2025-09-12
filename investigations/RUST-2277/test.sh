export MONGOCRYPT_LIB_DIR=$(cygpath --windows /cygdrive/c/code/c-bootstrap/install/libmongocrypt-1.15.1/lib)
export PATH="$PATH:$MONGOCRYPT_LIB_DIR/../bin"
export MONGOCRYPTD_PATH=$(cygpath --windows /cygdrive/c/bin/mongodb-win32-x86_64-enterprise-windows-8.3.0-alpha0-1179-gb3a8643/bin/mongocryptd.exe)
cargo run