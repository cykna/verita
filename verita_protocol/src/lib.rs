#[macro_export]
macro_rules! define_struct {
    (
        $struct:path = $wrapper:ident {
        $($param:ident : $pty:ty),* $(,)?}
    ) => {
        paste::paste!{
             pub struct $wrapper;

            pub struct [<$wrapper Reader>] {
                segments: capnp::message::Reader<capnp::serialize::OwnedSegments>,
            }

            impl [<$wrapper Reader>] {
                pub fn reader(&self) -> Result<$struct::Reader<'_>, capnp::Error> {
                    self.segments.get_root()
                }
            }

            impl $wrapper {
                pub fn new(
                    $( $param: $pty, )*
                    out: &mut [u8],
                ) -> Result<(), capnp::Error> {
                    let mut msg = capnp::message::Builder::new_default();
                    let mut builder = msg.init_root::<$struct::Builder>();

                    $(
                        builder.[<set_ $param>]($param);
                    )*

                    capnp::serialize::write_message(out, &msg)
                }

                pub fn from_bytes(bytes: &[u8]) -> Result<[<$wrapper Reader>], capnp::Error> {
                    let msg = capnp::serialize::read_message(
                        bytes,
                        capnp::message::ReaderOptions::new()
                    )?;
                    Ok([<$wrapper Reader>] { segments: msg })
                }
            }
        }
    };
}
mod auth;
pub use auth::*;
capnp::generated_code!(mod protocol_capnp);

pub use protocol_capnp::*;
