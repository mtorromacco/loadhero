mod cli;
mod models;

use std::{thread::{self, JoinHandle}, time::{Duration, Instant}, collections::HashMap, sync::{Arc, Mutex}};
use clap::Parser;
use textplots::{Chart, Shape, Plot};
use cli::Cli;
use models::ResponseInfo;

fn main() {

    let mut cli: Cli = Cli::parse();

    println!("🚨 Riepilogo:\n\tNumero di richieste al secondo: {}\n\tPer i prossimi {} secondi\n\tAll'URL {}\n", cli.requests_per_second, cli.seconds, cli.url);

    let mut requests: Vec<JoinHandle<()>> = vec![];

    let results: Arc<Mutex<Vec<ResponseInfo>>> = Arc::new(Mutex::new(vec![]));

    let started_time = Instant:: now();

    let mut sended_requests: u32 = 0;

    for second in 0..cli.seconds {

        if second > 0 {
            cli.requests_per_second = (cli.requests_per_second as f32 * (1.0 + (cli.increment as f32 / 100.0) as f32)).ceil() as u32;
        }

        for _ in 0..cli.requests_per_second {

            let url: String = cli.url.clone();
            let result = Arc::clone(&results);

            let headers = cli.headers.clone();
            let query_strings = cli.query_strings.clone();

            let request = thread::spawn(move || {

                let request_time: Instant = Instant::now();

                let client = reqwest::blocking::Client::new();
                let mut request_builder = client.get(url);

                for header in headers {
                    let key = header.split("=").next().unwrap();
                    let value = header.split("=").last().unwrap();
                    request_builder = request_builder.header(key, value);
                }

                for query_string in query_strings {
                    let key = query_string.split("=").next().unwrap();
                    let value = query_string.split("=").last().unwrap();
                    request_builder = request_builder.query(&[(key, value)]);
                }

                let resp = request_builder.send();

                let time = request_time.elapsed().as_millis();
                
                match resp {
                    Ok(res) => {
                        let response_info = ResponseInfo::new(res.status().as_u16(), time);
                        result.lock().unwrap().push(response_info);
                    },
                    _ => {}
                }

            });

            sended_requests += 1;

            requests.push(request);

        }

        println!("✈️ Inviate {} richieste", sended_requests);
        thread::sleep(Duration::from_secs(1));
    }    

    println!("💨 Finalizzazione in corso..");
    
    for request in requests {
        request.join().expect("Si è verificato un errore");
    }

    println!();

    println!("============================================================");
    println!("              🎉 TEST DI CARICO COMPLETATO 🎉              ");
    println!("============================================================");

    println!();

    println!("📃 REPORT FINALE");
    println!("🔥 Richieste completate con successo: {}/{}", results.lock().unwrap().len(), sended_requests);

    if results.lock().unwrap().len() == 0 {
        println!("\n❌ Nessuna richiesta completata con successo! Controlla che l'applicazione sia online");
        return;
    }

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

    Chart::new(200, 55, 0 as f32, (points.len() - 1) as f32)
        .lineplot(&Shape::Steps(&points))
        .display();

}