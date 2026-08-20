use std::io::{Cursor, Read};
use std::path::Path;

use anyhow::{Context, Result, anyhow};

pub struct ArchiveExtractor<'a> {
    archive_name: String,
    data:         &'a [u8],
}

impl<'a> ArchiveExtractor<'a> {
    pub fn new(archive_name: impl Into<String>, data: &'a [u8]) -> Self {
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
            || n.ends_with(".tar.zst")
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
            self.deb_members()
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

    /// Extracts the member matching `path`.
    ///
    /// `path` comes from the registry and may be either a bare file name (`dust.1`) or a
    /// relative path (`usr/share/bash-completion/completions/dust`). A path match is tried first —
    /// the member either equals `path` or ends with `/path` — so that a registry entry can
    /// disambiguate archives holding several files with the same base name (a `.deb` ships
    /// `usr/bin/dust` next to `usr/share/bash-completion/completions/dust`). Matching on the
    /// base name alone is the fallback, which is what a bare file name relies on.
    pub fn extract_by_path(&self, path: &str) -> Result<Vec<u8>> {
        let members = self.members()?;
        let suffix = format!("/{path}");
        let member = members
            .iter()
            .find(|m| {
                let m = Self::normalize(m);
                m == path || m.ends_with(&suffix)
            })
            .or_else(|| {
                let base = Path::new(path).file_name();
                members
                    .iter()
                    .find(|m| Path::new(m.as_str()).file_name() == base)
            })
            .cloned()
            .ok_or_else(|| anyhow!("Can't find '{}' in '{}'", path, self.archive_name))?;
        self.extract(&member)
    }

    /// Strips the `./` prefix tar members are often stored with, so that member paths compare
    /// against registry paths.
    fn normalize(member: &str) -> &str { member.strip_prefix("./").unwrap_or(member) }

    pub fn extract(&self, member: &str) -> Result<Vec<u8>> {
        if self.is_tar() {
            self.tar_extract(member)
        } else if self.is_zip() {
            self.zip_extract(member)
        } else if self.is_ar_deb() {
            self.deb_extract(member)
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
        let cursor = Cursor::new(self.data);
        let n = self.name().to_lowercase();

        let reader: Box<dyn Read> = if n.ends_with(".tar.gz") {
            Box::new(flate2::read::GzDecoder::new(cursor))
        } else if n.ends_with(".tar.bz2") {
            Box::new(bzip2::read::BzDecoder::new(cursor))
        } else if n.ends_with(".tar.xz") {
            Box::new(xz2::read::XzDecoder::new(cursor))
        } else if n.ends_with(".tar.zst") {
            Box::new(zstd::Decoder::new(cursor)?)
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
        let cursor = Cursor::new(self.data);
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
        let cursor = Cursor::new(self.data);
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
        let cursor = Cursor::new(self.data);
        let mut archive = ar::Archive::new(cursor);
        let mut members = Vec::new();
        while let Some(entry) = archive.next_entry() {
            let entry = entry?;
            let name = String::from_utf8_lossy(entry.header().identifier()).into_owned();
            members.push(name);
        }
        Ok(members)
    }

    /// A `.deb` is an `ar` archive whose payload is a nested `data.tar.*`. Both `members()` and
    /// `extract()` see through to that payload, so a `deb` asset behaves like the tarball it
    /// wraps and registry paths refer to the installed layout (`usr/bin/dust`).
    fn deb_data_tar(&self) -> Result<(String, Vec<u8>)> {
        let member = self
            .ar_members()?
            .into_iter()
            .find(|m| m.trim_end_matches('/').starts_with("data.tar"))
            .ok_or_else(|| anyhow!("Can't find 'data.tar' in '{}'", self.name()))?;
        let data = self.ar_extract(&member)?;
        // GNU ar terminates identifiers with '/'; the tar dispatch matches on the extension.
        Ok((member.trim_end_matches('/').to_owned(), data))
    }

    fn deb_members(&self) -> Result<Vec<String>> {
        let (name, data) = self.deb_data_tar()?;
        ArchiveExtractor::new(name, &data).members()
    }

    fn deb_extract(&self, member: &str) -> Result<Vec<u8>> {
        let (name, data) = self.deb_data_tar()?;
        ArchiveExtractor::new(name, &data).extract(member)
    }

    fn ar_extract(&self, member: &str) -> Result<Vec<u8>> {
        let cursor = Cursor::new(self.data);
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
        let cursor = Cursor::new(self.data);
        let mut decoder = flate2::read::GzDecoder::new(cursor);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn xz_decompress(&self) -> Result<Vec<u8>> {
        let cursor = Cursor::new(self.data);
        let mut decoder = xz2::read::XzDecoder::new(cursor);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_unsupported_extension_returns_err() {
        let e = ArchiveExtractor::new("app.exe", &[]);
        assert!(e.members().is_err());
    }

    #[test]
    fn members_gz_only_returns_inner_name_without_decompressing() {
        // members() strips the .gz suffix without touching the data bytes.
        let e = ArchiveExtractor::new("app-linux.gz", &[]);
        let names = e.members().unwrap();
        assert_eq!(names, vec!["app-linux"]);
    }

    #[test]
    fn members_xz_only_returns_inner_name_without_decompressing() {
        let e = ArchiveExtractor::new("app-linux.xz", &[]);
        let names = e.members().unwrap();
        assert_eq!(names, vec!["app-linux"]);
    }

    /// Builds an uncompressed tar with one file per `(path, contents)` pair.
    fn tar_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, contents) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, path, *contents).unwrap();
        }
        builder.into_inner().unwrap()
    }

    /// Builds a `.deb`: an `ar` archive wrapping `debian-binary` and an uncompressed `data.tar`.
    fn deb_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let data_tar = tar_with(entries);
        let mut builder = ar::Builder::new(Vec::new());
        builder
            .append(&ar::Header::new(b"debian-binary".to_vec(), 4), &b"2.0\n"[..])
            .unwrap();
        builder
            .append(
                &ar::Header::new(b"data.tar".to_vec(), data_tar.len() as u64),
                data_tar.as_slice(),
            )
            .unwrap();
        builder.into_inner().unwrap()
    }

