use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use crossbeam::channel::unbounded;
use lazy_static::lazy_static;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time;

mod util;

mod msg {
    rosrust::rosmsg_include!(
        roscpp_tutorials / TwoInts,
        rospy_tutorials / AddTwoInts,
        std_msgs / String
    );
}

fn global_init() -> util::ChildProcessTerminator {
    let roscore = util::run_roscore_for(util::TestVariant::Benchmark);
    rosrust::init("benchmarker");
    roscore
}

lazy_static! {
    static ref ROS_CORE: Option<util::ChildProcessTerminator> = Some(global_init());
}

fn setup() {
    assert!(ROS_CORE.is_some());
}

fn subscribe_publish_relayed(criterion: &mut Criterion) {
    setup();

    let pub_topic = format!("/chatter_at_{}", line!());
    let sub_topic = format!("/chatter_at_{}", line!());

    let _subscriber = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("topic_tools")
            .arg("relay")
            .arg(&pub_topic)
            .arg(&sub_topic),
    );

    let (tx, rx) = unbounded();

    let _log_subscriber =
        rosrust::subscribe::<msg::std_msgs::String, _>(&sub_topic, 2000, move |data| {
            tx.send(data.data).unwrap();
        })
        .unwrap();

    let publisher = rosrust::publish::<msg::std_msgs::String>(&pub_topic, 2000).unwrap();
    publisher.wait_for_subscribers(None).unwrap();

    loop {
        publisher
            .send(msg::std_msgs::String { data: "".into() })
            .unwrap();
        std::thread::sleep(time::Duration::from_millis(100));
        if !rx.is_empty() {
            break;
        }
    }

    publisher
        .send(msg::std_msgs::String {
            data: "ready".into(),
        })
        .unwrap();

    while rx.recv().unwrap() != "ready" {}

    let inner_publisher = publisher.clone();
    let receiver = rx.clone();
    criterion.bench_function("send and receive single message relayed", move |b| {
        let idx = AtomicUsize::new(0);

        b.iter_batched(
            || format!("{}:{}", line!(), idx.fetch_add(1, Ordering::SeqCst)),
            |data| {
                inner_publisher
                    .send(msg::std_msgs::String { data: data.clone() })
                    .unwrap();
                let response = receiver.recv().unwrap();
                assert_eq!(data, response);
            },
            BatchSize::SmallInput,
        );
    });

    let inner_publisher = publisher;
    let receiver = rx;
    // Limit to 8 due to the relay node's buffer limit being 10
    criterion.bench_function("send and receive 8 messages relayed", move |b| {
        let idx = AtomicUsize::new(0);

        b.iter_batched(
            || {
                let id = idx.fetch_add(1, Ordering::SeqCst);
                (0..8)
                    .map(|item| format!("{}:{}:{}", line!(), id, item))
                    .collect::<Vec<String>>()
            },
            |data| {
                for item in &data {
                    inner_publisher
                        .send(msg::std_msgs::String { data: item.clone() })
                        .unwrap();
                }
                for item in &data {
                    let response = receiver.recv().unwrap();
                    assert_eq!(item, &response);
                }
            },
            BatchSize::LargeInput,
        );
    });
}

