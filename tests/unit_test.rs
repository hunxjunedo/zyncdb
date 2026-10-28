use zyncdb::{ResponseFromDbOp, commands::Get, constants::{GET_VALUE_NOT_EXIST_ERROR, REMOVE_KEY_NOT_EXIST_ERROR, REMOVE_KEY_SUCCESFULL, SET_NEW_VALUE, SET_VALUE_UPDATE}, db::{self, Database}, db_operations::{DbOperation, GetKey, RemoveKey, SetKey}};

#[test]
fn gets_value() {
    let mut db = Database::new();
    db.data.insert("test_key".to_string(), ("test_value".to_string(), None));
    let response = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == "test_value".to_string()));
}

#[test]
fn get_fails_for_non_existant_key(){
    let mut db = Database::new();
    let response = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
}

#[test]
fn set_new_value(){
    let mut db = Database::new();
    let response = SetKey {key: "test_key".to_string(), value: "test_value".to_string(), ttl: None}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == SET_NEW_VALUE));
    //now check if it actually set the value
    let response_second = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::NormalResponse(x) if x == "test_value".to_string()));

}

#[test]
fn update_old_value(){
    let mut db = Database::new();
    SetKey {key: "test_key".to_string(), value: "test_value".to_string(), ttl: None}.execute(&mut db);
    let response = SetKey {key: "test_key".to_string(), value: "test_value2".to_string(), ttl: None}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == SET_VALUE_UPDATE));
    //now check if it actually updates the value
    let response_second = GetKey {key: "test_key".to_string()}.execute(&mut db);
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
    SetKey{key: "test_key".to_string(), value: "test_value".to_string(), ttl: None}.execute(&mut db);
    let response = RemoveKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response, ResponseFromDbOp::NormalResponse(x) if x == REMOVE_KEY_SUCCESFULL));
    //read again to ensure it does not exist
    let response_second = GetKey {key: "test_key".to_string()}.execute(&mut db);
    assert!(matches!(response_second, ResponseFromDbOp::Error(x) if x == GET_VALUE_NOT_EXIST_ERROR));
}