    #[test]
    fn deb_members_see_through_to_the_nested_data_tar() {
        let deb = deb_with(&[("./usr/bin/foo", b"elf")]);
        let e = ArchiveExtractor::new("foo_1.0_amd64.deb", &deb);
        // Not debian-binary / data.tar — the ar wrapper is transparent.
        assert_eq!(e.members().unwrap(), vec!["usr/bin/foo"]);
        assert_eq!(e.extract_by_path("usr/bin/foo").unwrap(), b"elf");
    }

    #[test]
    fn extract_by_path_prefers_the_full_path_over_a_colliding_base_name() {
        // A deb ships the completion script and the executable under the same base name, with
        // the completion first — base-name matching alone would install the wrong one.
        let deb = deb_with(&[
            ("./usr/share/bash-completion/completions/dust", b"# completion"),
            ("./usr/bin/dust", b"elf"),
        ]);
        let e = ArchiveExtractor::new("du-dust_1.2.5-1_amd64.deb", &deb);
        assert_eq!(
            e.extract_by_path("usr/share/bash-completion/completions/dust")
                .unwrap(),
            b"# completion"
        );
        assert_eq!(e.extract_by_path("usr/bin/dust").unwrap(), b"elf");
    }

    #[test]
    fn extract_by_path_falls_back_to_the_base_name() {
        // Registry paths that don't mirror the archive layout still resolve by base name.
        let tar = tar_with(&[("pkg-1.0/completion/foo.bash", b"# completion")]);
        let e = ArchiveExtractor::new("pkg.tar", &tar);
        assert_eq!(
            e.extract_by_path("build/completion/foo.bash").unwrap(),
            b"# completion"
        );
        assert_eq!(e.extract_by_path("foo.bash").unwrap(), b"# completion");
    }

    #[test]
    fn extract_by_path_missing_member_names_the_archive() {
        let deb = deb_with(&[("./usr/bin/foo", b"elf")]);
        let e = ArchiveExtractor::new("foo_1.0_amd64.deb", &deb);
        let err = e.extract_by_path("_foo").unwrap_err().to_string();
        assert!(err.contains("_foo") && err.contains("foo_1.0_amd64.deb"), "{err}");
    }

    #[test]
    fn members_tar_gz_not_treated_as_gz_only() {
        // .tar.gz must not fall through to the gz-only branch.
        // It will fail to parse as a tar (empty data), but the error must be a tar error, not a
        // "unsupported" error, which proves it was dispatched to the tar handler.
        let e = ArchiveExtractor::new("app.tar.gz", &[]);
        let err = e.members().unwrap_err();
        assert!(
            !err.to_string().contains("Unsupported archive type"),
            "tar.gz was mis-dispatched to the unsupported-type handler"
        );
    }
}
