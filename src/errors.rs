use mongodb;
use solana_client;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        Internal(t: String) {
            description("invalid toolchain name")
            display("invalid toolchain name: '{}'", t)
        }
    }

    foreign_links {
        ClientError(solana_client::client_error::ClientError);
        MongoError(mongodb::error::Error);
    }

}
