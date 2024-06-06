use bytes::Bytes;

use tokio::time::{sleep, Duration};

use super::*;


#[tokio::test]
async fn test_db_set_get() {
    let mut db = Db::new();

    let input = (
        String::from("key"),
        Bytes::from_static(b"data"),
    );
        

    db.set(
        input.0.clone(), 
        input.1.clone(),
        None
    );

    assert_eq!(
        Some(input.1), 
        db.get(&input.0),
    );
}

#[tokio::test]
async fn test_set_with_ttl () {
    let mut db = Db::new();

    let input = (
        String::from("key"),
        Bytes::from_static(b"data"),
        Duration::from_millis(100),
    );

    db.set(
        input.0.clone(), 
        input.1.clone(),
        Some(input.2.clone()),
    );

    let state = db.shared.state.lock().unwrap();
    assert!(state.expirations.first().is_some());
    drop(state);
    assert!(db.get(&input.0.clone()).is_some());

    sleep(Duration::from_millis(200)).await;

    assert!(db.get(&input.0.clone()).is_none());
    let state = db.shared.state.lock().unwrap();
    assert!(state.expirations.first().is_none());
    drop(state);

}
