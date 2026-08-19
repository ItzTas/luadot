mod support;

use std::hint::black_box;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use luadot::lua;
use luadot::state::Classes;
use support::{Fixture, OUTPUT_COUNT, write};

const STARTUP: &str = "return \"generated\\n\"\n";
const PICK: &str = "return ld.alt.file(\"variant.conf\")\n";
const RENDER: &str =
    "ld.alt.out({ content = ld.alt.render(\"init.tmpl.conf\", { name = \"host\", size = 24 }) })\n";
const EXPAND: &str = "ld.alt.out({ content = ld.alt.expand(\"init.tmpl.embed\", { name = \"host\", size = 24 }) })\n";
const TEMPLATE: &str = "return string.format(\"name = %q\\nsize = %d\\n\", name, size)\n";
const EMBED_LINES: usize = 64;

fn templates(c: &mut Criterion) {
    let mut group = c.benchmark_group("template");
    let fixture = Fixture::new(0);
    let classes = Classes::default();
    let scripts = [
        ("startup", STARTUP.to_string()),
        ("file", PICK.to_string()),
        ("render", RENDER.to_string()),
        ("expand", EXPAND.to_string()),
        ("outputs", outputs(OUTPUT_COUNT)),
    ];

    for (name, script) in &scripts {
        let dir = template(&fixture, name, script);

        group.bench_function(*name, |b| {
            b.iter(|| {
                black_box(
                    lua::load_template("alt", fixture.home(), fixture.repo(), &dir, &classes)
                        .expect("a resolved template"),
                )
            });
        });
    }

    let file = fixture.repo().join(".config/standalone/init.conf.luadot");
    write(&file, &embedded(EMBED_LINES, "\"host\"", "24"));

    group.bench_function("standalone", |b| {
        b.iter(|| {
            black_box(
                lua::load_template_file("alt", fixture.home(), fixture.repo(), &file, &classes)
                    .expect("a resolved standalone template"),
            )
        });
    });

    group.finish();
}

fn template(fixture: &Fixture, name: &str, script: &str) -> PathBuf {
    let dir = fixture
        .repo()
        .join(format!(".config/{name}/init.conf.luadot"));
    write(&dir.join("luadot.lua"), script);
    write(&dir.join("variant.conf"), "variant\n");
    write(&dir.join("init.tmpl.conf"), TEMPLATE);
    write(
        &dir.join("init.tmpl.embed"),
        &embedded(EMBED_LINES, "name", "size"),
    );

    dir
}

fn embedded(lines: usize, name: &str, size: &str) -> String {
    let mut source = format!("name = <%= {name} %>\nsize = <%= {size} %>\n");
    for index in 0..lines {
        source.push_str(&format!("key{index:03} = <%= {size} + {index} %>\n"));
    }
    source
}

fn outputs(count: usize) -> String {
    (0..count)
        .map(|index| {
            format!(
                "ld.alt.out({{ dest = \"~/.config/generated{index:03}.conf\", content = ld.alt.render(\"init.tmpl.conf\", {{ name = \"host{index}\", size = {index} }}) }})\n"
            )
        })
        .collect()
}

criterion_group!(benches, templates);
criterion_main!(benches);
