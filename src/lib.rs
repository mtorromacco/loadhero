mod models;

use std::{thread::{JoinHandle, self}, sync::{Mutex, Arc}, time::{Instant, Duration}, collections::HashMap, error::Error};
use reqwest::blocking::{Client, RequestBuilder, Response};
use textplots::{Chart, Shape, Plot};
use models::{ResponseInfo, StatsInfo};


/// Punto di ingresso effettivo della logica che si preoccupa di inviare le richieste e visualizzare statistiche e grafico finale
/// run() → send_requests() → execute_request()
/// run() → calc_stats()
pub fn run(seconds: u32, requests_per_second: u32, increment: u8, url: String, headers: Vec<String>, query_strings: Vec<String>) -> Result<(), Box<dyn Error>> {
    
    println!("🚨 Riepilogo:\n\tNumero di richieste al secondo: {}\n\tPer i prossimi {} secondi\n\tAll'URL {}\n", requests_per_second, seconds, url);

    let now: Instant = Instant:: now();

    let (successful_requests, sended_requests) = send_requests(seconds, requests_per_second, increment, url, headers, query_strings);

    let successful_requests_number: usize = successful_requests.len();

    println!("\n============================================================");
    println!("              🎉 TEST DI CARICO COMPLETATO 🎉              ");
    println!("============================================================\n");

    println!("📃 REPORT FINALE");
    println!("🔥 Richieste completate con successo: {}/{}", successful_requests_number, sended_requests);

    if successful_requests_number == 0 {
        println!("\n❌ Nessuna richiesta completata con successo! Controlla che l'applicazione sia online");
        return Ok(());
    }

    println!("🕛 Tempo totale: {} s", now.elapsed().as_secs_f32());

    let stats_info: StatsInfo = calc_stats(&successful_requests);

    println!("🎯 Status codes ricevuti:");

    for status_code in stats_info.status_codes {
        println!("\tStatus code: {} → {}", status_code.0, status_code.1);
    }

    println!("🕛 Tempo medio: {} ms", stats_info.average_time / successful_requests_number as u128);
    println!("🐌 Richiesta più lenta: {} ms", stats_info.max_time);
    println!("🚀 Richiesta più veloce: {} ms", stats_info.min_time);
    println!("\n");

    Chart::new(200, 55, 0 as f32, (stats_info.points.len() - 1) as f32)
        .lineplot(&Shape::Steps(&stats_info.points))
        .display();

    Ok(())
}


/// Funzione che si preoccupa di gestire le richieste nel tempo prefissato creando un nuovo thread logico per ogni richiesta e ritornando alla fine la lista di risposte andate a buon fine ottenute con il numero di richieste effettivamente inviate
fn send_requests(seconds: u32, mut requests_per_second: u32, increment: u8, url: String, headers: Vec<String>, query_strings: Vec<String>) -> (Vec<ResponseInfo>, u32) {

    let results: Arc<Mutex<Vec<ResponseInfo>>> = Arc::new(Mutex::new(vec![]));
    let mut requests: Vec<JoinHandle<()>> = vec![];
    let mut sended_requests: u32 = 0;

    for second in 0..seconds {

        if second > 0 {
            requests_per_second = (requests_per_second as f32 * (1.0 + (increment as f32 / 100.0) as f32)).ceil() as u32;
        }

        for _ in 0..requests_per_second {

            let url: String = url.clone();
            let result: Arc<Mutex<Vec<ResponseInfo>>> = Arc::clone(&results);

            let headers: Vec<String> = headers.clone();
            let query_strings: Vec<String> = query_strings.clone();

            let request: JoinHandle<()> = thread::spawn(move || {

                let resp: Result<ResponseInfo, reqwest::Error> = execute_request(url, headers, query_strings);

                if let Ok(res) = resp {
                    result.lock().unwrap().push(res);
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
        request.join().unwrap();
    }

    let results: Vec<ResponseInfo> = results.lock().unwrap().to_owned();

    (results, sended_requests)
}


/// Funzione per eseguire effettivamente la richiesta HTTP, compone la richiesta impostando gli eventuali headers, query strings e l'URL e calcolando il tempo di esecuzione
fn execute_request(url: String, headers: Vec<String>, query_strings: Vec<String>) -> Result<ResponseInfo, reqwest::Error> {
    
    let client: Client = reqwest::blocking::Client::new();
    let mut request_builder: RequestBuilder = client.get(url);

    for header in headers {
        let key: &str = header.split("=").next().unwrap();
        let value: &str = header.split("=").last().unwrap();
        request_builder = request_builder.header(key, value);
    }

    for query_string in query_strings {
        let key: &str = query_string.split("=").next().unwrap();
        let value: &str = query_string.split("=").last().unwrap();
        request_builder = request_builder.query(&[(key, value)]);
    }

    let request_time: Instant = Instant::now();
    let resp: Result<Response, reqwest::Error> = request_builder.send();
    let time: u128 = request_time.elapsed().as_millis();
    
    match resp {
        Ok(res) => Ok(ResponseInfo::new(res.status().as_u16(), time)),
        Err(err) => Err(err)
    }
}



/// Funzione per calcolare le statistiche del test di carico partendo dalle risposte ricevute
/// Ritorna uno StatsInfo contenente tempo medio di esecuzione, tempo minimo, tempo massimo, status codes e relativa conta e coordinate per il grafico finale
fn calc_stats(successful_requests: &Vec<ResponseInfo>) -> StatsInfo {

    let mut average_time: u128 = 0;
    let mut min_time: u128 = 10000;
    let mut max_time: u128 = 0;
    let mut status_codes: HashMap<u16, u32> = HashMap::new();

    let mut points: Vec<(f32, f32)> = vec![(0.0, 0.0)];
    let mut index: f32 = 1.0;

    for request in successful_requests {
        average_time += request.time;
        
        match status_codes.get_mut(&request.status) {
            Some(value) => {
                *value += 1;
            },
            None => {
                status_codes.insert(request.status, 1);
            },
        }

        if request.time < min_time {
            min_time = request.time;
        }

        if request.time > max_time {
            max_time = request.time;
        }

        let point: (f32, f32) = (index, request.time as f32);
        points.push(point);

        index += 1.0;
    }

    StatsInfo { 
        average_time, 
        min_time, 
        max_time, 
        status_codes, 
        points 
    }

}