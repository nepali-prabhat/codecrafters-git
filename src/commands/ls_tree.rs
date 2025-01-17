use std::io;
use std::io::{BufRead, Read, Write};

use anyhow::Context;

use crate::objects::ObjectType;

pub(crate) fn handler(inputs: crate::LsTree) -> anyhow::Result<()> {
    let (obj_type, mut reader) = ObjectType::get_handle(inputs.tree_sha)?;
    match obj_type {
        ObjectType::Tree => {
            let op = io::stdout();
            let mut op = op.lock();
            let mut hash = vec![0; 20];

            let mut buffer: Vec<u8> = Vec::new();

            loop {
                buffer.clear();

                let n = reader.read_until(b' ', &mut buffer)?;

                if n == 0 {
                    break;
                }
                buffer.pop();

                if !inputs.name_only {
                    let mut mode = std::str::from_utf8(&buffer[..]).context("converting to utf8 string")?;
                    let obj_type = match mode {
                        "100644" | "100755" | "120000" => ObjectType::Blob,
                        "40000" => {
                            mode = "040000";
                            ObjectType::Tree},
                        _ => {
                            anyhow::bail!("unknown value for mode {}", mode);
                        }
                    };

                    write!(&mut op, "{} {} ", mode, obj_type).context("headers to stdout")?;
                }

                buffer.clear();
                reader.read_until(0, &mut buffer)?;
                buffer.pop();
                let filename = std::str::from_utf8(&buffer[..])?;

                reader.read_exact(&mut hash).context("reading the hash")?;
                let sha1_hash = hex::encode(&hash[..]);
                if !inputs.name_only {
                    write!(&mut op, "{}    ", sha1_hash).context("sha1 hash to stdout")?;
                }

                write!(&mut op, "{}\n", filename).context("filename to stdout")?;
            }
        }
        _ => {
            anyhow::bail!("don't know how to process object {:?}", obj_type);
        }
    }

    Ok(())
}
