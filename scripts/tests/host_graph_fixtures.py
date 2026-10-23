"""Authored host graph fixtures: explicit archives, renamed edges and target stop."""
import gzip
import io
import tarfile

from host_process import digest
from package_snapshot import extract
from local_package import capture

FILES = {'app/Cargo.toml': '[package]\n'
                   'name="app"\n'
                   'version="1.0.0"\n'
                   'edition="2024"\n'
                   '[dependencies]\n'
                   'renamed={package="helper",version="=1.0.0",features=["target_side"]}\n'
                   'native-sys={version="=1.0.0"}\n'
                   '[build-dependencies]\n'
                   'renamed={package="helper",version="=1.0.0",features=["host_side"]}\n',
 'app/build.rs': 'fn main(){\n'
                 ' assert_eq!(renamed::value(),17);\n'
                 ' '
                 'assert_eq!(std::env::var("DEP_RUSTGM_NATIVE_ROOT_DIR").unwrap(),"headers=ready");\n'
                 ' assert_eq!(std::env::var("DEP_RUSTGM_NATIVE_EMPTY").unwrap(),"");\n'
                 ' assert!(std::env::var("DEP_RUSTGM_NATIVE_HIDDEN").is_err());\n'
                 ' println!("cargo::rustc-env=APP_VALUE={}",renamed::value());\n'
                 ' '
                 'std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("answer.rs"),"pub '
                 'const ANSWER:u8=17;\\n").unwrap();\n'
                 '}\n',
 'app/src/lib.rs': 'compile_error!("intentional target graph stop");\n',
 'helper/Cargo.toml': '[package]\n'
                      'name="helper"\n'
                      'version="1.0.0"\n'
                      'edition="2021"\n'
                      '[features]\n'
                      'host_side=[]\n'
                      'target_side=[]\n'
                      '[dependencies]\n'
                      'native-sys={version="=1.0.0"}\n',
 'helper/build.rs': 'fn main(){\n'
                    ' '
                    'assert_eq!(std::env::var("DEP_RUSTGM_NATIVE_ROOT_DIR").unwrap(),"headers=ready");\n'
                    ' println!("cargo::rustc-check-cfg=cfg(ready)");\n'
                    ' println!("cargo::rustc-cfg=ready");\n'
                    ' println!("cargo::rustc-env=LOCAL_VALUE=checked");\n'
                    ' println!("cargo::metadata=hidden=not-transitive");\n'
                    ' '
                    'std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("generated.rs"),"pub '
                    'fn generated()->u8{16}\\n").unwrap();\n'
                    '}\n',
 'helper/src/lib.rs': 'include!(concat!(env!("OUT_DIR"),"/generated.rs"));\n'
                      '#[cfg(all(feature="host_side",not(feature="target_side"),ready))]\n'
                      'pub fn value()->u8 { assert_eq!(env!("LOCAL_VALUE"),"checked"); '
                      'generated()+native_sys::one() }\n'
                      '#[cfg(all(feature="target_side",not(feature="host_side"),ready))]\n'
                      'pub fn value()->u8 { 99 }\n',
 'native-sys/Cargo.toml': '[package]\n'
                          'name="native-sys"\n'
                          'version="1.0.0"\n'
                          'edition="2021"\n'
                          'links="rustgm-native"\n',
 'native-sys/build.rs': 'fn main(){\n'
                        ' '
                        'assert_eq!(std::env::var("CARGO_MANIFEST_LINKS").unwrap(),"rustgm-native");\n'
                        ' println!("cargo:root-dir=old");\n'
                        ' println!("cargo::metadata=root-dir=headers=ready");\n'
                        ' println!("cargo::metadata=empty=");\n'
                        '}\n',
 'native-sys/src/lib.rs': 'pub fn one()->u8{1}\n'}


def packages(root):
    vendor = root / 'vendor'
    vendor.mkdir()
    result = []
    for name in ('app', 'helper', 'native-sys'):
        archive = root / (name + '.crate')
        with archive.open('wb') as raw, gzip.GzipFile(fileobj=raw, mode='wb', mtime=0, filename='') as zipped, \
                tarfile.open(fileobj=zipped, mode='w|') as output:
            for path, text in FILES.items():
                prefix, relative = path.split('/', 1)
                if prefix != name:
                    continue
                data = text.encode()
                entry = tarfile.TarInfo(name + '-1.0.0/' + relative)
                entry.size, entry.mode = len(data), 0o644
                output.addfile(entry, io.BytesIO(data))
        result.append(extract(archive, digest(archive), name, '1.0.0', vendor))
    driver = root / 'driver'
    driver.mkdir()
    (driver / 'Cargo.toml').write_text('[package]\nname="driver"\nversion="1.0.0"\nedition="2024"\n'
                                     '[workspace]\nresolver="2"\n[dependencies]\napp="=1.0.0"\n'
                                     '[profile.dev]\ndebug=0\n[lib]\npath="lib.rs"\n')
    (driver / 'lib.rs').write_text('pub fn driver(){}\n')
    return tuple(result), driver


def local_packages(root):
    """Same authored host behavior, with explicit path dependencies and originals."""
    workspace = root / 'workspace'
    workspace.mkdir()
    for path, text in FILES.items():
        destination = workspace / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.name == 'Cargo.toml':
            text = text.replace('package="helper",version="=1.0.0"',
                                'package="helper",path="../helper"')
            text = text.replace('native-sys={version="=1.0.0"}',
                                'native-sys={path="../native-sys"}')
        destination.write_text(text)
    (workspace / 'Cargo.toml').write_text(
        '[workspace]\nmembers=["app","helper","native-sys"]\n'
        'default-members=["app"]\nresolver="2"\n[profile.dev]\ndebug=0\n')
    result = tuple(capture(workspace / name, name, '1.0.0', 'fixture/' + name)
                   for name in ('app', 'helper', 'native-sys'))
    return result, workspace
