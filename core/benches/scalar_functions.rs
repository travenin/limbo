use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use limbo_core::{Database, PlatformIO, IO};
use pprof::criterion::{Output, PProfProfiler};
use rusqlite::types::Value;
use std::sync::Arc;

fn bench(c: &mut Criterion) {
    hex_bench(c);
    unhex_bench(c);
    substr_bench(c);
}

fn hex_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex");
    let inputs = vec![
        "",
        "limbo",
        "Lorem ipsum odor amet, consectetuer adipiscing elit. Ridiculus tristique cursus commodo primis volutpat elementum. Ante vehicula efficitur nulla scelerisque dictumst nisi. Rhoncus metus vitae; fusce vel facilisis hac. Nullam laoreet nostra lorem tempus nam varius mauris aliquet. Velit platea ultricies senectus conubia nisi ultrices mauris dignissim interdum. Rutrum magnis condimentum ultrices egestas imperdiet, elit consequat. Odio sagittis turpis ex ipsum est.",
    ];

    for input in inputs {
        group.throughput(Throughput::Elements(1));
        group.bench_function(format!("limbo: hex(x), len = {}", input.len()), |b| {
            b.iter(|| call_limbo(&format!("SELECT hex('{}')", input)));
        });
        group.bench_function(format!("rusqlite: hex(x), len = {}", input.len()), |b| {
            b.iter(|| call_rusqlite(&format!("SELECT hex('{}')", input)));
        });
    }
}

fn unhex_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("unhex");
    let inputs = vec![
        "",
        "6C696D626F",
        "4C6F72656D20697073756D206F646F7220616D65742C20636F6E7365637465747565722061646970697363696E6720656C69742E205269646963756C7573207472697374697175652063757273757320636F6D6D6F646F207072696D697320766F6C757470617420656C656D656E74756D2E20416E7465207665686963756C6120656666696369747572206E756C6C61207363656C657269737175652064696374756D7374206E6973692E2052686F6E637573206D657475732076697461653B2066757363652076656C20666163696C69736973206861632E204E756C6C616D206C616F72656574206E6F73747261206C6F72656D2074656D707573206E616D20766172697573206D617572697320616C69717565742E2056656C697420706C6174656120756C747269636965732073656E656374757320636F6E75626961206E69736920756C747269636573206D6175726973206469676E697373696D20696E74657264756D2E2052757472756D206D61676E697320636F6E64696D656E74756D20756C747269636573206567657374617320696D706572646965742C20656C697420636F6E7365717561742E204F64696F2073616769747469732074757270697320657820697073756D206573742E",
    ];

    for input in inputs {
        group.throughput(Throughput::Elements(1));
        group.bench_function(format!("limbo: unhex(x), len = {}", input.len()), |b| {
            b.iter(|| call_limbo(&format!("SELECT unhex('{}')", input)));
        });
        group.bench_function(format!("rusqlite: unhex(x), len = {}", input.len()), |b| {
            b.iter(|| call_rusqlite(&format!("SELECT unhex('{}')", input)));
        });
    }
}

fn substr_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("substr");
    let inputs = vec![
        ("limbo", 2, 3),
        ("Lorem ipsum odor amet, consectetuer adipiscing elit. Ridiculus tristique cursus commodo primis volutpat elementum. Ante vehicula efficitur nulla scelerisque dictumst nisi. Rhoncus metus vitae; fusce vel facilisis hac. Nullam laoreet nostra lorem tempus nam varius mauris aliquet. Velit platea ultricies senectus conubia nisi ultrices mauris dignissim interdum. Rutrum magnis condimentum ultrices egestas imperdiet, elit consequat. Odio sagittis turpis ex ipsum est.", 20, 30),
        ];

    for input in inputs {
        group.throughput(Throughput::Elements(1));
        let query = format!("SELECT substr('{}', {}, {})", input.0, input.1, input.2);
        group.bench_function(
            format!("limbo: substr(x,y,z) len = {}", input.0.len()),
            |b| {
                b.iter(|| call_limbo(&query));
            },
        );
        group.bench_function(
            format!("rusqlite: substr(x,y,z) len = {}", input.0.len()),
            |b| {
                b.iter(|| call_rusqlite(&query));
            },
        );
    }
}

fn call_limbo(query: &str) {
    let io = Arc::new(PlatformIO::new().unwrap());
    let db = Database::open_file(io.clone(), "../testing/testing.db").unwrap();
    let conn = db.connect();
    let mut stmt = conn.prepare(query).unwrap();
    let mut rows = stmt.query().unwrap();
    match rows.next_row().unwrap() {
        limbo_core::RowResult::Row(row) => {
            // ?
        }
        limbo_core::RowResult::IO => {
            io.run_once().unwrap();
        }
        limbo_core::RowResult::Done => {
            unreachable!();
        }
    }
    stmt.reset();
}

fn call_rusqlite(query: &str) {
    let conn = rusqlite::Connection::open("../testing/testing.db").unwrap();
    let mut stmt = conn.prepare(query).unwrap();
    let mut rows = stmt.query(()).unwrap();
    let row = rows.next().unwrap().unwrap();
    let _result: Value = row.get(0).unwrap();
}

criterion_group! {
  name = benches;
  config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
  targets = bench
}
criterion_main!(benches);
