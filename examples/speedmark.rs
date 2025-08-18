use english::*;

fn main() {
    //   println!("{}", English::noun("thyridium", &Number::Plural));
    benchmark_verb();
    // benchmark_noun();
    benchmark_adj();
    benchmark_insane();
}
use std::hint::black_box;
use std::time::Instant;

pub fn benchmark_verb() {
    let words = ["walk", "eat", "go", "see", "invent", "play"];
    let person = Person::Third;
    let number = Number::Singular;
    let tense = Tense::Present;
    let form = Form::Finite;

    run_benchmark("verb", &words, |w| {
        English::verb(w, &person, &number, &tense, &form)
    });
}
/*
pub fn benchmark_noun() {
    let words = ["cat", "child", "mouse", "bus", "sheep", "fish"];

    run_benchmark("noun", &words, |w| English::noun(w, &Number::Plural));
} */
pub fn benchmark_insane() {
    let words = ["cat90", "child90", "mouse90", "bus90", "sheep90", "fish90"];

    run_benchmark("insane", &words, |w| English::noun(w, &Number::Plural));
}

pub fn benchmark_adj() {
    let words = ["fast", "good", "bad", "fun", "happy"];

    run_benchmark("adjective", &words, |w| {
        English::adj(w, &Degree::Comparative)
    });
}

fn run_benchmark<F>(label: &str, words: &[&str], mut f: F)
where
    F: FnMut(&str) -> String,
{
    let iterations = 1_000_000;
    let total_calls = iterations * words.len();

    let start = Instant::now();
    let mut last_result = String::new();

    for _ in 0..iterations {
        for &word in words {
            // black_box prevents the optimizer from removing the call
            last_result = black_box(f(black_box(word)));
        }
    }

    let duration = start.elapsed();
    let nanos = duration.as_nanos() as f64;
    let calls_per_sec = (total_calls as f64) / (nanos / 1e9);

    let nanos_per_call = nanos / total_calls as f64;

    println!("[{label}] Last result: {last_result}");
    println!(
        "[{label}] Completed in {:?} → {} calls",
        duration, total_calls
    );
    println!(
        "[{label}] Throughput: {:.2} calls/sec | Time per call: {:.2} ns",
        calls_per_sec, nanos_per_call
    );
}
