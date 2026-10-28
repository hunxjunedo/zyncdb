use std::{thread, time::Duration};

use zyncdb::{ResponseFromDbOp, constants::{EVICTION_KEY_NOT_EXIST_ERROR, EVICTION_TTL_NEW, EVICTION_TTL_UPDATE, GET_VALUE_NOT_EXIST_ERROR, REMOVE_KEY_NOT_EXIST_ERROR, REMOVE_KEY_SUCCESFULL, SET_NEW_VALUE, SET_VALUE_UPDATE}, db::Database, db_operations::{DbOperation, GetKey, RemoveKey, SetEvictionPolicy, SetKey}};


#[test]
fn gets_value() {
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    db.data.insert(key.clone(), (value.clone(), None));
    let response = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == value.clone()));
}

#[test]
fn get_fails_for_non_existant_key(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    let response = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
}

#[test]
fn set_new_value(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    let response = SetKey {key: key.clone(), value: value.clone(), ttl: None}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == SET_NEW_VALUE));
    //now check if it actually set the value
    let response_second = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::NormalResponse(x) if x == value.clone()));

}

#[test]
fn update_old_value(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    SetKey {key: key.clone(), value: value.clone(), ttl: None}.execute(&mut db);
    let response = SetKey {key: key.clone(), value: "test_value2".to_string(), ttl: None}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == SET_VALUE_UPDATE));
    //now check if it actually updates the value
    let response_second = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::NormalResponse(x) if x == "test_value2".to_string()));

}

#[test]
fn remove_non_existant_key(){
    let mut db = Database::new();
    let response = RemoveKey {key: "non_existant_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == REMOVE_KEY_NOT_EXIST_ERROR));
}

#[test]
fn remove_existing_key(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    SetKey{key: key.clone(), value: value.clone(), ttl: None}.execute(&mut db);
    let response = RemoveKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == REMOVE_KEY_SUCCESFULL));
    //read again to ensure it does not exist
    let response_second = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
}

#[test]
fn expiry_while_setting_works(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    //first check if it does not expire
    SetKey{key: key.clone(), value: value.clone(), ttl: Some(10)}.execute(&mut db);
    thread::sleep(Duration::from_secs(9));
    let response = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == value));
    //now check if it expires also
    thread::sleep(Duration::from_secs(2));
    let response = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
}

#[test]
fn set_without_ttl_resets_expiry(){
        let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    SetKey{key: key.clone(), value: value.clone(), ttl: Some(3)}.execute(&mut db);
    SetKey{key: key.clone(), value: value.clone(), ttl: None}.execute(&mut db);
    thread::sleep(Duration::from_secs(4));
    let response = GetKey{key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == value));
}

#[test]
fn expire_on_missing_key_reports_missing(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    let response = SetEvictionPolicy{key: key.clone(), ttl: 10}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == EVICTION_KEY_NOT_EXIST_ERROR));
}

#[test]
fn expire_on_missing_key_does_nothing(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    SetEvictionPolicy{key: key.clone(), ttl: 10}.execute(&mut db);
    let response = GetKey{key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
}

#[test]
fn expiry_through_set_eviction_policy_works(){
    let mut db = Database::new();
    let key = String::from("test_key");
    let value = String::from("test_value");
    SetKey{key: key.clone(), value: value.clone(), ttl: None}.execute(&mut db);

    //1. check if the command returns correct message, and adds the expiry to the db correctly
    let response = SetEvictionPolicy{key: key.clone(), ttl: 10}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == EVICTION_TTL_NEW));
    //check after 10 seconds, it should be gone
    thread::sleep(Duration::from_secs(10));
    let response = GetKey{key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
    
    //3. check that command returns correct message, correctly updates the expiry, and esnure data exists before expiry but removed after
    SetKey{key: key.clone(), value: value.clone(), ttl: Some(10)}.execute(&mut db);
    thread::sleep(Duration::from_secs(9));
    let response = SetEvictionPolicy{key: key.clone(), ttl: 10}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == EVICTION_TTL_UPDATE));
    thread::sleep(Duration::from_secs(8));
    let response = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == value.clone()));
    thread::sleep(Duration::from_secs(3));
        let response = GetKey {key: key.clone()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));

}