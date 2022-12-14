mod models;

use std::{thread::{JoinHandle, self}, sync::{Mutex, Arc}, time::{Instant, Duration}, collections::HashMap, error::Error};
use reqwest::blocking::{Client, RequestBuilder, Response};
use textplots::{Chart, Shape, Plot};
use models::{ResponseInfo, StatsInfo};


/// Punto di ingresso effettivo della logica che si preoccupa di inviare le richieste e visualizzare statistiche e grafico finale
/// run() ā send_requests() ā execute_request()
/// run() ā calc_stats()
pub fn run(seconds: u32, requests_per_second: u32, increment: u8, url: String, headers: Vec<String>, query_strings: Vec<String>) -> Result<(), Box<dyn Error>> {
    
    println!("šØ Riepilogo:\n\tNumero di richieste al secondo: {}\n\tPer i prossimi {} secondi\n\tAll'URL {}\n", requests_per_second, seconds, url);

    let now: Instant = Instant::now();

    let (successful_requests, sended_requests) = send_requests(seconds, requests_per_second, increment, url, headers, query_strings);

    let successful_requests_number: usize = successful_requests.len();

    println!("\n============================================================");
    println!("              š TEST DI CARICO COMPLETATO š              ");
    println!("============================================================\n");

    println!("š REPORT FINALE");
    println!("š„ Richieste completate con successo: {}/{}", successful_requests_number, sended_requests);

    if successful_requests_number == 0 {
        println!("\nā Nessuna richiesta completata con successo! Controlla che l'applicazione sia online");
        return Ok(());
    }

    println!("š Tempo totale: {} s", now.elapsed().as_secs_f32());

    let stats_info: StatsInfo = calc_stats(&successful_requests);

    println!("šÆ Status codes ricevuti:");

    for status_code in stats_info.status_codes {
        println!("\tStatus code: {} ā {}", status_code.0, status_code.1);
    }

    println!("š Tempo medio: {} ms", stats_info.total_time / successful_requests_number as u128);
    println!("š Richiesta piĆ¹ lenta: {} ms", stats_info.max_time);
    println!("š Richiesta piĆ¹ veloce: {} ms", stats_info.min_time);
    println!("\n");

    Chart::new(200, 55, 0 as f32, (stats_info.points.len() - 1) as f32)
        .lineplot(&Shape::Steps(&stats_info.points))
        .display();

    Ok(())
}


/// Funzione che si preoccupa di gestire le richieste nel tempo prefissato creando un nuovo thread logico per ogni richiesta e ritornando alla fine la lista di risposte andate a buon fine ottenute con il numero di richieste effettivamente inviate
fn send_requests(seconds: u32, mut requests_per_second: u32, increment: u8, url: String, headers: Vec<String>, query_strings: Vec<String>) -> (Vec<ResponseInfo>, u32) {

    if seconds == 0 || requests_per_second == 0 {
        panic!("Valori delle richieste per secondo e/o tempo di esecuzione non validi");
    }

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

        println!("āļø Inviate {} richieste", sended_requests);
        thread::sleep(Duration::from_secs(1));
    }    

    println!("šØ Finalizzazione in corso..");
    
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

        if !header.contains("=") {
            panic!("Header non valido, separatore chiave-valore '=' mancante");
        }

        let key: &str = header.split("=").next().unwrap();
        let value: &str = header.split("=").last().unwrap();
        request_builder = request_builder.header(key, value);
    }

    for query_string in query_strings {

        if !query_string.contains("=") {
            panic!("Query string non valida, separatore chiave-valore '=' mancante");
        }

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

    if successful_requests.len() == 0 {
        return StatsInfo {
            total_time: 0,
            max_time: 0,
            min_time: 0,
            status_codes: HashMap::new(),
            points: vec![]
        };
    }

    let mut total_time: u128 = 0;
    let mut min_time: u128 = 10000;
    let mut max_time: u128 = 0;
    let mut status_codes: HashMap<u16, u32> = HashMap::new();

    let mut points: Vec<(f32, f32)> = vec![(0.0, 0.0)];
    let mut index: f32 = 1.0;

    for request in successful_requests {
        total_time += request.time;
        
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
        total_time, 
        min_time, 
        max_time, 
        status_codes, 
        points 
    }

}



