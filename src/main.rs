mod cli;

use std::{thread::{self, JoinHandle}, time::{Duration, Instant}, collections::HashMap, sync::{Arc, Mutex}};
use clap::Parser;
use textplots::{Chart, Shape, Plot};
use crate::cli::Cli;

fn main() {

    let cli: Cli = Cli::parse();

    println!("🚨 Riepilogo:\n\tNumero di richieste al secondo: {}\n\tPer i prossimi {} secondi\n\tAll'URL {}\n", cli.requests_per_second, cli.seconds, cli.url);

    let mut requests: Vec<JoinHandle<()>> = vec![];

    let results: Arc<Mutex<Vec<ResponseInfo>>> = Arc::new(Mutex::new(vec![]));

    let started_time = Instant:: now();

    for second in 0..cli.seconds {

        for _ in 0..cli.requests_per_second {

            let url: String = cli.url.clone();
            let result = Arc::clone(&results);

            let request = thread::spawn(move || {

                let request_time: Instant = Instant::now();
                let resp = reqwest::blocking::get(url);
                let time = request_time.elapsed().as_millis();

                match resp {
                    Ok(res) => {
                        let response_info = ResponseInfo::new(res.status().as_u16(), time);
                        result.lock().unwrap().push(response_info);
                    },
                    _ => {}
                }

            });

            requests.push(request);

        }

        println!("✈️ Inviate {} richieste", cli.requests_per_second * (second + 1));
        thread::sleep(Duration::from_secs(1));
    }    

    println!("💨 Finalizzazione in corso..");
    
    for request in requests {
        request.join().expect("Si è verificato un errore");
    }

    println!();

    println!("============================================================");
    println!("        🎉 TEST DI CARICO COMPLETATO CON SUCCESSO 🎉        ");
    println!("============================================================");

    println!();

    println!("📃 REPORT FINALE");
    println!("🔥 Richieste completate: {}", results.lock().unwrap().len());

    println!("🕛 Tempo totale: {} s", started_time.elapsed().as_secs_f32());

    let mut average_time: u128 = 0;
    let mut min_time = 10000;
    let mut max_time = 0;
    let mut status_codes: HashMap<u16, u32> = HashMap::new();

    let mut points: Vec<(f32, f32)> = vec![(0.0, 0.0)];
    let mut index: f32 = 1.0;

    for result in results.lock().unwrap().iter() {
        average_time += result.time;
        
        match status_codes.get_mut(&result.status) {
            Some(value) => {
                *value += 1;
            },
            None => {
                status_codes.insert(result.status, 1);
            },
        }

        if result.time < min_time {
            min_time = result.time;
        }

        if result.time > max_time {
            max_time = result.time;
        }

        let point: (f32, f32) = (index, result.time as f32);
        points.push(point);

        index += 1.0;
    }

    println!("🎯 Status codes ricevuti:");

    for status_code in status_codes {
        println!("\tStatus code: {} → {}", status_code.0, status_code.1);
    }

    println!("🕛 Tempo medio: {} ms", average_time / results.lock().unwrap().len() as u128);
    println!("🐌 Richiesta più lenta: {} ms", max_time);
    println!("🚀 Richiesta più veloce: {} ms", min_time);
    println!("\n");

    Chart::new(200, 55, 0 as f32, points.len() as f32)
        .lineplot(&Shape::Steps(&points))
        .display();

}


struct ResponseInfo {
    status: u16,
    time: u128
}

impl ResponseInfo {

    fn new(status: u16, time: u128) -> ResponseInfo {
        ResponseInfo { status, time }
    }

}