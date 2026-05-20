use anyhow::{Context, Result, anyhow};
use std::io::{Cursor, Read};

pub struct ArchiveExtractor {
    archive_name: String,
    data:         Vec<u8>,
}

impl ArchiveExtractor {
    pub fn new(archive_name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            archive_name: archive_name.into(),
            data,
        }
    }

    fn name(&self) -> &str { &self.archive_name }

    fn is_tar(&self) -> bool {
        let n = self.name().to_lowercase();
        n.ends_with(".tar.gz")
            || n.ends_with(".tar.bz2")
            || n.ends_with(".tar.xz")
            || n.ends_with(".tar")
    }

    fn is_zip(&self) -> bool { self.name().to_lowercase().ends_with(".zip") }

    fn is_ar_deb(&self) -> bool { self.name().to_lowercase().ends_with(".deb") }

    fn is_gzip_only(&self) -> bool {
        let n = self.name().to_lowercase();
        !self.is_tar() && n.ends_with(".gz")
    }

    fn is_xz_only(&self) -> bool {
        let n = self.name().to_lowercase();
        !self.is_tar() && n.ends_with(".xz")
    }

    pub fn members(&self) -> Result<Vec<String>> {
        if self.is_tar() {
            self.tar_members()
        } else if self.is_zip() {
            self.zip_members()
        } else if self.is_ar_deb() {
            self.ar_members()
        } else if self.is_gzip_only() {
            // Single compressed file — name without the .gz suffix
            let name = self.name();
            let inner = &name[..name.len() - 3];
            Ok(vec![inner.to_string()])
        } else if self.is_xz_only() {
            let name = self.name();
            let inner = &name[..name.len() - 3];
            Ok(vec![inner.to_string()])
        } else {
            Err(anyhow!("Unsupported archive type: {}", self.name()))
        }
    }

    pub fn extract(&self, member: &str) -> Result<Vec<u8>> {
        if self.is_tar() {
            self.tar_extract(member)
        } else if self.is_zip() {
            self.zip_extract(member)
        } else if self.is_ar_deb() {
            self.ar_extract(member)
        } else if self.is_gzip_only() {
            self.gz_decompress()
        } else if self.is_xz_only() {
            self.xz_decompress()
        } else {
            Err(anyhow!("Unsupported archive type: {}", self.name()))
        }
    }

    // ── tar helpers ──────────────────────────────────────────────────────────

    fn open_tar(&self) -> Result<tar::Archive<Box<dyn Read + '_>>> {
        let cursor = Cursor::new(&self.data);
        let n = self.name().to_lowercase();

        let reader: Box<dyn Read> = if n.ends_with(".tar.gz") {
            Box::new(flate2::read::GzDecoder::new(cursor))
        } else if n.ends_with(".tar.bz2") {
            Box::new(bzip2::read::BzDecoder::new(cursor))
        } else if n.ends_with(".tar.xz") {
            Box::new(xz2::read::XzDecoder::new(cursor))
        } else {
            Box::new(cursor)
        };

        Ok(tar::Archive::new(reader))
    }

    fn tar_members(&self) -> Result<Vec<String>> {
        let mut archive = self.open_tar()?;
        let mut members = Vec::new();
        for entry in archive.entries().context("reading tar entries")? {
            let entry = entry?;
            if entry.header().entry_type().is_file() {
                let path = entry.path()?.to_string_lossy().into_owned();
                members.push(path);
            }
        }
        Ok(members)
    }

    fn tar_extract(&self, member: &str) -> Result<Vec<u8>> {
        let mut archive = self.open_tar()?;
        for entry in archive.entries().context("reading tar entries")? {
            let mut entry = entry?;
            if entry.header().entry_type().is_file() {
                let path = entry.path()?.to_string_lossy().into_owned();
                if path == member {
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf)?;
                    return Ok(buf);
                }
            }
        }
        Err(anyhow!("Member '{}' not found in '{}'", member, self.name()))
    }

    // ── zip helpers ──────────────────────────────────────────────────────────

    fn zip_members(&self) -> Result<Vec<String>> {
        let cursor = Cursor::new(&self.data);
        let mut archive = zip::ZipArchive::new(cursor)?;
        Ok((0..archive.len())
            .filter_map(|i| {
                let file = archive.by_index_raw(i).ok()?;
                if !file.is_dir() {
                    Some(file.name().to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    fn zip_extract(&self, member: &str) -> Result<Vec<u8>> {
        let cursor = Cursor::new(&self.data);
        let mut archive = zip::ZipArchive::new(cursor)?;
        let mut file = archive
            .by_name(member)
            .with_context(|| format!("Member '{}' not found in '{}'", member, self.name()))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    // ── ar/deb helpers ───────────────────────────────────────────────────────

    fn ar_members(&self) -> Result<Vec<String>> {
        let cursor = Cursor::new(&self.data);
        let mut archive = ar::Archive::new(cursor);
        let mut members = Vec::new();
        while let Some(entry) = archive.next_entry() {
            let entry = entry?;
            let name = String::from_utf8_lossy(entry.header().identifier()).into_owned();
            members.push(name);
        }
        Ok(members)
    }

    fn ar_extract(&self, member: &str) -> Result<Vec<u8>> {
        let cursor = Cursor::new(&self.data);
        let mut archive = ar::Archive::new(cursor);
        while let Some(entry) = archive.next_entry() {
            let mut entry = entry?;
            let name = String::from_utf8_lossy(entry.header().identifier()).into_owned();
            if name == member {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                return Ok(buf);
            }
        }
        Err(anyhow!("Member '{}' not found in '{}'", member, self.name()))
    }

    // ── single-file decompressors ────────────────────────────────────────────

    fn gz_decompress(&self) -> Result<Vec<u8>> {
        let cursor = Cursor::new(&self.data);
        let mut decoder = flate2::read::GzDecoder::new(cursor);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn xz_decompress(&self) -> Result<Vec<u8>> {
        let cursor = Cursor::new(&self.data);
        let mut decoder = xz2::read::XzDecoder::new(cursor);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