#[cfg(test)]
mod calc_stats_tests {
    use super::*;


    #[test]
    fn empty_vector() {

        let input: Vec<ResponseInfo> = vec![];

        let response: StatsInfo = calc_stats(&input);

        assert_eq!(0, response.total_time);
        assert_eq!(0, response.max_time);
        assert_eq!(0, response.min_time);
        assert_eq!(0, response.status_codes.len());
        assert_eq!(0, response.points.len());
    }


    #[test]
    fn vector_with_single_value() {

        let input: Vec<ResponseInfo> = vec![
            ResponseInfo {
                status: 200,
                time: 50
            }
        ];

        let response: StatsInfo = calc_stats(&input);

        assert_eq!(50, response.total_time);
        assert_eq!(50, response.max_time);
        assert_eq!(50, response.min_time);

        assert_eq!(1, response.status_codes.len());
        assert_eq!((&200, &1), response.status_codes.iter().next().unwrap());

        assert_eq!(2, response.points.len());
        assert_eq!(&(0.0, 0.0), response.points.iter().next().unwrap());
        assert_eq!(&(1.0, 50.0), response.points.iter().last().unwrap());
    }


    #[test]
    fn vector_with_multiple_values() {

        let input: Vec<ResponseInfo> = vec![
            ResponseInfo {
                status: 200,
                time: 25
            },
            ResponseInfo {
                status: 400,
                time: 50
            },
            ResponseInfo {
                status: 200,
                time: 75
            }
        ];

        let response: StatsInfo = calc_stats(&input);

        assert_eq!(150, response.total_time);
        assert_eq!(75, response.max_time);
        assert_eq!(25, response.min_time);
        
        assert_eq!(2, response.status_codes.len());
        assert_eq!(&2, response.status_codes.iter().filter(|status_code| status_code.0 == &200).next().unwrap().1);
        assert_eq!(&1, response.status_codes.iter().filter(|status_code| status_code.0 == &400).next().unwrap().1);

        assert_eq!(4, response.points.len());
        assert_eq!(&(0.0, 0.0), response.points.iter().next().unwrap());
        assert_eq!(&(1.0, 25.0), response.points.iter().nth(1).unwrap());
        assert_eq!(&(2.0, 50.0), response.points.iter().nth(2).unwrap());
        assert_eq!(&(3.0, 75.0), response.points.iter().last().unwrap());

    }
}



#[cfg(test)]
mod execute_request_tests {
    use super::*;
    use mockito::{self, mock, Mock, Matcher};


