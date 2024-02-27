use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use xml::reader::{EventReader, XmlEvent};

fn main() {
    let mut file_pairs = vec![];

    for i in 0..10 {
        file_pairs.push((
            format!("xml/myXMLFile{i} (copy).xml"),
            format!("xml/myXMLFile{i}.xml"),
        ))
    }

    let (tx, rx) = mpsc::channel();

    for pair in file_pairs {
        let tx = tx.clone();
        let file1 = pair.0.to_string();
        let file2 = pair.1.to_string();

        thread::spawn(move || {
            let result = match compare_files(&file1, &file2) {
                Ok(true) => format!("{} and {} are the same", file1, file2),
                Ok(false) => format!("{} and {} are different", file1, file2),
                Err(err) => format!("Error comparing files: {}", err),
            };
            tx.send(result).unwrap();
        });
    }

    drop(tx);

    for received in rx {
        println!("{}", received);
    }
}

fn compare_files(file1: &str, file2: &str) -> Result<bool, Box<dyn Error>> {
    let reader1 = BufReader::new(File::open(file1)?);
    let reader2 = BufReader::new(File::open(file2)?);

    let parser1 = EventReader::new(reader1);
    let parser2 = EventReader::new(reader2);

    for (event1, event2) in parser1.into_iter().zip(parser2.into_iter()) {
        match (event1?, event2?) {
            (
                XmlEvent::StartElement {
                    name, attributes, ..
                },
                XmlEvent::StartElement {
                    name: name2,
                    attributes: attributes2,
                    ..
                },
            ) if name == name2 && attributes == attributes2 => {}
            (XmlEvent::EndElement { name }, XmlEvent::EndElement { name: name2 })
                if name == name2 => {}
            (XmlEvent::Characters(s1), XmlEvent::Characters(s2)) if s1 != s2 => {
                return Ok(false);
            }
            (XmlEvent::Comment(comment), XmlEvent::Comment(comment2)) if comment != comment2 => {
                return Ok(false);
            }
            (XmlEvent::CData(cdata), XmlEvent::CData(cdata2)) if cdata != cdata2 => {
                return Ok(false);
            }
            (XmlEvent::Characters(s1), XmlEvent::Characters(s2)) if s1 == s2 => {}
            (
                XmlEvent::StartDocument {
                    version,
                    encoding,
                    standalone,
                },
                XmlEvent::StartDocument {
                    version: version2,
                    encoding: encoding2,
                    standalone: standalone2,
                },
            ) if version == version2 && encoding == encoding2 && standalone == standalone2 => {}
            (XmlEvent::EndDocument, XmlEvent::EndDocument) => {
                return Ok(true);
            }
            (XmlEvent::Whitespace(_w1), XmlEvent::Whitespace(_w2)) => {}
            (
                XmlEvent::ProcessingInstruction { name, data },
                XmlEvent::ProcessingInstruction {
                    name: name2,
                    data: data2,
                },
            ) if name == name2 && data == data2 => {}
            _ => return Ok(false),
        }
    }

    Ok(true)
}
