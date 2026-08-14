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
const TEMPLATE: &str = "return string.format(\"name = %q\\nsize = %d\\n\", name, size)\n";

fn templates(c: &mut Criterion) {
    let mut group = c.benchmark_group("template");
    let fixture = Fixture::new(0);
    let classes = Classes::default();
    let scripts = [
        ("startup", STARTUP.to_string()),
        ("file", PICK.to_string()),
        ("render", RENDER.to_string()),
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

    group.finish();
}

fn template(fixture: &Fixture, name: &str, script: &str) -> PathBuf {
    let dir = fixture
        .repo()
        .join(format!(".config/{name}/init.conf.luadot"));
    write(&dir.join("luadot.lua"), script);
    write(&dir.join("variant.conf"), "variant\n");
    write(&dir.join("init.tmpl.conf"), TEMPLATE);

    dir
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