    #[test]
    fn wrong_url() {

        let url: String = format!("{}/wrong", mockito::server_url());

        let mock: Mock = mock("GET", "/test")
            .expect(0)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, vec![], vec![]);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(501, result.status);
        mock.assert();
    }


    #[test]
    fn no_headers_and_query_strings() {

        let url: String = format!("{}/test", mockito::server_url());

        let mock: Mock = mock("GET", "/test")
            .with_status(200)
            .expect(1)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, vec![], vec![]);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(200, result.status);
        mock.assert();
    }


    #[test]
    fn with_headers() {

        let url: String = format!("{}/test", mockito::server_url());
        let headers: Vec<String> = vec![
            String::from("header_key_1=header_value_1"),
            String::from("header_key_2=header_value_2")
        ];

        let mock: Mock = mock("GET", "/test")
            .with_status(200)
            .match_header("header_key_1", "header_value_1")
            .match_header("header_key_2", "header_value_2")
            .expect(1)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, headers, vec![]);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(200, result.status);
        mock.assert();
        mock.matched();
    }


    #[test]
    fn with_query_strings() {

        let url: String = format!("{}/test", mockito::server_url());
        let query_strings: Vec<String> = vec![
            String::from("query_string_key_1=query_string_value_1"),
            String::from("query_string_key_2=query_string_value_2")
        ];

        let mock: Mock = mock("GET", "/test")
            .with_status(200)
            .match_query(Matcher::UrlEncoded(String::from("query_string_key_1"), String::from("query_string_value_1")))
            .match_query(Matcher::UrlEncoded(String::from("query_string_key_2"), String::from("query_string_value_2")))
            .expect(1)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, vec![], query_strings);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(200, result.status);
        mock.assert();
        mock.matched();
    }


    #[test]
    fn with_headers_and_query_string() {

        let url: String = format!("{}/test", mockito::server_url());
        let headers: Vec<String> = vec![
            String::from("header_key_1=header_value_1"),
            String::from("header_key_2=header_value_2")
        ];
        let query_strings: Vec<String> = vec![
            String::from("query_string_key_1=query_string_value_1"),
            String::from("query_string_key_2=query_string_value_2")
        ];

        let mock: Mock = mock("GET", "/test")
            .with_status(200)
            .match_header("header_key_1", "header_value_1")
            .match_header("header_key_2", "header_value_2")
            .match_query(Matcher::UrlEncoded(String::from("query_string_key_1"), String::from("query_string_value_1")))
            .match_query(Matcher::UrlEncoded(String::from("query_string_key_2"), String::from("query_string_value_2")))
            .expect(1)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, headers, query_strings);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(200, result.status);
        mock.assert();
        mock.matched();
    }


    #[test]
    fn response_400() {

        let url: String = format!("{}/test", mockito::server_url());

        let mock: Mock = mock("GET", "/test")
            .with_status(400)
            .expect(1)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, vec![], vec![]);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(400, result.status);
        mock.assert();
    }


    #[test]
    fn response_500() {

        let url: String = format!("{}/test", mockito::server_url());

        let mock: Mock = mock("GET", "/test")
            .with_status(500)
            .expect(1)
            .create();

        let result: Result<ResponseInfo, reqwest::Error> = execute_request(url, vec![], vec![]);

        let result = match result {
            Ok(res) => res,
            Err(_) => panic!("Error in request execution")
        };

        assert_eq!(500, result.status);
        mock.assert();
    }

}



#[cfg(test)]
mod send_requests_tests {
    use mockito::{Mock, mock};

    use super::*;


    #[test]
    #[should_panic(expected = "Valori delle richieste per secondo e/o tempo di esecuzione non validi")]
    fn seconds_0() {
        send_requests(0, 3, 0, String::from(""), vec![], vec![]);
    }


    #[test]
    #[should_panic(expected = "Valori delle richieste per secondo e/o tempo di esecuzione non validi")]
    fn requests_per_second_0() {
        send_requests(3, 0, 0, String::from(""), vec![], vec![]);
    }


    #[test]
    fn correct() {

        let url: String = format!("{}/test", mockito::server_url());

        let mock: Mock = mock("GET", "/test")
            .with_status(200)
            .expect(9)
            .create();

        let (info, requests_number) = send_requests(3, 3, 0, url, vec![], vec![]);
        
        mock.assert();
        assert_eq!(9, info.iter().filter(|i| i.status == 200).count());
        assert_eq!(9, requests_number);
    }


    #[test]
    fn with_increment() {

        let url: String = format!("{}/test", mockito::server_url());

        let mock: Mock = mock("GET", "/test")
            .with_status(200)
            .expect(12)
            .create();

        let (info, requests_number) = send_requests(3, 3, 1, url, vec![], vec![]);
        
        mock.assert();
        assert_eq!(12, info.iter().filter(|i| i.status == 200).count());
        assert_eq!(12, requests_number);
    }
}