fn subscribe_publish_directly(criterion: &mut Criterion) {
    setup();

    let topic = format!("/topicat{}", line!());

    let publisher = rosrust::publish::<msg::std_msgs::String>(&topic, 2000).unwrap();

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::std_msgs::String, _>(&topic, 2000, move |message| {
            tx.send(message.data).unwrap();
        })
        .unwrap();
    publisher.wait_for_subscribers(None).unwrap();

    loop {
        publisher
            .send(msg::std_msgs::String { data: "".into() })
            .unwrap();
        if !rx.is_empty() {
            break;
        }
    }

    publisher
        .send(msg::std_msgs::String {
            data: "ready".into(),
        })
        .unwrap();

    while rx.recv().unwrap() != "ready" {}

    let inner_publisher = publisher.clone();
    let receiver = rx.clone();
    criterion.bench_function("send and receive single message directly", move |b| {
        let idx = AtomicUsize::new(0);

        b.iter_batched(
            || format!("{}:{}", line!(), idx.fetch_add(1, Ordering::SeqCst)),
            |data| {
                inner_publisher
                    .send(msg::std_msgs::String { data: data.clone() })
                    .unwrap();
                let response = receiver.recv().unwrap();
                assert_eq!(data, response);
            },
            BatchSize::SmallInput,
        );
    });

    let inner_publisher = publisher;
    let receiver = rx;
    criterion.bench_function("send and receive 100 messages directly", move |b| {
        let idx = AtomicUsize::new(0);

        b.iter_batched(
            || format!("{}:{}", line!(), idx.fetch_add(1, Ordering::SeqCst)),
            |data| {
                for _ in 0..100 {
                    inner_publisher
                        .send(msg::std_msgs::String { data: data.clone() })
                        .unwrap();
                }
                for _ in 0..100 {
                    let response = receiver.recv().unwrap();
                    assert_eq!(data, response);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn call_service(criterion: &mut Criterion) {
    #![allow(clippy::needless_collect)]

    setup();

    let namespace = format!("/namespaceat{}", line!());

    let _roscpp_service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("roscpp_tutorials")
            .arg("add_two_ints_server")
            .arg(format!("__ns:={}", namespace))
            .arg("__name:=roscpp_service")
            .arg("add_two_ints:=add_two_ints_cpp"),
    );

    let _roscpp_service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("add_two_ints_server")
            .arg(format!("__ns:={}", namespace))
            .arg("__name:=rospy_service")
            .arg("add_two_ints:=add_two_ints_py"),
    );

    let _roscpp_service = util::ChildProcessTerminator::spawn_example_bench(
        Command::new("cargo")
            .arg("run")
            .arg("--release")
            .arg("--example")
            .arg("service")
            .arg(format!("__ns:={}", namespace))
            .arg("__name:=rosrust_service")
            .arg("add_two_ints:=add_two_ints_rust"),
    );

    let service_name_inline = format!("service_at_line_{}", line!());

    let _service_raii =
        rosrust::service::<msg::roscpp_tutorials::TwoInts, _>(&service_name_inline, move |req| {
            let sum = req.a + req.b;
            Ok(msg::roscpp_tutorials::TwoIntsRes { sum })
        })
        .unwrap();

    let service_name_cpp = format!("{}/add_two_ints_cpp", namespace);
    let service_name_py = format!("{}/add_two_ints_py", namespace);
    let service_name_rust = format!("{}/add_two_ints_rust", namespace);

    rosrust::wait_for_service(&service_name_cpp, Some(time::Duration::from_secs(10))).unwrap();
    rosrust::wait_for_service(&service_name_py, Some(time::Duration::from_secs(10))).unwrap();
    rosrust::wait_for_service(&service_name_rust, Some(time::Duration::from_secs(10))).unwrap();
    rosrust::wait_for_service(&service_name_inline, Some(time::Duration::from_secs(10))).unwrap();

    let client_original =
        rosrust::client::<msg::roscpp_tutorials::TwoInts>(&service_name_cpp).unwrap();
    let client = client_original.clone();
    criterion.bench_function("call roscpp service once", move |b| {
        b.iter(|| {
            let sum = client
                .req(&msg::roscpp_tutorials::TwoIntsReq { a: 48, b: 12 })
                .unwrap()
                .unwrap()
                .sum;
            assert_eq!(60, sum);
        });
    });
    let client = client_original;
    criterion.bench_function("call roscpp service 50 times in parallel", move |b| {
        b.iter(|| {
            let requests = (0..50)
                .map(|a| client.req_async(msg::roscpp_tutorials::TwoIntsReq { a, b: 5 }))
                .collect::<Vec<_>>();
            for (idx, request) in requests.into_iter().enumerate() {
                assert_eq!(idx as i64 + 5, request.read().unwrap().unwrap().sum);
            }
        });
    });

    let client_original =
        rosrust::client::<msg::rospy_tutorials::AddTwoInts>(&service_name_py).unwrap();
    let client = client_original;
    criterion.bench_function("call rospy service once", move |b| {
        b.iter(|| {
            let sum = client
                .req(&msg::rospy_tutorials::AddTwoIntsReq { a: 48, b: 12 })
                .unwrap()
                .unwrap()
                .sum;
            assert_eq!(60, sum);
        });
    });

    let client_original =
        rosrust::client::<msg::roscpp_tutorials::TwoInts>(&service_name_rust).unwrap();
    let client = client_original.clone();
    criterion.bench_function("call rosrust service once", move |b| {
        b.iter(|| {
            let sum = client
                .req(&msg::roscpp_tutorials::TwoIntsReq { a: 48, b: 12 })
                .unwrap()
                .unwrap()
                .sum;
            assert_eq!(60, sum);
        });
    });
    let client = client_original;
    criterion.bench_function("call rosrust service 50 times in parallel", move |b| {
        b.iter(|| {
            let requests = (0..50)
                .map(|a| client.req_async(msg::roscpp_tutorials::TwoIntsReq { a, b: 5 }))
                .collect::<Vec<_>>();
            for (idx, request) in requests.into_iter().enumerate() {
                assert_eq!(idx as i64 + 5, request.read().unwrap().unwrap().sum);
            }
        });
    });

    let client_original =
        rosrust::client::<msg::roscpp_tutorials::TwoInts>(&service_name_inline).unwrap();
    let client = client_original.clone();
    criterion.bench_function("call inline rosrust service once", move |b| {
        b.iter(|| {
            let sum = client
                .req(&msg::roscpp_tutorials::TwoIntsReq { a: 48, b: 12 })
                .unwrap()
                .unwrap()
                .sum;
            assert_eq!(60, sum);
        });
    });
    let client = client_original;
    criterion.bench_function(
        "call inline rosrust service 50 times in parallel",
        move |b| {
            b.iter(|| {
                let requests = (0..50)
                    .map(|a| client.req_async(msg::roscpp_tutorials::TwoIntsReq { a, b: 5 }))
                    .collect::<Vec<_>>();
                for (idx, request) in requests.into_iter().enumerate() {
                    assert_eq!(idx as i64 + 5, request.read().unwrap().unwrap().sum);
                }
            });
        },
    );
}

criterion_group!(
    benches,
    subscribe_publish_directly,
    subscribe_publish_relayed,
    call_service
);
criterion_main!(benches);
