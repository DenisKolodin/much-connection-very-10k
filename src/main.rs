use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::net;
use std::io::{Read, Write};

fn main() {
    const THREADS_NUM: usize = 1000;

    println!("Starting...");
    let listener = Arc::new(Mutex::new(net::TcpListener::bind("127.0.0.1:10042").unwrap()));
    let counter = Arc::new((Mutex::new(0), Condvar::new()));
    let mut threads: Vec<_> = Vec::with_capacity(THREADS_NUM);
    for i in 0..THREADS_NUM {
        println!("Generate thread {}", i);
        let listener = listener.clone();
        let counter = counter.clone();
        let t = thread::spawn(move || {
            let (mut stream, _) = listener.lock().unwrap().accept().unwrap();
            println!("Thread {} accepted", i);
            let expected = [13, 10, 13, 10];
            let mut eidx = 0;
            let mut buf = [0; 10];
            'outer: loop {
                match stream.read(&mut buf) {
                    Ok(v) => {
                        for idx in 0..v {
                            let c = buf[idx];
                            if expected[eidx] == c {
                                eidx += 1;
                                if eidx >= expected.len() {
                                    break 'outer;
                                }
                            } else {
                                eidx = 0;
                            }
                        }
                    },
                    Err(_) => break,
                }
            }
            println!("Request {:?}", buf);
            let response = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n";
            println!("Thread {} write response", i);
            stream.write_all(response).unwrap();

            stream.shutdown(std::net::Shutdown::Both).unwrap();

            let &(ref lock, ref condvar) = &*counter;
            let mut value = lock.lock().unwrap();
            println!("{}: {}", i, *value);
            *value += 1;
            if *value == THREADS_NUM {
                condvar.notify_one();
            }
        });
        threads.push(t);
    }
    let &(ref lock, ref condvar) = &*counter;
    let mut value = lock.lock().unwrap();
    while *value != THREADS_NUM {
        value = condvar.wait(value).unwrap();
    }
    let n: usize = *value;
    println!("{:?}", n);
}
